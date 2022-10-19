use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Error, Field, Result, Type, Visibility};

use super::Preprocessor;

#[derive(Debug, Clone)]
pub struct UnnamedFieldProcessor {
	pub attrs: Vec<Attribute>,
	pub vis: Visibility,
	pub ty: Type,
	pub preprocessors: Vec<Preprocessor>,
}

impl TryFrom<(Field, usize)> for UnnamedFieldProcessor {
	type Error = Error;

	fn try_from((value, index): (Field, usize)) -> Result<Self> {
		let Field {
			attrs,
			vis,
			ident: _,
			colon_token: _,
			ty,
		} = value;

		let preprocessors = attrs
			.iter()
			.cloned()
			.filter(|attr| attr.path.is_ident("preprocess"))
			.map::<Result<_>, _>(|attr| {
				Preprocessor::from_attr(
					format!("field_{}", index),
					ty.to_token_stream().to_string(),
					attr,
				)
			})
			.collect::<Result<Vec<_>>>()?
			.into_iter()
			.flatten()
			.collect::<Vec<_>>();

		Ok(Self {
			attrs,
			vis,
			ty,
			preprocessors,
		})
	}
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
