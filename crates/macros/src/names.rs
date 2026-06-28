use syn::Ident;
pub(crate) fn name_to_ident(name: &str) -> String {
	let mut chars = name.chars();
	chars.next().map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

pub(crate) fn multiply_ident(base: &Ident, multiplier: &str) -> Ident {
	Ident::new(&name_to_ident(&(multiplier.to_owned() + &base.to_string().to_lowercase())), base.span())
}
