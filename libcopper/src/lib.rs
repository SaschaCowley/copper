use proc_macro2::{TokenStream, TokenTree};
use syn::{
	parse::{Parse, ParseStream, Result},
	parse_macro_input,
	Token,
	token,
	punctuated::Punctuated,
	braced,
};
use quote::quote;
use std::fmt;

mod punc {
	syn::custom_punctuation!(ThinArrow, ->);
}

mod kw {
	syn::custom_keyword!(mul);
	syn::custom_keyword!(div);
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
	// Div(?),
	// Add(?),
	// Sub(?),
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

enum OutputType {
	Single(ConversionRHS),
	Multiple(ConversionRHSs),
}

impl Parse for OutputType {
	fn parse(input: ParseStream) -> Result<Self> {
		Ok(if !input.peek(token::Brace) {
			Self::Single(input.parse()?)
		} else {
			Self::Multiple(input.parse()?)
		})
	}
}

struct ConversionRHS {
	output_unit: TokenStream,
	arrow_token: Token!(=>),
	conversion: Conversion,
}

impl Parse for ConversionRHS {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut output_unit = TokenStream::new();
		while !input.is_empty() && !input.peek(Token!(=>)) {
			output_unit.extend(input.parse::<TokenTree>());
		}
		let arrow_token = input.parse::<Token!(=>)>()?;
		let  conversion = input.parse::<Conversion>()?;
		// let mut conversion = TokenStream::new();
		// while !input.is_empty() && !input.peek(Token!(,)) {
			// conversion.extend(input.parse::<TokenTree>());
		// }
		println!("Converting to: {}\nConversion method: {:?}", output_unit, conversion);
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
	input_unit: TokenStream,
	arrow_token: punc::ThinArrow,
	outputs: OutputType,
}

impl Parse for ConversionRow {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut input_unit = TokenStream::new();
		while !input.is_empty() && !input.peek(punc::ThinArrow) {
			input_unit.extend(input.parse::<TokenTree>());
		}
		let arrow_token = input.parse::<punc::ThinArrow>()?;
		println!("Converting from: {}", input_unit);
		let outputs = input.parse::<OutputType>()?;
		Ok(Self {input_unit, arrow_token, outputs})
	}
}

struct ConversionTable {
	
}

impl Parse for ConversionTable {
	fn parse(input: ParseStream) -> Result<Self> {
		let _rows = Punctuated::<ConversionRow, Token!(,)>::parse_terminated(input);
		
		Ok(Self {})
	}
}

#[proc_macro]
pub fn make_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let table = parse_macro_input!(input as ConversionTable);
	let expanded = quote! {};
	proc_macro::TokenStream::from(expanded)
}