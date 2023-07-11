use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
	token::Brace,
	Attribute,
	Error,
	Field,
	Fields,
	FieldsNamed,
	FieldsUnnamed,
	Generics,
	Ident,
	ItemEnum,
	Token,
	Type,
	Variant,
	Visibility,
};

use crate::{
	preprocessor::Preprocessor,
	processed_fields::{ProcessedFields, ProcessedNamed, ProcessedUnnamed},
};

pub struct ParsedEnum {
	attrs: Vec<Attribute>,
	vis: Visibility,
	enum_token: Token![enum],
	ident: Ident,
	generics: Generics,
	#[allow(dead_code)]
	brace_token: Brace,
	variants: Vec<ProcessedVariant>,
	global: Vec<Preprocessor>,
}

pub struct ProcessedVariant {
	attrs: Vec<Attribute>,
	ident: Ident,
	fields: ProcessedFields,
}

impl ToTokens for ProcessedVariant {
	fn to_tokens(&self, tokens: &mut TokenStream2) {
		let attrs = &self.attrs;
		let ident = &self.ident.clone();
		let fields = self.fields.to_token_stream();

		tokens.extend(quote! {
			#(#attrs) *
			#ident #fields
		});
	}
}

impl TryFrom<ItemEnum> for ParsedEnum {
	type Error = Error;

	fn try_from(item: ItemEnum) -> Result<Self, Self::Error> {
		let ItemEnum {
			attrs,
			vis,
			enum_token,
			ident,
			generics,
			brace_token,
			variants,
		} = item;

		let variants = variants
			.into_iter()
			.map(|variant| {
				let Variant {
					attrs,
					ident,
					fields,
					discriminant,
				} = variant;

				if let Some(discriminant) = discriminant {
					return Err(Error::new_spanned(
						&discriminant.1,
						"Preprocess does not support discriminants.",
					));
				}

				// For now, no preprocessors are allowed on variants.

				Ok(ProcessedVariant {
					attrs,
					ident,
					fields: fields.try_into()?,
				})
			})
			.collect::<Result<_, Error>>()?;

		let global = attrs
			.iter()
			.filter(|attr| attr.path().is_ident("preprocess"))
			.map(|attr| Preprocessor::from_attr(attr, true))
			.collect::<Result<Vec<_>, Error>>()?
			.into_iter()
			.flatten()
			.collect::<Vec<_>>();

		Ok(Self {
			attrs: attrs
				.into_iter()
				.filter(|attr| !attr.path().is_ident("preprocess"))
				.collect(),
			vis,
			enum_token,
			ident,
			generics,
			brace_token,
			variants,
			global,
		})
	}
}

pub fn into_processed(item: ItemEnum) -> Result<TokenStream, Error> {
	let parsed: ParsedEnum = item.try_into()?;

	let ParsedEnum {
		attrs,
		vis,
		enum_token,
		ident,
		generics,
		brace_token: _,
		variants,
		global,
	} = parsed;

	let processed_ident = format_ident!("{}Processed", ident);

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let new_variants = variants
		.iter()
		.map(|variant| {
			let fields = match &variant.fields {
				ProcessedFields::Unit => Fields::Unit,
				ProcessedFields::Named(ProcessedNamed {
					named,
					brace_token,
				}) => Fields::Named(FieldsNamed {
					brace_token: brace_token.clone(),
					named: named
						.iter()
						.map(|(field, preprocessors)| {
							let new_type = preprocessors
								.iter()
								.fold(
									field.ty.to_token_stream(),
									|acc, preprocessor| {
										preprocessor.get_new_type(&acc)
									},
								)
								.to_string();

							let ty: Type = syn::parse_str(&new_type)?;
							Ok(Field {
								attrs: field.attrs.clone(),
								vis: field.vis.clone(),
								mutability: field.mutability.clone(),
								ident: field.ident.clone(),
								colon_token: field.colon_token.clone(),
								ty,
							})
						})
						.collect::<Result<_, Error>>()?,
				}),
				ProcessedFields::Unnamed(ProcessedUnnamed {
					unnamed,
					paren_token,
				}) => Fields::Unnamed(FieldsUnnamed {
					paren_token: paren_token.clone(),
					unnamed: unnamed
						.iter()
						.map(|(field, preprocessors)| {
							let new_type = preprocessors
								.iter()
								.fold(
									field.ty.to_token_stream(),
									|acc, preprocessor| {
										preprocessor.get_new_type(&acc)
									},
								)
								.to_string();

							let ty: Type = syn::parse_str(&new_type)?;
							Ok(Field {
								attrs: field.attrs.clone(),
								vis: field.vis.clone(),
								mutability: field.mutability.clone(),
								ident: field.ident.clone(),
								colon_token: field.colon_token.clone(),
								ty,
							})
						})
						.collect::<Result<_, Error>>()?,
				}),
			};
			Ok(Variant {
				attrs: variant.attrs.clone(),
				ident: variant.ident.clone(),
				fields,
				discriminant: None,
			})
		})
		.collect::<Result<Vec<_>, Error>>()?;

	let global_preprocessors = global.into_iter().map(|preprocessor| {
		preprocessor.into_processor_token_stream(
			&format_ident!("value"),
			&ident.to_token_stream(),
		)
	});

	let variants_destructed = variants.iter().map(|variant| {
		let ProcessedVariant {
			attrs,
			ident,
			fields,
		} = variant;

		let field_names_destructured = match &fields {
			ProcessedFields::Unit => TokenStream2::new(),
			ProcessedFields::Named(ProcessedNamed { named, .. }) => {
				let named =
					named.iter().map(|(field, _)| field.ident.clone().unwrap());
				quote! {
					{
						#(#named),*
					}
				}
			}
			ProcessedFields::Unnamed(ProcessedUnnamed { unnamed, .. }) => {
				let unnamed = unnamed
					.iter()
					.enumerate()
					.map(|(index, _)| format_ident!("field_{}", index));
				quote! {
					(
						#(#unnamed),*
					)
				}
			}
		};

		let field_preprocessors = match &fields {
			ProcessedFields::Unit => vec![],
			ProcessedFields::Named(ProcessedNamed { named, .. }) => named
				.iter()
				.map(|(field, preprocessors)| {
					preprocessors
						.iter()
						.map(|preprocessor| {
							preprocessor.into_processor_token_stream(
								field.ident.as_ref().unwrap(),
								&field.ty.to_token_stream(),
							)
						})
						.collect::<Vec<_>>()
				})
				.flatten()
				.collect(),
			ProcessedFields::Unnamed(ProcessedUnnamed { unnamed, .. }) => {
				unnamed
					.iter()
					.map(|(field, preprocessors)| {
						preprocessors
							.iter()
							.enumerate()
							.map(|(index, preprocessor)| {
								preprocessor.into_processor_token_stream(
									&format_ident!("field_{}", index),
									&field.ty.to_token_stream(),
								)
							})
							.collect::<Vec<_>>()
					})
					.flatten()
					.collect()
			}
		};

		quote! {
			#(#attrs) *
			Self:: #ident #field_names_destructured => {
				#(#field_preprocessors
				)*

				Ok(#processed_ident :: #ident
					#field_names_destructured
				)
			}
		}
	});

	Ok(quote! {
		#(#attrs)*
		#vis #enum_token #ident #generics {
			#(#variants,)*
		}

		#(#attrs)*
		#vis struct #processed_ident #generics {
			#(#new_variants,)*
		}

		impl #impl_generics #ident #ty_generics #where_clause {
			pub fn preprocess(self) -> ::std::result::Result<#processed_ident #ty_generics, ::preprocess::Error> {
				let value = self;

				#(#global_preprocessors
				)*

				match value {
					#(#variants_destructed) *
				}
			}
		}
	}
	.into())
}
