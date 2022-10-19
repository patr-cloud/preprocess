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
	unnamed_field_processor::UnnamedFieldProcessor,
};

#[derive(Debug, Clone)]
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
				attrs: _,
				vis,
				name,
				generics,
				preprocessors,
				fields,
			} => {
				let (impl_generics, ty_generics, where_clause) =
					generics.split_for_impl();
				let processed_type_name = format_ident!("{}Processed", name);
				let field_definitions = fields.get_field_definitions();
				let field_definitions = match &fields {
					FieldValues::NoFields => quote! {;},
					FieldValues::Named(_) => {
						quote! { { #field_definitions } }
					}
					FieldValues::Unnamed(_) => {
						quote! { ( #field_definitions ) }
					}
				};
				let fields_comma_separated =
					fields.get_fields_comma_separated();
				let fields_comma_separated = match &fields {
					FieldValues::NoFields => quote! {},
					FieldValues::Named(_) => {
						quote! { { #fields_comma_separated } }
					}
					FieldValues::Unnamed(_) => {
						quote! { ( #fields_comma_separated ) }
					}
				};

				let field_processors = match &fields {
					FieldValues::NoFields => vec![quote! {}],
					FieldValues::Named(fields) => fields
						.iter()
						.map(|field| field.get_processor_tokens())
						.collect::<Vec<_>>(),
					FieldValues::Unnamed(fields) => fields
						.iter()
						.enumerate()
						.map(|(index, field)| field.get_processor_tokens(index))
						.collect::<Vec<_>>(),
				};

				let global_preprocessors =
					preprocessors.into_iter().map(|preprocessor| {
						preprocessor.get_processor_token(&format_ident!(
							"parent_struct"
						))
					});

				quote! {
					#vis struct #processed_type_name #ty_generics #where_clause
					#field_definitions

					impl #impl_generics preprocess::PreProcessor for #name #ty_generics #where_clause {
						fn preprocess(self) -> Result<#processed_type_name, preprocess::PreProcessError> {
							let parent_struct = self;

							#(#global_preprocessors) *

							let #name #fields_comma_separated = parent_struct;

							#(#field_processors) *

							Ok(#processed_type_name #fields_comma_separated)
						}
					}
				}
			}
			PreprocessorDerive::Enum {
				attrs: _,
				vis,
				name,
				generics,
				preprocessors,
				variants,
			} => {
				let (impl_generics, ty_generics, where_clause) =
					generics.split_for_impl();
				let processed_type_name = format_ident!("{}Processed", name);

				let variant_definitions =
					variants.clone().into_iter().map(|variant| {
						variant.get_definition(&processed_type_name)
					});

				let global_preprocessors =
					preprocessors.into_iter().map(|preprocessor| {
						preprocessor.get_processor_token(&format_ident!(
							"parent_struct"
						))
					});

				let variant_processors = variants.into_iter().map(
					|EnumVariant {
					     name,
					     preprocessors,
					     fields,
					 }| {
						let fields_comma_separated =
							fields.get_fields_comma_separated();
						let fields_comma_separated = match &fields {
							FieldValues::NoFields => quote! {},
							FieldValues::Named(_) => {
								quote! { { #fields_comma_separated } }
							}
							FieldValues::Unnamed(_) => {
								quote! { ( #fields_comma_separated ) }
							}
						};

						let variant_processors =
							preprocessors.into_iter().map(|preprocessor| {
								preprocessor.get_processor_token(
									&format_ident!("current_variant"),
								)
							});

						let field_processors = match &fields {
							FieldValues::NoFields => vec![quote! {}],
							FieldValues::Named(fields) => fields
								.iter()
								.map(|field| field.get_processor_tokens())
								.collect::<Vec<_>>(),
							FieldValues::Unnamed(fields) => fields
								.iter()
								.enumerate()
								.map(|(index, field)| {
									field.get_processor_tokens(index)
								})
								.collect::<Vec<_>>(),
						};

						quote! {
							Self::#name #fields_comma_separated => {
								let current_variant = Self:: #ty_generics ::#name #fields_comma_separated ;

								#(#variant_processors) *

								let Self:: #ty_generics ::#name #fields_comma_separated = current_variant;

								#(#field_processors) *

								Ok(#processed_type_name :: #ty_generics :: #name #fields_comma_separated)
							}
						}
					},
				);

				quote! {
					#vis enum #processed_type_name #ty_generics #where_clause {
						#(#variant_definitions,) *
					}

					impl #impl_generics preprocess::PreProcessor for #name #ty_generics #where_clause {
						fn preprocess(self) -> Result<#processed_type_name, preprocess::PreProcessError> {
							let parent_struct = self;

							#(#global_preprocessors) *

							match parent_struct {
								#(#variant_processors) *
							}
						}
					}
				}
			}
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
			.filter(|attr| attr.path.is_ident("preprocess"))
			.map::<Result<_>, _>(|attr| {
				Preprocessor::from_attr("".to_string(), ident.to_string(), attr)
			})
			.collect::<Result<Vec<_>>>()?
			.into_iter()
			.flatten()
			.map(|preprocessor| match &preprocessor.r#type {
				PreprocessorType::Custom { .. } |
				PreprocessorType::TypeSpecifier { .. } => Ok(preprocessor),
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
				Fields::Unnamed(unnamed) => Ok(Self::Struct {
					attrs,
					vis,
					name: ident,
					generics,
					preprocessors,
					fields: FieldValues::Unnamed(
						unnamed
							.unnamed
							.into_iter()
							.enumerate()
							.map(|(index, field)| {
								UnnamedFieldProcessor::try_from((field, index))
							})
							.collect::<Result<Vec<_>>>()?,
					),
				}),
				Fields::Unit => Ok(Self::Struct {
					attrs,
					vis,
					name: ident,
					generics,
					preprocessors,
					fields: FieldValues::NoFields,
				}),
			},
			Data::Enum(enum_data) => {
				let variants = enum_data
					.variants
					.into_iter()
					.map(|variant| EnumVariant::try_from(variant))
					.collect::<Result<Vec<_>>>()?;
				Ok(Self::Enum {
					attrs,
					vis,
					name: ident,
					generics,
					preprocessors,
					variants,
				})
			}
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
