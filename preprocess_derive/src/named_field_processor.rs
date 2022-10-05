use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
	spanned::Spanned,
	Attribute,
	Error,
	Field,
	Ident,
	Result,
	Type,
	Visibility,
};

use super::Preprocessor;

#[derive(Debug)]
pub struct NamedFieldProcessor {
	pub attrs: Vec<Attribute>,
	pub vis: Visibility,
	pub name: Ident,
	pub ty: Type,
	pub preprocessors: Vec<Preprocessor>,
}

impl TryFrom<Field> for NamedFieldProcessor {
	type Error = Error;

	fn try_from(value: Field) -> Result<Self> {
		let Field {
			attrs,
			vis,
			ident,
			colon_token: _,
			ty,
		} = value;

		let ident = if let Some(ident) = ident {
			ident
		} else {
			return Err(Error::new(
				ty.span(),
				concat!(
					"found an unnamed field while parsing a named field",
					". Ideally, this shouldn't have happened.",
					" Would appreciate a bug report to the `preprocess` crate"
				),
			));
		};

		let preprocessors = attrs
			.iter()
			.cloned()
			.filter(|attr| attr.path.is_ident("preprocess"))
			.map::<Result<_>, _>(|attr| {
				Preprocessor::from_attr(
					ident.to_string(),
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
			name: ident,
			ty,
			preprocessors,
		})
	}
}

impl NamedFieldProcessor {
	pub fn get_processor_tokens(&self) -> TokenStream {
		let processors = self
			.preprocessors
			.iter()
			.map(|processor| processor.get_processor_token(&self.name));

		quote! {
			#(#processors) *
		}
	}
}
