mod preprocess;

use proc_macro::TokenStream;

#[proc_macro_derive(PreProcess, attributes(preprocess, validate, map))]
pub fn preprocess(input: TokenStream) -> TokenStream {
	preprocess::parse(input)
}
