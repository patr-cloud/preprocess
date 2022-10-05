use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Type, Visibility};

use super::Preprocessor;

#[derive(Debug)]
pub struct UnnamedFieldProcessor {
	pub attrs: Vec<Attribute>,
	pub vis: Visibility,
	pub ty: Type,
	pub preprocessors: Vec<Preprocessor>,
}

impl UnnamedFieldProcessor {
	pub fn get_processor_tokens(&self, index: usize) -> TokenStream {
		let processors = self.preprocessors.iter().map(|processor| {
			processor.get_processor_token(&format_ident!("field_{}", index))
		});

		quote! {
			#(#processors) *
		}
	}
}
