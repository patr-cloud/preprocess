use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
	spanned::Spanned,
	Attribute,
	Data,
	DeriveInput,
	Error,
	Fields,
	Generics,
	Ident,
	Result,
	Visibility,
};

use super::{EnumVariant, FieldValues, Preprocessor};
use crate::{
	named_field_processor::NamedFieldProcessor,
	preprocessor::PreprocessorType,
};

#[derive(Debug)]
pub enum PreprocessorDerive {
	Struct {
		attrs: Vec<Attribute>,
		vis: Visibility,
		name: Ident,
		generics: Generics,
		preprocessors: Vec<Preprocessor>,
		fields: FieldValues,
	},
	Enum {
		attrs: Vec<Attribute>,
		vis: Visibility,
		name: Ident,
		generics: Generics,
		preprocessors: Vec<Preprocessor>,
		variants: Vec<EnumVariant>,
	},
}

impl PreprocessorDerive {
	pub fn preprocess_tokens(self) -> TokenStream2 {
		match self {
			PreprocessorDerive::Struct {
				attrs,
				vis,
				name,
				generics,
				preprocessors,
				fields,
			} => {
				let (impl_generics, ty_generics, where_clause) =
					generics.split_for_impl();
				let processed_type_name = format_ident!("{}Processed", name);
				let end = if let FieldValues::NoFields = &fields {
					quote! { ; }
				} else {
					quote! { }
				};
				quote! {
					#vis struct #processed_type_name #ty_generics #where_clause
					#end
				}
			}
			PreprocessorDerive::Enum {
				attrs,
				vis,
				name,
				generics,
				preprocessors,
				variants,
			} => todo!(),
		}
	}
}

impl TryFrom<TokenStream> for PreprocessorDerive {
	type Error = Error;

	fn try_from(value: TokenStream) -> Result<Self> {
		let DeriveInput {
			attrs,
			vis,
			ident,
			generics,
			data,
		} = syn::parse::<DeriveInput>(value)?;

		let preprocessors = attrs
			.iter()
			.cloned()
			.map::<Result<_>, _>(|attr| {
				Preprocessor::from_attr("".to_string(), ident.to_string(), attr)
			})
			.collect::<Result<Vec<_>>>()?
			.into_iter()
			.flatten()
			.map(|preprocessor| match &preprocessor.r#type {
				PreprocessorType::Custom { function_name: _ } => {
					Ok(preprocessor)
				}
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

		match data {
			Data::Struct(struct_data) => match struct_data.fields {
				Fields::Named(named) => Ok(Self::Struct {
					attrs,
					vis,
					name: ident,
					generics,
					preprocessors,
					fields: FieldValues::Named(
						named
							.named
							.into_iter()
							.map(|field| NamedFieldProcessor::try_from(field))
							.collect::<Result<Vec<_>>>()?,
					),
				}),
				Fields::Unnamed(_) => todo!(),
				Fields::Unit => Ok(Self::Struct {
					attrs,
					vis,
					name: ident,
					generics,
					preprocessors,
					fields: FieldValues::NoFields,
				}),
			},
			Data::Enum(enum_data) => todo!(),
			Data::Union(union_data) => {
				return Err(Error::new(
					union_data.union_token.span(),
					concat!(
						"unions are not currently supported.",
						"Try using an enum or a struct instead"
					),
				));
			}
		}
	}
}
