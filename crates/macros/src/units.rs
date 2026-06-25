use std::iter::once;

use quote::{format_ident, quote};
use syn::{
	Ident, LitStr, Token, Visibility, parenthesized,
	parse::{Parse, ParseStream, Result},
	parse_macro_input,
	punctuated::Punctuated,
	token,
};

use crate::{
	multipliers::{DATA_METRIC_MULTIPLIERS, IEC_MULTIPLIERS, Multiplier, NONDATA_METRIC_MULTIPLIERS},
	token::kw,
};

fn name_to_ident(name: &str) -> String {
	let mut chars = name.chars();
	chars.next().map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

struct IdentAndName(Ident, Token!(,), LitStr);

impl Parse for IdentAndName {
	fn parse(input: ParseStream) -> Result<Self> {
		if input.peek(Ident) {
			Ok(Self(input.parse()?, input.parse()?, input.parse()?))
		} else {
			let name: LitStr = input.parse()?;
			Ok(Self(format_ident!("{}", name_to_ident(&name.value())), <Token!(,)>::default(), name))
		}
	}
}

#[derive(Clone)]
struct SingleUnit {
	#[allow(dead_code)]
	paren_token: token::Paren,
	ident: Ident,
	#[allow(dead_code)]
	comma_token1: Token!(,),
	name: LitStr,
	#[allow(dead_code)]
	comma_token2: Token!(,),
	plural: LitStr,
	#[allow(dead_code)]
	comma_token3: Token!(,),
	symbols: Punctuated<LitStr, Token!(,)>,
}

impl Parse for SingleUnit {
	fn parse(input: ParseStream<'_>) -> Result<Self> {
		let content;
		let paren_token = parenthesized!(content in input);
		let IdentAndName(ident, comma_token1, name) = content.parse()?;
		Ok(Self {
			paren_token,
			ident,
			comma_token1,
			name,
			comma_token2: content.parse()?,
			plural: content.parse()?,
			comma_token3: content.parse()?,
			symbols: content.parse_terminated(<LitStr as Parse>::parse, Token![,])?,
		})
	}
}

enum CompositeType {
	Metric(#[allow(dead_code)] kw::metric),
	Data(#[allow(dead_code)] kw::data),
}

impl CompositeType {
	fn multipliers(&self) -> Box<dyn Iterator<Item = &Multiplier> + '_> {
		match self {
			Self::Metric(_) => Box::new(DATA_METRIC_MULTIPLIERS.iter().chain(NONDATA_METRIC_MULTIPLIERS.iter())),
			Self::Data(_) => Box::new(DATA_METRIC_MULTIPLIERS.iter().chain(IEC_MULTIPLIERS.iter())),
		}
	}
}

impl Parse for CompositeType {
	fn parse(input: ParseStream) -> Result<Self> {
		let lookahead = input.lookahead1();
		if lookahead.peek(kw::metric) {
			Ok(Self::Metric(input.parse()?))
		} else if lookahead.peek(kw::data) {
			Ok(Self::Data(input.parse()?))
		} else {
			Err(lookahead.error())
		}
	}
}

struct CompositeUnit {
	#[allow(dead_code)]
	composite_token: CompositeType,
	paren_token: token::Paren,
	ident: Ident,
	#[allow(dead_code)]
	comma_token1: Token!(,),
	name: LitStr,
	#[allow(dead_code)]
	comma_token2: Token!(,),
	plural: LitStr,
	#[allow(dead_code)]
	comma_token3: Token!(,),
	symbol: LitStr,
}

impl CompositeUnit {
	fn multiply(&self, multiplier: &Multiplier) -> SingleUnit {
		SingleUnit {
			paren_token: self.paren_token,
			ident: Ident::new(
				&name_to_ident(&(multiplier.name().to_owned() + &self.ident.to_string().to_lowercase())),
				self.ident.span(),
			),
			comma_token1: self.comma_token1,
			name: LitStr::new(&(multiplier.name().to_owned() + &self.name.value()), self.name.span()),
			comma_token2: self.comma_token2,
			plural: LitStr::new(&(multiplier.name().to_owned() + &self.plural.value()), self.plural.span()),
			comma_token3: self.comma_token3,
			symbols: multiplier
				.symbols()
				.iter()
				.map(|p| LitStr::new(&((*p).to_owned() + &self.symbol.value()), self.symbol.span()))
				.collect(),
		}
	}

	fn base(&self) -> SingleUnit {
		SingleUnit {
			paren_token: self.paren_token,
			ident: self.ident.clone(),
			comma_token1: self.comma_token1,
			name: self.name.clone(),
			comma_token2: self.comma_token2,
			plural: self.plural.clone(),
			comma_token3: self.comma_token3,
			symbols: once(self.symbol.clone()).collect(),
		}
	}

	fn expand(&self) -> Punctuated<SingleUnit, Token!(,)> {
		once(self.base()).chain(self.composite_token.multipliers().map(|m| self.multiply(m))).collect()
	}
}

impl Parse for CompositeUnit {
	fn parse(input: ParseStream) -> Result<Self> {
		let composite_token = input.parse()?;
		let content;
		let paren_token = parenthesized!(content in input);
		let IdentAndName(ident, comma_token1, name) = content.parse()?;
		Ok(Self {
			composite_token,
			paren_token,
			ident,
			comma_token1,
			name,
			comma_token2: content.parse()?,
			plural: content.parse()?,
			comma_token3: content.parse()?,
			symbol: content.parse()?,
		})
	}
}

enum Unit {
	Single(SingleUnit),
	Composite(CompositeUnit),
}

impl Parse for Unit {
	fn parse(input: ParseStream) -> Result<Self> {
		let lookahead = input.lookahead1();
		if lookahead.peek(kw::metric) || lookahead.peek(kw::data) {
			Ok(Self::Composite(input.parse()?))
		} else if lookahead.peek(token::Paren) {
			Ok(Self::Single(input.parse()?))
		} else {
			Err(lookahead.error())
		}
	}
}

struct UnitDecls {
	vis: Visibility,
	ident: Ident,
	#[allow(dead_code)]
	paren_token: token::Paren,
	units: Punctuated<Unit, Token!(,)>,
}

impl Parse for UnitDecls {
	fn parse(input: ParseStream) -> Result<Self> {
		let content;
		Ok(Self {
			vis: input.parse()?,
			ident: input.parse()?,
			paren_token: parenthesized!(content in input),
			units: Punctuated::parse_terminated(&content)?,
		})
	}
}

impl UnitDecls {
	fn expand(&self) -> Punctuated<SingleUnit, Token!(,)> {
		let mut units = Punctuated::<SingleUnit, Token!(,)>::new();
		self.units.iter().for_each(|unit| match unit {
			Unit::Single(u) => units.push((*u).clone()),
			Unit::Composite(u) => units.extend(u.expand()),
		});
		units
	}
}

pub(crate) fn declare_units_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(tokens as UnitDecls);
	let mut enum_vars = Vec::new();
	let mut name_arms = Vec::new();
	let mut plural_arms = Vec::new();
	let mut symbol_arms = Vec::new();
	let mut parse_arms = Vec::new();
	for unit in input.expand() {
		let SingleUnit { ident, name, plural, symbols, .. } = unit;
		enum_vars.push(ident.clone());
		name_arms.push(quote! {Self::#ident => #name});
		plural_arms.push(quote! {Self::#ident => #plural});
		let symbols = symbols.iter().clone().collect::<Vec<&LitStr>>();
		symbol_arms.push(quote! {Self::#ident => &[#(#symbols),*]});
		parse_arms.push(quote! {#(#symbols)|* => Ok(Self::#ident)});
	}
	let UnitDecls { vis, ident, .. } = input;
	let expanded = quote! {
		#vis enum #ident {
			#(#enum_vars),*
		}

		impl #ident {
			pub fn name(&self) -> &'static str {
				match self {
					#(#name_arms),*
				}
			}

			pub fn plural(&self) -> &'static str {
				match self {
					#(#plural_arms),*
				}
			}

			pub fn symbols(&self) -> &[&'static str] {
				match self {
					#(#symbol_arms),*
				}
			}
		}

		impl std::str::FromStr for #ident {
			type Err = ParseUnitError;

			fn from_str(input: &str) -> Result<Self, Self::Err> {
				match input {
					#(#parse_arms),*,
					_ => Err(Self::Err::VariantNotFound(input.to_owned())),
				}
			}
		}

		impl std::fmt::Display for #ident {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.write_str(self.name())
			}
		}
	};
	proc_macro::TokenStream::from(expanded)
}
