pub(crate) mod punc {
	syn::custom_punctuation!(ThinArrow, ->);
}

pub(crate) mod kw {
	syn::custom_keyword!(mul);
	syn::custom_keyword!(div);
	syn::custom_keyword!(add);
	syn::custom_keyword!(sub);
	syn::custom_keyword!(fun);
	syn::custom_keyword!(metric);
	syn::custom_keyword!(metric_pos);
}
