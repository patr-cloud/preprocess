use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Error, Fields, Ident, Result, Variant};

use super::{FieldValues, Preprocessor};
use crate::{
	named_field_processor::NamedFieldProcessor,
	preprocessor::PreprocessorType,
	unnamed_field_processor::UnnamedFieldProcessor,
};

#[derive(Debug, Clone)]
pub struct EnumVariant {
	pub name: Ident,
	pub preprocessors: Vec<Preprocessor>,
	pub fields: FieldValues,
}

impl TryFrom<Variant> for EnumVariant {
	type Error = Error;

	fn try_from(value: Variant) -> Result<Self> {
		let name = value.ident;

		let preprocessors = value
			.attrs
			.into_iter()
			.filter(|attr| attr.path.is_ident("preprocess"))
			.map::<Result<_>, _>(|attr| {
				Preprocessor::from_attr("".to_string(), name.to_string(), attr)
			})
			.collect::<Result<Vec<_>>>()?
			.into_iter()
			.flatten()
			.map(|preprocessor| match &preprocessor.r#type {
				PreprocessorType::Custom { .. } => Ok(preprocessor),
				_ => Err(Error::new(
					preprocessor.resultant_type.span(),
					format!(
						concat!(
							"`{}` preprocessor cannot be used on the type level",
							". Did you mean to use it for a specific field?"
						),
						preprocessor.r#type.preprocessor_name()
					),
				)),
			})
			.collect::<Result<Vec<_>>>()?;

		let fields = match value.fields {
			Fields::Named(named) => FieldValues::Named(
				named
					.named
					.into_iter()
					.map(|field| NamedFieldProcessor::try_from(field))
					.collect::<Result<Vec<_>>>()?,
			),
			Fields::Unnamed(unnamed) => FieldValues::Unnamed(
				unnamed
					.unnamed
					.into_iter()
					.enumerate()
					.map(|(index, field)| {
						UnnamedFieldProcessor::try_from((field, index))
					})
					.collect::<Result<Vec<_>>>()?,
			),
			Fields::Unit => FieldValues::NoFields,
		};

		Ok(EnumVariant {
			name,
			preprocessors,
			fields,
		})
	}
}

impl EnumVariant {
	pub fn get_definition(&self, processed_type_name: &Ident) -> TokenStream {
		let name = &self.name;
		let field_definitions = self.fields.get_field_definitions();
		let field_definitions = match &self.fields {
			FieldValues::NoFields => quote! {},
			FieldValues::Named(_) => {
				quote! { { #field_definitions } }
			}
			FieldValues::Unnamed(_) => {
				quote! { ( #field_definitions ) }
			}
		};

		quote! {
			#processed_type_name :: #name #field_definitions
		}
	}
}
