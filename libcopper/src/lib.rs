use proc_macro2::{TokenStream, TokenTree};
use syn::{
	parse::{Parse, ParseStream, Result},
	parse_macro_input,
	Token,
	token,
	punctuated::Punctuated,
	braced,
};
use quote::{quote, ToTokens};
use std::{
	collections::HashMap,
	fmt,
};

mod punc {
	syn::custom_punctuation!(ThinArrow, ->);
}

mod kw {
	syn::custom_keyword!(mul);
	syn::custom_keyword!(div);
}

#[derive(Clone, Debug)]
struct Unit {
	tokens: TokenStream,
	key: String, // canonical form, used for Hash/Eq
}

impl Unit {
	fn new() -> Self {
		Self { tokens: TokenStream::new(), key: String::new() }
	}
	
	fn extend(&mut self, tt: impl IntoIterator<Item = TokenTree>) {
		self.tokens.extend(tt);
		self.key = self.tokens.to_string(); // recompute canonical key
	}
}

impl PartialEq for Unit {
	fn eq(&self, other: &Self) -> bool { self.key == other.key }
}
impl Eq for Unit {}

impl std::hash::Hash for Unit {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.key.hash(state); }
}

// keep ToTokens so it drops straight into quote!{}
impl quote::ToTokens for Unit {
	fn to_tokens(&self, out: &mut TokenStream) { self.tokens.to_tokens(out); }
}

enum Conversion {
	Mul{
		mul_token: kw::mul,
		by: TokenTree,
	},
	Div {
		div_token: kw::div,
		by: TokenTree,
	}
	// Fun(?),
}

impl fmt::Debug for Conversion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Mul { by, .. } => f.debug_struct("Conversion::Mul").field("by", by).finish(),
			Self::Div { by, .. } => f.debug_struct("Conversion::Div").field("by", by).finish()
		}
	}
}

impl Parse for Conversion {
	fn parse(input: ParseStream) -> Result<Self> {
		let lookahead = input.lookahead1();
		if lookahead.peek(kw::mul) {
			Ok(Self::Mul {
				mul_token: input.parse()?,
				by: input.parse()?,
			})
		} else if lookahead.peek(kw::div) {
			Ok(Self::Div {
				div_token: input.parse()?,
				by: input.parse()?,
			})
		} else  {
			Err(lookahead.error())
		}
	}
}

impl ToTokens for Conversion {
	fn to_tokens(&self, output: &mut TokenStream) {
		match self {
			Self::Mul { by, .. } => output.extend(quote! { |x| x * #by }),
			Self::Div { by, .. } => output.extend(quote! { |x| x / #by }),
		}
	}
}

enum OutputType {
	Single(ConversionRHS),
	Multiple(ConversionRHSs),
}

impl Parse for OutputType {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(if input.peek(token::Brace) {
			Self::Multiple(input.parse()?)
		} else {
			Self::Single(input.parse()?)
		})
	}
}

struct ConversionRHS {
	output_unit: Unit,
	arrow_token: Token!(=>),
	conversion: Conversion,
}

impl Parse for ConversionRHS {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut output_unit = Unit::new();
		while !input.is_empty() && !input.peek(Token!(=>)) {
			output_unit.extend(input.parse::<TokenTree>());
		}
		let arrow_token = input.parse::<Token!(=>)>()?;
		let  conversion = input.parse::<Conversion>()?;
		Ok(Self {output_unit, arrow_token, conversion })
	}
}

struct ConversionRHSs {
	brace_token: token::Brace,
	outputs: Punctuated::<ConversionRHS, Token!(,)>,
}

impl Parse for ConversionRHSs {
	fn parse(input: ParseStream) -> Result<Self> {
		let content;
		Ok(Self {
			brace_token: braced!(content in input),
			outputs: content.parse_terminated(ConversionRHS::parse, Token!(,))?,
		})
	}
}

struct ConversionRow {
	input_unit: Unit,
	arrow_token: punc::ThinArrow,
	outputs: OutputType,
}

impl Parse for ConversionRow {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut input_unit = Unit::new();
		while !input.is_empty() && !input.peek(punc::ThinArrow) {
			input_unit.extend(input.parse::<TokenTree>());
		}
		let arrow_token = input.parse::<punc::ThinArrow>()?;
		let outputs = input.parse::<OutputType>()?;
		Ok(Self {input_unit, arrow_token, outputs})
	}
}

#[derive(Debug)]
struct ConcreteConversion {
	input_unit: Unit,
	output_unit: Unit,
	conversion: Conversion,
}

impl ConcreteConversion {
	fn inverse(&self) -> Option<Self> {
		let Self { input_unit, output_unit, conversion } = self;
		match conversion {
			Conversion::Mul{ mul_token, by } => Some(Self { input_unit: output_unit.clone(), output_unit: input_unit.clone(), conversion: Conversion::Div{div_token: kw::div(mul_token.span), by: by.clone()}  }),
			Conversion::Div{ div_token, by } => Some(Self { input_unit: output_unit.clone(), output_unit: input_unit.clone(), conversion: Conversion::Mul{mul_token: kw::mul(div_token.span), by: by.clone()}  }),
		}
	}
}

struct ConversionTable {
	explicit_conversions: Vec<ConcreteConversion>,
	implicit_conversions: Vec<ConcreteConversion>,
}

impl Parse for ConversionTable {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut explicit_conversions: Vec<ConcreteConversion> = Vec::new();
		for row in Punctuated::<ConversionRow, Token!(,)>::parse_terminated(input)? {
			let input_unit = row.input_unit;
			match row.outputs {
				OutputType::Single(ConversionRHS { output_unit, conversion, ..}) => {explicit_conversions.push(ConcreteConversion {input_unit, output_unit, conversion})},
				OutputType::Multiple(ConversionRHSs{ outputs, .. }) => {
					explicit_conversions.extend(outputs.into_iter().map(|ConversionRHS{output_unit, conversion, ..}| ConcreteConversion{input_unit: input_unit.clone(), output_unit, conversion}))
				},
			}
		}
		let  implicit_conversions: Vec<ConcreteConversion> = explicit_conversions.iter().filter_map(ConcreteConversion::inverse).collect();
		Ok(Self {explicit_conversions, implicit_conversions})
	}
}

#[proc_macro]
pub fn make_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut table = parse_macro_input!(input as ConversionTable);
	let mut tree: HashMap<Unit, HashMap<Unit, Conversion>> = HashMap::new();
	for ConcreteConversion { input_unit, output_unit, conversion } in table.implicit_conversions.drain(..).chain(table.explicit_conversions.drain(..)) {
		tree.entry(input_unit).or_default().insert(output_unit, conversion);
	}
	let mut branches = Vec::new();
	for (input_unit, outputs) in tree.iter() {
		let mut leaves = Vec::new();
		for (output_unit, conversion) in outputs.iter() {
			leaves.push(quote! { (#output_unit, (#conversion) as ConversionFunc) });
		}
		branches.push(quote!{(#input_unit, HashMap::from([#(#leaves),*]))});
	}
	let expanded = quote! {HashMap::from([#(#branches),*])};
	proc_macro::TokenStream::from(expanded)
}