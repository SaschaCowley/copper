use std::{collections::HashMap, fmt, iter::once};

use proc_macro2::{Literal, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
	Ident, Path, Token, braced, parenthesized,
	parse::{self, Parse, ParseStream, Result},
	parse_macro_input,
	punctuated::Punctuated,
	token,
};

use crate::{
	kw,
	multipliers::{DATA_METRIC_MULTIPLIERS, IEC_MULTIPLIERS, Multiplier, NONDATA_METRIC_MULTIPLIERS},
	names::multiply_ident,
};

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
	fn eq(&self, other: &Self) -> bool {
		self.key == other.key
	}
}
impl Eq for Unit {}

impl std::hash::Hash for Unit {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.key.hash(state);
	}
}

// keep ToTokens so it drops straight into quote!{}
impl quote::ToTokens for Unit {
	fn to_tokens(&self, out: &mut TokenStream) {
		self.tokens.to_tokens(out);
	}
}

enum Conversion {
	Mul {
		mul_token: kw::mul,
		by: TokenTree,
	},
	Div {
		div_token: kw::div,
		by: TokenTree,
	},
	Add {
		add_token: kw::add,
		by: TokenTree,
	},
	Sub {
		sub_token: kw::sub,
		by: TokenTree,
	},
	Fun {
		#[allow(dead_code)]
		fun_token: kw::fun,
		#[allow(dead_code)]
		paren_token: token::Paren,
		fun: TokenStream,
	},
}

impl fmt::Debug for Conversion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Mul { by, .. } => f.debug_struct("Conversion::Mul").field("by", by).finish(),
			Self::Div { by, .. } => f.debug_struct("Conversion::Div").field("by", by).finish(),
			Self::Add { by, .. } => f.debug_struct("Conversion::Add").field("by", by).finish(),
			Self::Sub { by, .. } => f.debug_struct("Conversion::Sub").field("by", by).finish(),
			Self::Fun { fun, .. } => f.debug_struct("Conversion::Fun").field("fun", fun).finish(),
		}
	}
}

impl Parse for Conversion {
	fn parse(input: ParseStream) -> Result<Self> {
		let lookahead = input.lookahead1();
		if lookahead.peek(kw::mul) {
			Ok(Self::Mul { mul_token: input.parse()?, by: input.parse()? })
		} else if lookahead.peek(kw::div) {
			Ok(Self::Div { div_token: input.parse()?, by: input.parse()? })
		} else if lookahead.peek(kw::add) {
			Ok(Self::Add { add_token: input.parse()?, by: input.parse()? })
		} else if lookahead.peek(kw::sub) {
			Ok(Self::Sub { sub_token: input.parse()?, by: input.parse()? })
		} else if lookahead.peek(kw::fun) {
			let content;
			Ok(Self::Fun {
				fun_token: input.parse()?,
				paren_token: parenthesized!(content in input),
				fun: content.parse()?,
			})
		} else {
			Err(lookahead.error())
		}
	}
}

impl ToTokens for Conversion {
	fn to_tokens(&self, output: &mut TokenStream) {
		match self {
			Self::Mul { by, .. } => output.extend(quote! { |x| x * #by }),
			Self::Div { by, .. } => output.extend(quote! { |x| x / #by }),
			Self::Add { by, .. } => output.extend(quote! { |x| x + #by }),
			Self::Sub { by, .. } => output.extend(quote! { |x| x - #by }),
			Self::Fun { fun, .. } => output.extend(quote! {#fun}),
		}
	}
}

enum OutputType {
	Single(ConversionRHS),
	Multiple(ConversionRHSs),
}

impl Parse for OutputType {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(if input.peek(token::Brace) { Self::Multiple(input.parse()?) } else { Self::Single(input.parse()?) })
	}
}

struct ConversionRHS {
	output_unit: Unit,
	#[allow(dead_code)]
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
		let conversion = input.parse::<Conversion>()?;
		Ok(Self { output_unit, arrow_token, conversion })
	}
}

struct ConversionRHSs {
	#[allow(dead_code)]
	brace_token: token::Brace,
	outputs: Punctuated<ConversionRHS, Token!(,)>,
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
	#[allow(dead_code)]
	arrow_token: Token!(->),
	outputs: OutputType,
}

impl Parse for ConversionRow {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut input_unit = Unit::new();
		while !input.is_empty() && !input.peek(Token!(->)) {
			input_unit.extend(input.parse::<TokenTree>());
		}
		let arrow_token = input.parse::<Token!(->)>()?;
		let outputs = input.parse::<OutputType>()?;
		Ok(Self { input_unit, arrow_token, outputs })
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
			Conversion::Mul { mul_token, by } => Some(Self {
				input_unit: output_unit.clone(),
				output_unit: input_unit.clone(),
				conversion: Conversion::Div { div_token: kw::div(mul_token.span), by: by.clone() },
			}),
			Conversion::Div { div_token, by } => Some(Self {
				input_unit: output_unit.clone(),
				output_unit: input_unit.clone(),
				conversion: Conversion::Mul { mul_token: kw::mul(div_token.span), by: by.clone() },
			}),
			Conversion::Add { add_token, by } => Some(Self {
				input_unit: output_unit.clone(),
				output_unit: input_unit.clone(),
				conversion: Conversion::Sub { sub_token: kw::sub(add_token.span), by: by.clone() },
			}),
			Conversion::Sub { sub_token, by } => Some(Self {
				input_unit: output_unit.clone(),
				output_unit: input_unit.clone(),
				conversion: Conversion::Add { add_token: kw::add(sub_token.span), by: by.clone() },
			}),
			Conversion::Fun { .. } => None,
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
				OutputType::Single(ConversionRHS { output_unit, conversion, .. }) => {
					explicit_conversions.push(ConcreteConversion { input_unit, output_unit, conversion })
				}
				OutputType::Multiple(ConversionRHSs { outputs, .. }) => explicit_conversions.extend(
					outputs.into_iter().map(|ConversionRHS { output_unit, conversion, .. }| ConcreteConversion {
						input_unit: input_unit.clone(),
						output_unit,
						conversion,
					}),
				),
			}
		}
		let implicit_conversions: Vec<ConcreteConversion> =
			explicit_conversions.iter().filter_map(ConcreteConversion::inverse).collect();
		Ok(Self { explicit_conversions, implicit_conversions })
	}
}

pub(crate) fn make_table_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut table = parse_macro_input!(input as ConversionTable);
	let mut tree: HashMap<Unit, HashMap<Unit, Conversion>> = HashMap::new();
	for ConcreteConversion { input_unit, output_unit, conversion } in
		table.implicit_conversions.drain(..).chain(table.explicit_conversions.drain(..))
	{
		tree.entry(input_unit).or_default().insert(output_unit, conversion);
	}
	let mut branches = Vec::new();
	for (input_unit, outputs) in tree.iter() {
		let mut leaves = Vec::new();
		for (output_unit, conversion) in outputs.iter() {
			leaves.push(quote! { (#output_unit, (#conversion) as ConversionFunc) });
		}
		branches.push(quote! {(#input_unit, HashMap::from([#(#leaves),*]))});
	}
	let expanded = quote! {HashMap::from([#(#branches),*])};
	proc_macro::TokenStream::from(expanded)
}

struct SingleOutput {
	unit: Ident,
	arrow_token: Token!(=>),
	conversion: Conversion,
}

impl Parse for SingleOutput {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(Self { unit: input.parse()?, arrow_token: input.parse()?, conversion: input.parse()? })
	}
}

struct MultipleOutputs {
	brace_token: token::Brace,
	outputs: Punctuated<SingleOutput, Token!(,)>,
}

impl Parse for MultipleOutputs {
	fn parse(input: ParseStream) -> Result<Self> {
		let content;
		Ok(Self { brace_token: braced!(content in input), outputs: content.call(Punctuated::parse_terminated)? })
	}
}

enum Output {
	Single(SingleOutput),
	Multiple(MultipleOutputs),
}

impl Parse for Output {
	fn parse(input: ParseStream) -> Result<Self> {
		if input.peek(token::Brace) { input.parse().map(Self::Multiple) } else { input.parse().map(Self::Single) }
	}
}

struct OutputIterator(Box<dyn Iterator<Item = SingleOutput>>);

impl Iterator for OutputIterator {
	type Item = SingleOutput;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl IntoIterator for Output {
	type Item = SingleOutput;
	type IntoIter = OutputIterator;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Self::Single(o) => OutputIterator(Box::new(once(o))),
			Self::Multiple(o) => OutputIterator(Box::new(o.outputs.into_iter())),
		}
	}
}

struct ExplicitRow {
	input_unit: Ident,
	fat_arrow_token: Token!(->),
	output: Output,
}

impl Parse for ExplicitRow {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(Self { input_unit: input.parse()?, fat_arrow_token: input.parse()?, output: input.parse()? })
	}
}

#[derive(Clone)]
enum Derivation {
	Metric(kw::metric),
	Data(kw::data),
}

impl Parse for Derivation {
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

impl Derivation {
	fn multipliers(&self) -> Box<dyn Iterator<Item = &'static Multiplier>> {
		match self {
			Self::Metric(_) => Box::new(DATA_METRIC_MULTIPLIERS.iter().chain(NONDATA_METRIC_MULTIPLIERS.iter())),
			Self::Data(_) => Box::new(DATA_METRIC_MULTIPLIERS.iter().chain(IEC_MULTIPLIERS.iter())),
		}
	}
}

struct DerivedRow {
	derivation: Derivation,
	paren_token: token::Paren,
	units: Punctuated<Ident, Token![,]>,
}

impl Parse for DerivedRow {
	fn parse(input: ParseStream) -> Result<Self> {
		let content;
		Ok(Self {
			derivation: input.parse()?,
			paren_token: parenthesized!(content in input),
			units: content.call(Punctuated::parse_terminated)?,
		})
	}
}

impl DerivedRow {
	fn derive(base: &Ident, multiplier: &Multiplier) -> [ConcreteConversion2; 2] {
		let derived = multiply_ident(base, multiplier.name());
		let factor: TokenTree = Literal::f64_suffixed(multiplier.factor()).into();
		[
			ConcreteConversion2(
				derived.clone(),
				base.clone(),
				Conversion::Mul { mul_token: kw::mul(Span::call_site()), by: factor.clone() },
			),
			ConcreteConversion2(
				base.clone(),
				derived,
				Conversion::Div { div_token: kw::div(Span::call_site()), by: factor },
			),
		]
	}
}

enum Row {
	Explicit(ExplicitRow),
	Derived(DerivedRow),
}

impl Parse for Row {
	fn parse(input: ParseStream) -> Result<Self> {
		if input.peek2(token::Paren) { input.parse().map(Self::Derived) } else { input.parse().map(Self::Explicit) }
	}
}

struct Table {
	name: Path,
	sep_token: Token!(::),
	brace_token: token::Brace,
	rows: Punctuated<Row, Token!(,)>,
}

impl Parse for Table {
	fn parse(input: ParseStream) -> Result<Self> {
		let content;
		Ok(Self {
			name: Path::parse_mod_style(input)?,
			sep_token: input.parse()?,
			brace_token: braced!(content in input),
			rows: content.call(Punctuated::parse_terminated)?,
		})
	}
}

struct ConcreteConversion2(Ident, Ident, Conversion);
type ExplicitImplicitConversionPair = (Option<ConcreteConversion2>, Option<ConcreteConversion2>);

impl ConcreteConversion2 {
	fn inverse(&self) -> Option<Self> {
		let Self(input_unit, output_unit, conversion) = self;
		match conversion {
			Conversion::Mul { mul_token, by } => Some(Self(
				output_unit.clone(),
				input_unit.clone(),
				Conversion::Div { div_token: kw::div(mul_token.span), by: by.clone() },
			)),
			Conversion::Div { div_token, by } => Some(Self(
				output_unit.clone(),
				input_unit.clone(),
				Conversion::Mul { mul_token: kw::mul(div_token.span), by: by.clone() },
			)),
			Conversion::Add { add_token, by } => Some(Self(
				output_unit.clone(),
				input_unit.clone(),
				Conversion::Sub { sub_token: kw::sub(add_token.span), by: by.clone() },
			)),
			Conversion::Sub { sub_token, by } => Some(Self(
				output_unit.clone(),
				input_unit.clone(),
				Conversion::Add { add_token: kw::add(sub_token.span), by: by.clone() },
			)),
			Conversion::Fun { .. } => None,
		}
	}
}

struct RowsIterator(Box<dyn Iterator<Item = ExplicitImplicitConversionPair>>);

impl Iterator for RowsIterator {
	type Item = ExplicitImplicitConversionPair;
	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl IntoIterator for Row {
	type Item = ExplicitImplicitConversionPair;
	type IntoIter = RowsIterator;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Self::Explicit(row) => RowsIterator(Box::new(row.into_iter().map(|row| {
				let inverse = row.inverse();
				(Some(row), inverse)
			}))),
			Self::Derived(row) => RowsIterator(Box::new(row.into_iter().map(|row| (None, Some(row))))),
		}
	}
}

struct RowIterator(Box<dyn Iterator<Item = ConcreteConversion2>>);
impl Iterator for RowIterator {
	type Item = ConcreteConversion2;
	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl IntoIterator for ExplicitRow {
	type Item = ConcreteConversion2;
	type IntoIter = RowIterator;

	fn into_iter(self) -> Self::IntoIter {
		RowIterator(Box::new(
			self.output.into_iter().map(move |o| ConcreteConversion2(self.input_unit.clone(), o.unit, o.conversion)),
		))
	}
}

impl IntoIterator for DerivedRow {
	type Item = ConcreteConversion2;
	type IntoIter = RowIterator;

	fn into_iter(self) -> Self::IntoIter {
		RowIterator(Box::new(self.units.into_iter().flat_map(move |unit| {
			self.derivation.multipliers().flat_map(move |multiplier| Self::derive(&unit, multiplier))
		})))
	}
}
