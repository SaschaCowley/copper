mod kw;
mod multipliers;
mod names;
mod table;
mod units;

use table::make_table_impl;
use units::declare_units_impl;

#[proc_macro]
pub fn make_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	make_table_impl(input)
}

#[proc_macro]
pub fn declare_units(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	declare_units_impl(input)
}
