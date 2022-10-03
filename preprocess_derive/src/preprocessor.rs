use preprocess_types::validators::LengthValidatorArgs;
use syn::{spanned::Spanned, Attribute, Error, Lit, Meta, NestedMeta, Result};

#[derive(Debug)]
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
			Meta::List(list) => list
				.nested
				.into_iter()
				.map(|meta| match meta {
					NestedMeta::Meta(meta) => Preprocessor::from_meta(
						field_name.clone(),
						input_type.clone(),
						meta,
					),
					NestedMeta::Lit(_) => todo!(),
				})
				.collect::<Result<Vec<_>>>(),
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
			Meta::Path(_) => todo!(),
			Meta::List(_) => todo!(),
			Meta::NameValue(name_value) => {
				if name_value.path.get_ident().unwrap().to_string().as_str() ==
					"custom"
				{
					let function_name = match name_value.lit {
						Lit::Str(litstr) => litstr.value(),
						_ => {
							return Err(Error::new(
								name_value.span(),
								concat!(
									"only function names as strings",
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
				} else {
					return Err(Error::new(
						name_value.span(),
						"unexpected assignment in attribute",
					));
				}
			}
		}
	}
}

#[derive(Debug)]
pub enum PreprocessorType {
	Custom { function_name: String },
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
	Regex { value: String },
	// Preprocessors
	Trimmed,
	Lowercase,
	Uppercase,
}

impl PreprocessorType {
	pub fn preprocessor_name(&self) -> &'static str {
		match self {
			PreprocessorType::Custom { function_name: _ } => "custom",
			PreprocessorType::Nested => "nested",
			PreprocessorType::Email => "email",
			PreprocessorType::Ip => "ip",
			PreprocessorType::Ipv4 => "ipv4",
			PreprocessorType::Ipv6 => "ipv6",
			PreprocessorType::Length(_) => "length",
			PreprocessorType::Url => "url",
			PreprocessorType::Host => "host",
			PreprocessorType::Domain => "domain",
			PreprocessorType::Contains { value } => "contains",
			PreprocessorType::DoesNotContain { value } => "doesnotcontain",
			PreprocessorType::Required => "required",
			PreprocessorType::Regex { value } => "regex",
			PreprocessorType::Trimmed => "trimmed",
			PreprocessorType::Lowercase => "lowercase",
			PreprocessorType::Uppercase => "uppercase",
		}
	}

	pub fn get_processed_type(&self, input_type: &str) -> String {
		match self {
			PreprocessorType::Custom { function_name } => todo!(),
			PreprocessorType::Nested => {
				format!("{}Processed", input_type)
			}
			PreprocessorType::Email => todo!(),
			PreprocessorType::Ip => todo!(),
			PreprocessorType::Ipv4 => todo!(),
			PreprocessorType::Ipv6 => todo!(),
			PreprocessorType::Length(_) => todo!(),
			PreprocessorType::Url => todo!(),
			PreprocessorType::Host => todo!(),
			PreprocessorType::Domain => todo!(),
			PreprocessorType::Contains { value } => todo!(),
			PreprocessorType::DoesNotContain { value } => todo!(),
			PreprocessorType::Required => todo!(),
			PreprocessorType::Regex { value } => todo!(),
			PreprocessorType::Trimmed => todo!(),
			PreprocessorType::Lowercase => todo!(),
			PreprocessorType::Uppercase => todo!(),
		}
	}
}
