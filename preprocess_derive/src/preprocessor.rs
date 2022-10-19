use std::collections::HashMap;

use preprocess_types::validators::LengthValidatorArgs;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;
use syn::{
	spanned::Spanned,
	Attribute,
	Error,
	Ident,
	Lit,
	Meta,
	NestedMeta,
	Result,
};

#[derive(Debug, Clone)]
pub struct Preprocessor {
	pub r#type: PreprocessorType,
	pub log_value: bool,
	pub field_name: String,
	pub resultant_type: String,
}

impl Preprocessor {
	pub fn from_attr(
		field_name: String,
		input_type: String,
		value: Attribute,
	) -> Result<Vec<Self>> {
		let meta = value.parse_meta()?;
		match meta {
			// In case there's a #[preprocess] attribute, just create a nested
			// preprocessor
			Meta::Path(_) => {
				let preprocessor_type = PreprocessorType::Nested;
				let resultant_type =
					preprocessor_type.get_processed_type(&input_type);
				Ok(vec![Preprocessor {
					r#type: preprocessor_type,
					log_value: true,
					field_name,
					resultant_type,
				}])
			}
			// If there's a #[preprocess(email, length, etc)] attribute,
			// parse each one of them as a preprocessor
			Meta::List(list) => {
				let mut input_type = input_type;
				let mut preprocessors = vec![];

				for meta in list.nested.into_iter() {
					let meta = match meta {
						NestedMeta::Meta(meta) => meta,
						NestedMeta::Lit(lit) => {
							return Err(Error::new(
								lit.span(),
								"found literal, expected preprocessor",
							))
						}
					};
					let preprocessor = Preprocessor::from_meta(
						field_name.clone(),
						input_type,
						meta,
					)?;
					input_type = preprocessor.resultant_type.clone();
					preprocessors.push(preprocessor);
				}

				Ok(preprocessors)
			}
			Meta::NameValue(name_value) => {
				return Err(Error::new(
					name_value.span(),
					concat!(
						"invalid assignment on `preprocess` attribute. ",
						"Did you mean to use `#[preprocess]` instead?"
					),
				))
			}
		}
	}

	pub fn from_meta(
		field_name: String,
		input_type: String,
		value: Meta,
	) -> Result<Self> {
		match value {
			// If there's just `email` as a preprocessor
			Meta::Path(path) => {
				let preprocessor = path
					.get_ident()
					.ok_or_else(|| {
						Error::new(path.span(), "unknown preprocessor found")
					})?
					.to_string();
				let r#type = match preprocessor.as_str() {
					"nested" => PreprocessorType::Nested,
					"email" => PreprocessorType::Email,
					"ip" => PreprocessorType::Ip,
					"ipv4" => PreprocessorType::Ipv4,
					"ipv6" => PreprocessorType::Ipv6,
					"length" => PreprocessorType::Length(Default::default()),
					"url" => PreprocessorType::Url,
					"host" => PreprocessorType::Host,
					"domain" => PreprocessorType::Domain,
					"required" => PreprocessorType::Required,
					"trimmed" => PreprocessorType::Trimmed,
					"lowercase" => PreprocessorType::Lowercase,
					"uppercase" => PreprocessorType::Uppercase,

					preprocessor @ ("custom" | "type" | "contains" |
					"doesnotcontain" | "regex") => {
						return Err(Error::new(
							path.span(),
							format!(
								concat!(
									"preprocessor `{}` cannot be used",
									" without a value parameter"
								),
								preprocessor
							),
						));
					}
					preprocessor => {
						return Err(Error::new(
							path.span(),
							format!("unknown preprocessor `{}`", preprocessor),
						));
					}
				};
				let resultant_type = r#type.get_processed_type(&input_type);

				Ok(Preprocessor {
					r#type,
					log_value: true,
					field_name,
					resultant_type,
				})
			}
			// If there's `email(with some parameters)` as a preprocessor
			Meta::List(meta_list) => {
				let mut args = meta_list
					.nested
					.into_iter()
					.map(|item| match item {
						NestedMeta::Meta(Meta::Path(path)) => {
							let ident = path
								.get_ident()
								.ok_or_else(|| {
									Error::new(
										path.span(),
										"unknown preprocessor found",
									)
								})?
								.to_string();

							Ok((ident, "true".to_string()))
						}
						NestedMeta::Meta(Meta::NameValue(name_value)) => {
							let key = name_value
								.path
								.get_ident()
								.ok_or_else(|| {
									Error::new(
										name_value.path.span(),
										"unknown preprocessor found",
									)
								})?
								.to_string();
							let value = match name_value.lit {
								Lit::Str(litstr) => litstr.value(),
								Lit::Bool(litbool) => {
									litbool.value().to_string()
								}
								Lit::Int(litint) => litint.to_string(),
								Lit::Float(litfloat) => litfloat.to_string(),
								lit => {
									return Err(Error::new(
										lit.span(),
										concat!(
											"unknown literal found. ",
											"Expected numbers, bools and string"
										),
									));
								}
							};

							Ok((key, value))
						}
						NestedMeta::Meta(Meta::List(list)) => {
							return Err(Error::new(
								list.span(),
								concat!(
									"preprocessor arguments must be in",
									"`key = \"value\"` format"
								),
							));
						}
						NestedMeta::Lit(lit) => {
							return Err(Error::new(
								lit.span(),
								concat!(
									"preprocessor arguments must be in",
									"`key = \"value\"` format"
								),
							));
						}
					})
					.collect::<Result<HashMap<_, _>>>()?;

				let sensitive = args
					.remove("sensitive")
					.map(|value| {
						value.parse::<bool>().map_err(|_| {
							Error::new(
								meta_list.path.span(),
								format!(
									"unable to parse `{}` as a bool",
									value
								),
							)
						})
					})
					.transpose()?
					.unwrap_or(false);
				let log_value = !sensitive;

				let field_name = args.remove("rename").unwrap_or(field_name);

				let name = meta_list
					.path
					.get_ident()
					.ok_or_else(|| {
						Error::new(
							meta_list.path.span(),
							"unknown preprocessor found",
						)
					})?
					.to_string();

				let r#type =
					match PreprocessorType::parse_preprocessor_name(&name)? {
						PreprocessorType::Custom { .. } => {
							let function_name = args
								.remove("fn")
								.or(args.remove("function"))
								.ok_or_else(|| {
									Error::new(
										meta_list.path.span(),
										concat!(
											"custom functions require",
											" a function name"
										),
									)
								})?;
							PreprocessorType::Custom { function_name }
						}
						PreprocessorType::TypeSpecifier { .. } => {
							// #[preprocessor(type(some parameter))] doesn't
							// make sense
							return Err(Error::new(
								meta_list.path.span(),
								concat!(
									"type specifier must be a key-value",
									" of the form type = \"TypeName\""
								),
							));
						}
						// TODO allow abitrary strings that reference variables
						PreprocessorType::Length(_) => {
							let length_validator = match (
								args.remove("min")
									.map(|value| value.parse::<usize>().ok())
									.flatten(),
								args.remove("max")
									.map(|value| value.parse::<usize>().ok())
									.flatten(),
								args.remove("exact")
									.map(|value| value.parse::<usize>().ok())
									.flatten(),
							) {
								(None, None, None) => {
									LengthValidatorArgs::default()
								}
								(Some(min), None, None) => {
									LengthValidatorArgs::Min { min }
								}
								(None, Some(max), None) => {
									LengthValidatorArgs::Max { max }
								}
								(Some(min), Some(max), None) => {
									LengthValidatorArgs::MinMax { min, max }
								}
								(None, None, Some(exact)) => {
									LengthValidatorArgs::Exact { exact }
								}
								(Some(_), None, Some(_)) |
								(None, Some(_), Some(_)) |
								(Some(_), Some(_), Some(_)) => {
									return Err(Error::new(
										meta_list.path.span(),
										"exact cannot be used with min and max",
									));
								}
							};

							PreprocessorType::Length(length_validator)
						}
						PreprocessorType::Contains { .. } => {
							let value =
								args.remove("value").ok_or_else(|| {
									Error::new(
										meta_list.path.span(),
										concat!(
											"contains preprocessor",
											" requires a value"
										),
									)
								})?;
							PreprocessorType::Contains { value }
						}
						PreprocessorType::DoesNotContain { .. } => {
							let value =
								args.remove("value").ok_or_else(|| {
									Error::new(
										meta_list.path.span(),
										concat!(
											"does_not_contain preprocessor",
											" requires a value"
										),
									)
								})?;
							PreprocessorType::DoesNotContain { value }
						}
						PreprocessorType::Regex { .. } => {
							let regex =
								args.remove("regex").ok_or_else(|| {
									Error::new(
										meta_list.path.span(),
										concat!(
											"regex preprocessor",
											" requires a regex"
										),
									)
								})?;
							PreprocessorType::Regex { regex }
						}
						_ => {
							return Err(Error::new(
								meta_list.path.span(),
								format!(
									concat!(
										"preprocessor `{}` does not accept",
										" any arguments"
									),
									name
								),
							))
						}
					};

				let resultant_type = args
					.remove("type")
					.unwrap_or(r#type.get_processed_type(&input_type));

				if let Some(key) = args.keys().into_iter().next() {
					return Err(Error::new(
						meta_list.path.span(),
						format!(
							"unknown argument to `{}` preprocessor `{}`",
							name, key
						),
					));
				}

				Ok(Preprocessor {
					r#type,
					log_value,
					field_name,
					resultant_type,
				})
			}
			// If there's `email = "value"` as a preprocessor
			Meta::NameValue(name_value) => {
				let path = name_value
					.path
					.get_ident()
					.ok_or_else(|| {
						Error::new(
							name_value.path.span(),
							"unknown preprocessor found",
						)
					})?
					.to_string();
				match path.as_str() {
					"custom" => {
						let function_name = match name_value.lit {
							Lit::Str(litstr) => litstr.value(),
							_ => {
								return Err(Error::new(
									name_value.span(),
									concat!(
										"only function names (as strings) ",
										"are allowed in custom preprocessors"
									),
								));
							}
						};
						Ok(Preprocessor {
							r#type: PreprocessorType::Custom { function_name },
							log_value: true,
							field_name,
							resultant_type: input_type,
						})
					}
					"type" => {
						let type_name = match name_value.lit {
							Lit::Str(litstr) => litstr.value(),
							_ => {
								return Err(Error::new(
									name_value.span(),
									concat!(
										"only types (as strings) ",
										"are allowed in the type specifier"
									),
								));
							}
						};
						Ok(Preprocessor {
							r#type: PreprocessorType::TypeSpecifier {
								type_name,
							},
							log_value: true,
							field_name,
							resultant_type: input_type,
						})
					}
					"length" => {
						let value = match &name_value.lit {
							Lit::Int(litint) => litint
								.base10_parse::<usize>()
								.map_err(|err| {
									Error::new(
										name_value.lit.span(),
										format!(
											concat!(
												"could not parse number",
												" as integer: {}"
											),
											err.to_string()
										),
									)
								})?,
							_ => {
								return Err(Error::new(
									name_value.span(),
									concat!(
										"only numbers are allowed ",
										"in for the length preprocessor"
									),
								));
							}
						};
						Ok(Preprocessor {
							r#type: PreprocessorType::Length(
								LengthValidatorArgs::Exact { exact: value },
							),
							log_value: true,
							field_name,
							resultant_type: input_type,
						})
					}
					"contains" => {
						let value = match name_value.lit {
							Lit::Str(litstr) => litstr.value(),
							_ => {
								return Err(Error::new(
									name_value.span(),
									concat!(
										"only values (as strings) are allowed",
										" in the contains preprocessor"
									),
								));
							}
						};
						Ok(Preprocessor {
							r#type: PreprocessorType::Contains { value },
							log_value: true,
							field_name,
							resultant_type: input_type,
						})
					}
					"does_not_contain" => {
						let value = match name_value.lit {
							Lit::Str(litstr) => litstr.value(),
							_ => {
								return Err(Error::new(
									name_value.span(),
									concat!(
										"only values (as strings) are allowed",
										" in the does_not_contain preprocessor"
									),
								));
							}
						};
						Ok(Preprocessor {
							r#type: PreprocessorType::DoesNotContain { value },
							log_value: true,
							field_name,
							resultant_type: input_type,
						})
					}
					"regex" => {
						let regex = match &name_value.lit {
							Lit::Str(litstr) => litstr.value(),
							_ => {
								return Err(Error::new(
									name_value.lit.span(),
									concat!(
										"only regex strings are allowed",
										" in the regex preprocessor"
									),
								));
							}
						};
						Regex::new(&regex).map_err(|err| {
							Error::new(
								name_value.lit.span(),
								format!("invalid regex: {}", err.to_string()),
							)
						})?;
						Ok(Preprocessor {
							r#type: PreprocessorType::Regex { regex },
							log_value: true,
							field_name,
							resultant_type: input_type,
						})
					}
					_ => {
						return Err(Error::new(
							name_value.span(),
							"unexpected assignment in attribute",
						))
					}
				}
			}
		}
	}

	pub fn get_processor_token(&self, field_name: &Ident) -> TokenStream {
		let resultant_type = format_ident!("{}", self.resultant_type);
		let processing_function = match &self.r#type {
			PreprocessorType::Custom { function_name } => function_name.clone(),
			PreprocessorType::TypeSpecifier { .. } => {
				"preprocess::preprocessor::type_specifier".to_string()
			}
			PreprocessorType::Nested => {
				"preprocess::PreProcessor::preprocess".to_string()
			}
			PreprocessorType::Email => {
				"preprocess::preprocessor::email".to_string()
			}
			PreprocessorType::Ip => "preprocess::preprocessor::ip".to_string(),
			PreprocessorType::Ipv4 => {
				"preprocess::preprocessor::ipv4".to_string()
			}
			PreprocessorType::Ipv6 => {
				"preprocess::preprocessor::ipv6".to_string()
			}
			PreprocessorType::Length(_) => {
				"preprocess::preprocessor::length".to_string()
			}
			PreprocessorType::Url => {
				"preprocess::preprocessor::url".to_string()
			}
			PreprocessorType::Host => {
				"preprocess::preprocessor::host".to_string()
			}
			PreprocessorType::Domain => {
				"preprocess::preprocessor::domain".to_string()
			}
			PreprocessorType::Contains { .. } => {
				"preprocess::preprocessor::contains".to_string()
			}
			PreprocessorType::DoesNotContain { .. } => {
				"preprocess::preprocessor::does_not_contain".to_string()
			}
			PreprocessorType::Required => {
				"preprocess::preprocessor::required".to_string()
			}
			PreprocessorType::Regex { .. } => {
				"preprocess::preprocessor::regex".to_string()
			}
			PreprocessorType::Trimmed => {
				"preprocess::preprocessor::trimmed".to_string()
			}
			PreprocessorType::Lowercase => {
				"preprocess::preprocessor::lowercase".to_string()
			}
			PreprocessorType::Uppercase => {
				"preprocess::preprocessor::uppercase".to_string()
			}
		};
		let processing_function = format_ident!("{}", processing_function);

		let args = match &self.r#type {
			PreprocessorType::Length(args) => match args {
				LengthValidatorArgs::Min { min } => Some(format!(
					concat!(
						"preprocess::preprocessor::LengthValidatorArgs::Min",
						" {{ min: {} }}"
					),
					min
				)),
				LengthValidatorArgs::Max { max } => Some(format!(
					concat!(
						"preprocess::preprocessor::LengthValidatorArgs::Max",
						" {{ max: {} }}"
					),
					max
				)),
				LengthValidatorArgs::MinMax { min, max } => Some(format!(
					concat!(
						"preprocess::preprocessor::LengthValidatorArgs::MinMax",
						" {{ min: {}, max: {} }}"
					),
					min, max
				)),
				LengthValidatorArgs::Exact { exact } => Some(format!(
					concat!(
						"preprocess::preprocessor::LengthValidatorArgs::Exact",
						" {{ exact: {} }}"
					),
					exact
				)),
			},
			PreprocessorType::Contains { value } => {
				Some(format!("{:?}", value))
			}
			PreprocessorType::DoesNotContain { value } => {
				Some(format!("{:?}", value))
			}
			PreprocessorType::Regex { regex } => Some(format!("{:?}", regex)),
			_ => None,
		};

		if let Some(args) = args {
			let args = format_ident!("{}", args);
			quote! {
				let #field_name : #resultant_type = #processing_function (#field_name, #args)?;
			}
		} else {
			quote! {
				let #field_name : #resultant_type = #processing_function (#field_name)?;
			}
		}
	}
}

#[derive(Debug, Clone)]
pub enum PreprocessorType {
	Custom { function_name: String },
	TypeSpecifier { type_name: String },
	Nested,
	// Validators
	Email,
	Ip,
	Ipv4,
	Ipv6,
	Length(LengthValidatorArgs),
	Url,
	Host,
	Domain,
	Contains { value: String },
	DoesNotContain { value: String },
	Required,
	Regex { regex: String },
	// Preprocessors
	Trimmed,
	Lowercase,
	Uppercase,
}

impl PreprocessorType {
	pub fn preprocessor_name(&self) -> &'static str {
		match self {
			PreprocessorType::Custom { .. } => "custom",
			PreprocessorType::TypeSpecifier { .. } => "",
			PreprocessorType::Nested => "nested",
			PreprocessorType::Email => "email",
			PreprocessorType::Ip => "ip",
			PreprocessorType::Ipv4 => "ipv4",
			PreprocessorType::Ipv6 => "ipv6",
			PreprocessorType::Length(_) => "length",
			PreprocessorType::Url => "url",
			PreprocessorType::Host => "host",
			PreprocessorType::Domain => "domain",
			PreprocessorType::Contains { .. } => "contains",
			PreprocessorType::DoesNotContain { .. } => "doesnotcontain",
			PreprocessorType::Required => "required",
			PreprocessorType::Regex { .. } => "regex",
			PreprocessorType::Trimmed => "trimmed",
			PreprocessorType::Lowercase => "lowercase",
			PreprocessorType::Uppercase => "uppercase",
		}
	}

	pub fn parse_preprocessor_name(name: &str) -> Result<Self> {
		match name {
			"custom" => Ok(PreprocessorType::Custom {
				function_name: " ".to_string(),
			}),
			"type" => Ok(PreprocessorType::TypeSpecifier {
				type_name: " ".to_string(),
			}),
			"nested" => Ok(PreprocessorType::Nested),
			"email" => Ok(PreprocessorType::Email),
			"ip" => Ok(PreprocessorType::Ip),
			"ipv4" => Ok(PreprocessorType::Ipv4),
			"ipv6" => Ok(PreprocessorType::Ipv6),
			"length" => {
				Ok(PreprocessorType::Length(LengthValidatorArgs::default()))
			}
			"url" => Ok(PreprocessorType::Url),
			"host" => Ok(PreprocessorType::Host),
			"domain" => Ok(PreprocessorType::Domain),
			"contains" => Ok(PreprocessorType::Contains {
				value: "".to_string(),
			}),
			"doesnotcontain" => Ok(PreprocessorType::DoesNotContain {
				value: "".to_string(),
			}),
			"required" => Ok(PreprocessorType::Required),
			"regex" => Ok(PreprocessorType::Regex {
				regex: "".to_string(),
			}),
			"trimmed" => Ok(PreprocessorType::Trimmed),
			"lowercase" => Ok(PreprocessorType::Lowercase),
			"uppercase" => Ok(PreprocessorType::Uppercase),
			name => Err(Error::new(
				name.span(),
				format!("unknown preprocessor `{}`", name),
			)),
		}
	}

	pub fn get_processed_type(&self, input_type: &str) -> String {
		match self {
			PreprocessorType::Custom { .. } => input_type.to_string(),
			PreprocessorType::TypeSpecifier { type_name } => type_name.clone(),
			PreprocessorType::Nested => {
				format!("{}Processed", input_type)
			}
			PreprocessorType::Email => input_type.to_string(),
			PreprocessorType::Ip => "std::net::IpAddr".to_string(),
			PreprocessorType::Ipv4 => "std::net::Ipv4Addr".to_string(),
			PreprocessorType::Ipv6 => "std::net::Ipv6Addr".to_string(),
			PreprocessorType::Length(_) => input_type.to_string(),
			PreprocessorType::Url => {
				"preprocess::validators::url::Url".to_string()
			}
			PreprocessorType::Host => "String".to_string(),
			PreprocessorType::Domain => "String".to_string(),
			PreprocessorType::Contains { .. } => input_type.to_string(),
			PreprocessorType::DoesNotContain { .. } => input_type.to_string(),
			PreprocessorType::Required => input_type.to_string(),
			PreprocessorType::Regex { .. } => input_type.to_string(),
			PreprocessorType::Trimmed => input_type.to_string(),
			PreprocessorType::Lowercase => input_type.to_string(),
			PreprocessorType::Uppercase => input_type.to_string(),
		}
	}
}
