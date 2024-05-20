use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use regex::Regex;
use syn::{
	punctuated::Punctuated,
	spanned::Spanned,
	Attribute,
	Error,
	Expr,
	ExprLit,
	Ident,
	Lit,
	LitInt,
	Meta,
	Path,
	Token,
};

use crate::ext_traits::{ExprExt, LitExpr};

#[derive(Debug)]
pub enum IpPreprocessorType {
	V4,
	V6,
	Any,
}

pub enum Preprocessor {
	/// Empty preprocessor
	None,

	/// Complex type handlers
	Optional(Vec<Preprocessor>),

	// Validators
	Email,
	Domain,
	Url,
	Length {
		min: Option<Expr>,
		max: Option<Expr>,
		equal: Option<Expr>,
	},
	Range {
		min: Option<Expr>,
		max: Option<Expr>,
	},
	Contains(String),
	DoesNotContain(String),
	Custom(String),
	Regex(Expr),
	Nested,
	Type(String),
	Ip(IpPreprocessorType),

	// Preprocessors
	Trim,
	Lowercase,
	Uppercase,
	// TODO add later on:
	// KeyValue {
	// 	key: Vec<Preprocessor>,
	// 	value: Vec<Preprocessor>,
	// },
	// If {
	// 	condition: String,
	// 	then: Vec<Preprocessor>,
	// },
	// UUID(type)
}

impl Preprocessor {
	pub fn from_attr(
		attr: &Attribute,
		is_global: bool,
	) -> Result<Vec<Self>, Error> {
		// If the attribute is not `#[preprocess]`, return an error.
		if !attr.path().is_ident("preprocess") {
			return Err(Error::new(
				attr.span(),
				"expected `preprocess` attribute",
			));
		}

		// If the attribute is `#[preprocess]`, but not global, it is shorthand
		// for `#[preprocess(nested)]`.
		if let (Meta::Path(_), false) = (&attr.meta, is_global) {
			return Ok(vec![Preprocessor::Nested]);
		}

		// If the attribute is `#[preprocess(...)]`, parse the inner contents.
		attr.meta
			.require_list()?
			.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
			.into_iter()
			.map(Preprocessor::try_from)
			.collect::<Result<Vec<_>, Error>>()
	}

	pub fn get_new_type(&self, current_type: &TokenStream2) -> TokenStream2 {
		match self {
			Self::None => current_type.clone(),

			Self::Optional(preprocessors) => {
				let inner_type = preprocessors
					.iter()
					.fold(current_type.clone(), |ty, preprocessor| {
						preprocessor.get_new_type(&ty)
					});
				quote! {
					::core::option::Option<#inner_type>
				}
			}

			Self::Email => current_type.clone(),
			Self::Domain => current_type.clone(),
			Self::Url => "::preprocess::types::Url"
				.parse()
				.expect("unable to parse token stream"),
			Self::Length { .. } => current_type.clone(),
			Self::Range { .. } => current_type.clone(),
			Self::Contains(_) => current_type.clone(),
			Self::DoesNotContain(_) => current_type.clone(),
			Self::Custom(_) => current_type.clone(),
			Self::Regex(_) => current_type.clone(),
			Self::Nested => {
				let current_type = current_type.to_string();
				format_ident!("{}Processed", current_type).to_token_stream()
			}
			Self::Type(r#type) => {
				r#type.parse().expect("unable to parse token stream")
			}
			Self::Ip(IpPreprocessorType::V4) => "::std::net::Ipv4Addr"
				.parse()
				.expect("unable to parse token stream"),
			Self::Ip(IpPreprocessorType::V6) => "::std::net::Ipv6Addr"
				.parse()
				.expect("unable to parse token stream"),
			Self::Ip(IpPreprocessorType::Any) => "::std::net::IpAddr"
				.parse()
				.expect("unable to parse token stream"),

			Self::Trim => "::std::borrow::Cow<'static, str>"
				.parse()
				.expect("unable to parse token stream"),
			Self::Lowercase => "::std::borrow::Cow<'static, str>"
				.parse()
				.expect("unable to parse token stream"),
			Self::Uppercase => "::std::borrow::Cow<'static, str>"
				.parse()
				.expect("unable to parse token stream"),
		}
	}

	pub fn as_processor_token_stream(
		&self,
		field_name: &Ident,
		ty: &TokenStream2,
	) -> TokenStream2 {
		let new_ty = self.get_new_type(ty);

		match self {
			Preprocessor::None => quote! {},

			Preprocessor::Optional(preprocessors) => {
				let mut new_type = ty.clone();
				let preprocessors = preprocessors
					.iter()
					.map(|preprocessor| {
						new_type = preprocessor.get_new_type(&new_type);
						preprocessor.as_processor_token_stream(
							&format_ident!("value"),
							&new_type,
						)
					})
					.collect::<TokenStream2>();
				quote! {
					let #field_name: ::core::option::Option<#new_type> = ::core::option::Option::map::<::core::result::Result<#new_type, ::preprocess::Error>, _>(#field_name, |value| {
						#preprocessors
						Ok(value)
					})
					.transpose()
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
				}
			}

			Preprocessor::Email => quote! {
				let #field_name: #new_ty = ::preprocess::validators::validate_email(#field_name)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},
			Preprocessor::Domain => quote! {
				let #field_name: #new_ty = ::preprocess::validators::validate_domain(#field_name)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},
			Preprocessor::Url => quote! {
				let #field_name: #new_ty = ::preprocess::validators::validate_url(#field_name)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},
			Preprocessor::Length { min, max, equal } => {
				let min = min
					.as_ref()
					.map(|min| {
						quote! {
							::std::option::Option::Some(#min)
						}
					})
					.unwrap_or_else(|| {
						quote! {
							::std::option::Option::None
						}
					});
				let max = max
					.as_ref()
					.map(|max| {
						quote! {
							::std::option::Option::Some(#max)
						}
					})
					.unwrap_or_else(|| {
						quote! {
							::std::option::Option::None
						}
					});
				let equal = equal
					.as_ref()
					.map(|equal| {
						quote! {
							::std::option::Option::Some(#equal)
						}
					})
					.unwrap_or_else(|| {
						quote! {
							::std::option::Option::None
						}
					});
				quote! {
					let #field_name: #new_ty = ::preprocess::validators::validate_length(#field_name, #min, #max, #equal)
						.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
				}
			}
			Preprocessor::Range { min, max } => {
				let min = min
					.as_ref()
					.map(|min| {
						quote! {
							::std::option::Option::Some(#min)
						}
					})
					.unwrap_or_else(|| {
						quote! {
							::std::option::Option::None
						}
					});
				let max = max
					.as_ref()
					.map(|max| {
						quote! {
							::std::option::Option::Some(#max)
						}
					})
					.unwrap_or_else(|| {
						quote! {
							::std::option::Option::None
						}
					});

				quote! {
					let #field_name: #new_ty = ::preprocess::validators::validate_range(#field_name, #min, #max)
						.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
				}
			}
			Preprocessor::Contains(look_for) => quote! {
				let #field_name: #new_ty = ::preprocess::validators::validate_contains(#field_name, #look_for)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},
			Preprocessor::DoesNotContain(look_for) => quote! {
				let #field_name: #new_ty = ::preprocess::validators::validate_does_not_contain(#field_name, #look_for)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},
			Preprocessor::Custom(validator) => {
				let validator = format_ident!("{validator}");
				quote! {
					let #field_name: #new_ty = #validator (#field_name)
						.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
				}
			}
			Preprocessor::Regex(regex) => quote! {
				let #field_name: #new_ty = ::preprocess::validators::validate_regex(#field_name, #regex)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},
			Preprocessor::Nested => quote! {
				let #field_name: <#ty as ::preprocess::Preprocessable>::Processed = ::preprocess::Preprocessable::preprocess(#field_name)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},
			Preprocessor::Type(r#type) => {
				let ident = format_ident!("{}", r#type);
				quote! {
					::std::convert::TryInto::<#ident>::try_into
				}
			}

			Preprocessor::Ip(IpPreprocessorType::V4) => quote! {
				let #field_name: #new_ty = ::preprocess::validators::validate_ipv4(#field_name)
					.map_err(|err| err.set_field(::std::stringify(#field_name)))?;
			},
			Preprocessor::Ip(IpPreprocessorType::V6) => quote! {
				let #field_name: #new_ty = ::preprocess::validators::validate_ipv6(#field_name)
					.map_err(|err| err.set_field(::std::stringify(#field_name)))?;
			},
			Preprocessor::Ip(IpPreprocessorType::Any) => quote! {
				let #field_name: #new_ty = ::preprocess::validators::validate_ip(#field_name)
					.map_err(|err| err.set_field(::std::stringify(#field_name)))?;
			},

			Preprocessor::Trim => quote! {
				let #field_name: #new_ty = ::preprocess::preprocessors::preprocess_trim(#field_name)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},

			Preprocessor::Lowercase => quote! {
				let #field_name: #new_ty = ::preprocess::preprocessors::preprocess_lowercase(#field_name)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},
			Preprocessor::Uppercase => quote! {
				let #field_name: #new_ty = ::preprocess::preprocessors::preprocess_uppercase(#field_name)
					.map_err(|err| err.set_field(::std::stringify!(#field_name)))?;
			},
		}
	}
}

impl TryFrom<Meta> for Preprocessor {
	type Error = Error;

	/// By the time it comes to this function, this is what is passed:
	/// #[preprocess(length(min = 1, max = 10))]
	///              ^^^^^^^^^^^^^^^^^^^^^^^^
	/// #[preprocess(email, url, custom = "some-custom-validator")]
	///              ^^^^^  ^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
	/// #[preprocess(regex = "some-regexp")]
	///              ^^^^^^^^^^^^^^^^^^^^^
	fn try_from(value: Meta) -> Result<Self, Self::Error> {
		match value {
			// #[preprocess(optional(...))]
			Meta::List(list) if list.path.is_ident("optional") => {
				let args = list.parse_args_with(
					Punctuated::<Meta, Token![,]>::parse_terminated,
				)?;

				let preprocessors =
					args.into_iter().map(Preprocessor::try_from);

				Ok(Self::Optional(
					preprocessors.collect::<Result<Vec<_>, Error>>()?,
				))
			}
			// #[preprocess(none)]
			Meta::Path(path) if path.is_ident("none") => Ok(Self::None),
			// #[preprocess(email)]
			Meta::Path(path) if path.is_ident("email") => Ok(Self::Email),
			// #[preprocess(domain)]
			Meta::Path(path) if path.is_ident("domain") => Ok(Self::Domain),
			// #[preprocess(url)]
			Meta::Path(path) if path.is_ident("url") => Ok(Self::Url),
			// #[preprocess(nested)]
			Meta::Path(path) if path.is_ident("nested") => Ok(Self::Nested),
			// #[preprocess(trim)]
			Meta::Path(path) if path.is_ident("trim") => Ok(Self::Trim),
			// #[preprocess(lowercase)]
			Meta::Path(path) if path.is_ident("lowercase") => {
				Ok(Self::Lowercase)
			}
			// #[preprocess(uppercase)]
			Meta::Path(path) if path.is_ident("uppercase") => {
				Ok(Self::Uppercase)
			}
			// #[preprocess(length)]
			Meta::Path(path) if path.is_ident("length") => Ok(Self::Length {
				min: Some(Expr::Lit(ExprLit {
					attrs: vec![],
					lit: Lit::Int(LitInt::new("0usize", path.span())),
				})),
				max: None,
				equal: None,
			}),
			// #[preprocess(ip)]
			Meta::Path(path) if path.is_ident("ip") => {
				Ok(Self::Ip(IpPreprocessorType::Any))
			}
			// #[preprocess(length = 10)]
			Meta::NameValue(meta) if meta.path.is_ident("length") => {
				Ok(Self::Length {
					min: None,
					max: None,
					equal: Some(meta.value),
				})
			}
			// #[preprocess(contains = "some-string")]
			Meta::NameValue(meta) if meta.path.is_ident("contains") => {
				Ok(Self::Contains(
					meta.value.require_lit()?.lit.require_str()?.value(),
				))
			}
			// #[preprocess(does_not_contain = "some-string")]
			Meta::NameValue(meta) if meta.path.is_ident("does_not_contain") => {
				Ok(Self::DoesNotContain(
					meta.value.require_lit()?.lit.require_str()?.value(),
				))
			}
			// #[preprocess(custom = "some-string")]
			Meta::NameValue(meta) if meta.path.is_ident("custom") => {
				Ok(Self::Custom(
					meta.value.require_lit()?.lit.require_str()?.value(),
				))
			}
			// #[preprocess(regex = "some-string")]
			Meta::NameValue(meta) if meta.path.is_ident("regex") => {
				if let Ok(Ok(value)) = meta
					.value
					.clone()
					.require_lit()
					.map(|lit| lit.lit.require_str().map(|lit| lit.value()))
				{
					Regex::new(&value).map_err(|err| {
						Error::new(
							value.span(),
							format!("invalid regex: {}", err),
						)
					})?;
				}

				Ok(Self::Regex(meta.value))
			}
			// #[preprocess(type = "String")] or
			// #[preprocess(type = std::string::String)]
			Meta::NameValue(meta) if meta.path.is_ident("type") => {
				let r#type = match &meta.value {
					Expr::Lit(lit) => {
						let Lit::Str(lit_str) = &lit.lit else {
							return Err(Error::new(
								meta.span(),
								"only string literals are allowed here",
							));
						};
						lit_str.value()
					}
					Expr::Path(path) => {
						if let Some(ident) = path.path.get_ident() {
							ident.to_string()
						} else {
							return Err(Error::new(
								meta.span(),
								"expected resulting type",
							));
						}
					}
					_ => {
						return Err(Error::new(
							meta.span(),
							"expected string literal",
						))
					}
				};
				Ok(Self::Type(r#type))
			}
			// #[preprocess(ip(v4))]
			Meta::List(list) if list.path.is_ident("ip") => {
				let args = list.parse_args::<Path>()?;

				if args.is_ident("v4") {
					Ok(Self::Ip(IpPreprocessorType::V4))
				} else if args.is_ident("v6") {
					Ok(Self::Ip(IpPreprocessorType::V6))
				} else {
					Err(Error::new(args.span(), "expected `v4` or `v6`"))
				}
			}
			// #[preprocess(length(min = 1, max = 10))]
			Meta::List(list) if list.path.is_ident("length") => {
				let args = list.parse_args_with(
					Punctuated::<Meta, Token![,]>::parse_terminated,
				)?;

				let (min, max, equal) = args.into_iter().try_fold(
					(None, None, None),
					|(min, max, equal), meta| match meta {
						Meta::NameValue(meta) if meta.path.is_ident("min") => {
							if min.is_some() {
								return Err(Error::new(
									meta.span(),
									"duplicate argument `min`",
								));
							}
							Ok((Some(meta.value), max, equal))
						}
						Meta::NameValue(meta) if meta.path.is_ident("max") => {
							if max.is_some() {
								return Err(Error::new(
									meta.span(),
									"duplicate argument `max`",
								));
							}
							Ok((min, Some(meta.value), equal))
						}
						Meta::NameValue(meta)
							if meta.path.is_ident("equal") =>
						{
							if equal.is_some() {
								return Err(Error::new(
									meta.span(),
									"duplicate argument `equal`",
								));
							}
							Ok((min, max, Some(meta.value)))
						}
						meta => {
							return Err(
								if let Some(ident) = meta.path().get_ident() {
									Error::new(
										meta.span(),
										format!(
											"unexpected argument `{}`",
											ident,
										),
									)
								} else {
									Error::new(
										meta.span(),
										"unexpected argument",
									)
								},
							)
						}
					},
				)?;

				if min.is_none() && max.is_none() && equal.is_none() {
					Err(Error::new(
						list.span(),
						"expected at least one argument `min`, `max` or `equal`",
					))
				} else {
					Ok(Self::Length { min, max, equal })
				}
			}
			// #[preprocess(range(min = 1, max = 10))]
			Meta::List(list) if list.path.is_ident("range") => {
				let args = list.parse_args_with(
					Punctuated::<Meta, Token![,]>::parse_terminated,
				)?;

				let (min, max) = args.into_iter().try_fold(
					(None, None),
					|(min, max), meta| match meta {
						Meta::NameValue(meta) if meta.path.is_ident("min") => {
							if min.is_some() {
								return Err(Error::new(
									meta.span(),
									"duplicate argument `min`",
								));
							}
							Ok((Some(meta.value), max))
						}
						Meta::NameValue(meta) if meta.path.is_ident("max") => {
							if max.is_some() {
								return Err(Error::new(
									meta.span(),
									"duplicate argument `max`",
								));
							}
							Ok((min, Some(meta.value)))
						}
						meta => {
							return Err(
								if let Some(ident) = meta.path().get_ident() {
									Error::new(
										meta.span(),
										format!(
											"unexpected argument `{}`",
											ident,
										),
									)
								} else {
									Error::new(
										meta.span(),
										"unexpected argument",
									)
								},
							)
						}
					},
				)?;

				if min.is_none() && max.is_none() {
					Err(Error::new(
						list.span(),
						"expected at least one argument `min` or `max`",
					))
				} else {
					Ok(Self::Range { min, max })
				}
			}
			_ => Err(Error::new(
				value.span(),
				if let Some(ident) = value.path().get_ident() {
					format!("unexpected preprocessor `{}`", ident)
				} else {
					"unexpected preprocessor".to_string()
				},
			)),
		}
	}
}
