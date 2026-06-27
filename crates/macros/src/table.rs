use std::{collections::HashMap, fmt, iter::once};

use proc_macro2::{Literal, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
	Ident, Path, Token, Visibility, braced, parenthesized,
	parse::{Parse, ParseStream, Result},
	parse_macro_input,
	punctuated::Punctuated,
	token,
};

use crate::{
	kw,
	multipliers::{DATA_METRIC_MULTIPLIERS, IEC_MULTIPLIERS, Multiplier, NONDATA_METRIC_MULTIPLIERS},
	names::multiply_ident,
};

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

struct SingleOutput {
	unit: Ident,
	#[allow(dead_code)]
	arrow_token: Token!(=>),
	conversion: Conversion,
}

impl Parse for SingleOutput {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(Self { unit: input.parse()?, arrow_token: input.parse()?, conversion: input.parse()? })
	}
}

struct MultipleOutputs {
	#[allow(dead_code)]
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
	#[allow(dead_code)]
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
	Metric(#[allow(dead_code)] kw::metric),
	Data(#[allow(dead_code)] kw::data),
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
	#[allow(dead_code)]
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
	vis: Visibility,
	name: Ident,
	#[allow(dead_code)]
	lt_token: Token!(<),
	conv_ty: Path,
	#[allow(dead_code)]
	gt_token: Token!(>),
	origin: Path,
	#[allow(dead_code)]
	brace_token: token::Brace,
	rows: Punctuated<Row, Token!(,)>,
}

impl Parse for Table {
	fn parse(input: ParseStream) -> Result<Self> {
		let content;
		Ok(Self {
			vis: input.parse()?,
			name: input.parse()?,
			lt_token: input.parse()?,
			conv_ty: input.parse()?,
			gt_token: input.parse()?,
			origin: Path::parse_mod_style(input)?,
			// sep_token: input.parse()?,
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

struct ConversionTable2 {
	explicits: Vec<ConcreteConversion2>,
	implicits: Vec<ConcreteConversion2>,
}

impl Table {
	fn conversions(self) -> ConversionTable2 {
		let mut explicits = Vec::new();
		let mut implicits = Vec::new();
		for (explicit, implicit) in self.rows.into_iter().flat_map(Row::into_iter) {
			explicits.extend(explicit);
			implicits.extend(implicit);
		}
		ConversionTable2 { explicits, implicits }
	}
}

pub(crate) fn make_table_impl2(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let table = parse_macro_input!(input as Table);
	let vis = table.vis.clone();
	let name = table.name.clone();
	let origin = table.origin.clone();
	let conv_ty = table.conv_ty.clone();
	let mut tree: HashMap<Ident, HashMap<Ident, Conversion>> = HashMap::new();
	let mut conversions = table.conversions();
	for ConcreteConversion2(input_unit, output_unit, conversion) in
		conversions.implicits.drain(..).chain(conversions.explicits.drain(..))
	{
		tree.entry(input_unit).or_default().insert(output_unit, conversion);
	}
	let mut branches = Vec::new();
	for (input_unit, outputs) in tree {
		let mut leaves = Vec::new();
		for (output_unit, conversion) in outputs {
			leaves.push(quote! { (#origin::#output_unit, (#conversion) as #conv_ty) });
		}
		branches.push(quote! {(#origin::#input_unit, ::std::collections::HashMap::from([#(#leaves),*]))});
	}
	// let expanded = quote! {HashMap::from([#(#branches),*])};
	let expanded = quote! {
		#vis static #name: ::std::sync::LazyLock<::std::collections::HashMap<#origin, ::std::collections::HashMap<#origin, #conv_ty>>> = ::std::sync::LazyLock::new(|| {
			::std::collections::HashMap::from([#(#branches),*])
		});
	};
	// let expanded = quote!{};
	proc_macro::TokenStream::from(expanded)
}
