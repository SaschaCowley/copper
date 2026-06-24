mod table;

use table::make_table_impl;

#[proc_macro]
pub fn make_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	make_table_impl(input)
}
