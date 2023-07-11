use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
	token::{Brace, Paren},
	Error,
	Field,
	Fields,
	FieldsNamed,
	FieldsUnnamed,
};

use crate::preprocessor::Preprocessor;

pub enum ProcessedFields {
	Unit,
	Named(ProcessedNamed),
	Unnamed(ProcessedUnnamed),
}

impl TryInto<ProcessedFields> for Fields {
	type Error = Error;

	fn try_into(self) -> Result<ProcessedFields, Self::Error> {
		match self {
			Fields::Named(FieldsNamed { brace_token, named }) => {
				Ok(ProcessedFields::Named(ProcessedNamed {
					named: named
						.into_iter()
						.map(|field| {
							let Field {
								attrs,
								vis,
								mutability,
								ident,
								colon_token,
								ty,
							} = field;

							let preprocessors = attrs
								.iter()
								.filter(|attr| {
									attr.path().is_ident("preprocess")
								})
								.map(|attr| {
									Preprocessor::from_attr(attr, false)
								})
								.collect::<Result<Vec<_>, Error>>()?
								.into_iter()
								.flatten()
								.collect::<Vec<_>>();

							Ok((
								Field {
									attrs: attrs
										.into_iter()
										.filter(|attr| {
											!attr.path().is_ident("preprocess")
										})
										.collect(),
									vis,
									mutability,
									ident,
									colon_token,
									ty,
								},
								preprocessors,
							))
						})
						.collect::<Result<Vec<_>, Error>>()?,
					brace_token,
				}))
			}
			Fields::Unnamed(FieldsUnnamed {
				paren_token,
				unnamed,
			}) => Ok(ProcessedFields::Unnamed(ProcessedUnnamed {
				unnamed: unnamed
					.into_iter()
					.map(|field| {
						let Field {
							attrs,
							vis,
							mutability,
							ident,
							colon_token,
							ty,
						} = field;

						let preprocessors = attrs
							.iter()
							.filter(|attr| attr.path().is_ident("preprocess"))
							.map(|attr| Preprocessor::from_attr(attr, false))
							.collect::<Result<Vec<_>, Error>>()?
							.into_iter()
							.flatten()
							.collect::<Vec<_>>();

						Ok((
							Field {
								attrs: attrs
									.into_iter()
									.filter(|attr| {
										!attr.path().is_ident("preprocess")
									})
									.collect(),
								vis,
								mutability,
								ident,
								colon_token,
								ty,
							},
							preprocessors,
						))
					})
					.collect::<Result<Vec<_>, Error>>()?,
				paren_token,
			})),
			Fields::Unit => Ok(ProcessedFields::Unit),
		}
	}
}

impl ToTokens for ProcessedFields {
	fn to_tokens(&self, tokens: &mut TokenStream2) {
		match &self {
			ProcessedFields::Unit => Fields::Unit.to_tokens(tokens),
			ProcessedFields::Named(ProcessedNamed { named, brace_token }) => {
				Fields::Named(FieldsNamed {
					named: named
						.iter()
						.map(|(field, _)| field.clone())
						.collect(),
					brace_token: brace_token.clone(),
				})
				.to_tokens(tokens)
			}
			ProcessedFields::Unnamed(ProcessedUnnamed {
				unnamed,
				paren_token,
			}) => Fields::Unnamed(FieldsUnnamed {
				unnamed: unnamed
					.iter()
					.map(|(field, _)| field.clone())
					.collect(),
				paren_token: paren_token.clone(),
			})
			.to_tokens(tokens),
		}
	}
}

pub struct ProcessedNamed {
	pub named: Vec<(Field, Vec<Preprocessor>)>,
	pub brace_token: Brace,
}

pub struct ProcessedUnnamed {
	pub unnamed: Vec<(Field, Vec<Preprocessor>)>,
	pub paren_token: Paren,
}
