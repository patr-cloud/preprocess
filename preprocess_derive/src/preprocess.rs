use proc_macro::TokenStream;
use serde_json::{Map, Value};
use syn::{
	spanned::Spanned,
	Attribute,
	DeriveInput,
	Error,
	Lit,
	Meta::{self, List, NameValue, Path},
	NestedMeta,
	Result,
};

#[derive(Debug)]
pub struct PreProcessorAttribute {
	pub preprocessor_type: String,
	pub output_type: Option<String>,
	pub args: Map<String, Value>,
}

pub fn parse(input: TokenStream) -> TokenStream {
	let DeriveInput {
		attrs,
		vis: _,
		ident,
		generics: _,
		data: _,
	} = syn::parse::<DeriveInput>(input).unwrap();

	let _global_args = match parse_attributes(attrs, ident.to_string(), false) {
		Ok(attrs) => attrs,
		Err(err) => {
			return err.into_compile_error().into();
		}
	};

	// let processed_type_name = format_ident!("{ident}Processed");
	// let processed_type = match data {
	// 	Data::Struct(_) => format_ident!("struct"),
	// 	Data::Enum(_) => format_ident!("enum"),
	// 	Data::Union(_) => todo!(),
	// };

	// let processed_data = match data {
	// 	Data::Struct(data) => data.fields,
	// 	Data::Enum(data) => data.variants.into_iter().next().unwrap().fields,
	// 	Data::Union(_) => todo!(),
	// };

	// let a: TokenStream = quote::quote! {
	// 	impl preprocess::PreProcessor for #ident {
	// 		type Args = ();
	// 		type Processed = #processed_type_name;

	// 		fn preprocess(self) -> Result<Self::Processed, PreProcessError> {
	// 			Ok(#processed_type_name {
	// 				#processed_data
	// 			})
	// 		}
	// 	}

	// 	#vis #processed_type #processed_type_name {
	// 		#processed_data
	// 	}
	// }
	// .into();

	"".parse().unwrap()
}

fn parse_attributes(
	attrs: Vec<Attribute>,
	process_on_type: String,
	type_level: bool,
) -> Result<Vec<PreProcessorAttribute>> {
	attrs
		.into_iter()
		// Only select outer attributes
		.filter(|attr| matches!(attr.style, syn::AttrStyle::Outer))
		// Only select attributes with the `preprocess` name
		.filter_map(|attr| {
			if attr.path.is_ident("preprocess") {
				Some(attr.parse_meta())
			} else {
				None
			}
		})
		// Don't allow attributes without arguments at the type level
		.map::<Result<Meta>, _>(|meta| {
			let meta = match meta {
				Ok(meta) => meta,
				Err(err) => return Err(err),
			};
			if let Path(path) = &meta {
				if type_level && path.is_ident("preprocess") {
					Err(Error::new(
						meta.span(),
						"preprocess needs to have arguments at the type level",
					))
				} else {
					Ok(meta)
				}
			} else {
				Ok(meta)
			}
		})
		// Parse the arguments into a list of preprocessors
		.map(|meta| {
			let meta = match meta {
				Ok(meta) => meta,
				Err(err) => return Err(err),
			};
			let preprocessors = match meta {
				// In case there's a #[preprocess] attribute, just create a
				// preprocessor with the default arguments
				Path(_) => vec![PreProcessorAttribute {
					preprocessor_type: process_on_type.clone(),
					output_type: None,
					args: Map::new(),
				}],
				// If there's a #[preprocess(email, length, etc)] attribute,
				// parse each one of them as a preprocessor
				List(list) => {
					list.nested
						.into_iter()
						.map(|item| {
							// For each preprocessor, parse it as an attribute
							match item {
								NestedMeta::Meta(meta) => {
									parse_preprocessor(meta)
								}
								NestedMeta::Lit(_) => Err(Error::new(
									item.span(),
									concat!(
										"expected preprocessors, ",
										"found a string literal"
									),
								)),
							}
						})
						.collect::<Result<Vec<_>>>()?
				}
				NameValue(name_value) => {
					return Err(Error::new(
						name_value.span(),
						"expected a name-value pair",
					))
				}
			};

			Ok(preprocessors)
		})
		.collect::<Result<Vec<_>>>()
		.map(|vec_of_vecs| vec_of_vecs.into_iter().flatten().collect())
}

fn parse_preprocessor(meta: Meta) -> Result<PreProcessorAttribute> {
	let span = meta.span();
	let preprocessors = match meta {
		NameValue(name_value) => {
			// If there's a #[preprocess(custom = "function")] attribute
			if !name_value.path.is_ident("custom") {
				return Err(Error::new(
					span,
					concat!(
						"cannot assign a value to a preprocessor. ",
						"Did you mean to use the `custom` preprocessor?"
					),
				));
			}
			let name = name_value.path.get_ident().unwrap().to_string();
			let value = if let Lit::Str(string) = name_value.lit {
				Value::String(string.value())
			} else {
				return Err(Error::new(
					span,
					concat!(
						"custom preprocess argument must be ",
						"a string with as the function name",
					),
				));
			};
			PreProcessorAttribute {
				preprocessor_type: name,
				output_type: None,
				args: {
					let mut map = Map::new();
					map.insert("function".to_string(), value);
					map
				},
			}
		}
		Path(path) => {
			let name = path.get_ident().unwrap().to_string();
			PreProcessorAttribute {
				preprocessor_type: name,
				output_type: None,
				args: Default::default(),
			}
		}
		List(list) => PreProcessorAttribute {
			preprocessor_type: list.path.get_ident().unwrap().to_string(),
			output_type: None,
			args: list
				.nested
				.into_iter()
				.map(|item| {
					let meta = match item {
						NestedMeta::Meta(meta) => meta,
						NestedMeta::Lit(_) => {
							return Err(Error::new(
								item.span(),
								concat!(
									"expected preprocessor arguments, found a ",
									"string. Try (arg_field = value) instead"
								),
							));
						}
					};
					Ok(meta)
				})
				.map(|meta| {
					let meta = match meta {
						Ok(meta) => meta,
						Err(err) => return Err(err),
					};
					let span = meta.span();
					match meta {
						Path(path) => {
							return Err(Error::new(
								span,
								format!(
									"expected a name-value pair. Found `{}`",
									path.get_ident().unwrap()
								),
							));
						}
						List(list) => {
							return Err(Error::new(
								span,
								format!(
									"expected a name-value pair. {} `{}`",
									"Found a list of arguments for ",
									list.path.get_ident().unwrap()
								),
							));
						}
						NameValue(name_value) => {
							let key = name_value
								.path
								.get_ident()
								.unwrap()
								.to_string();
							let value = match name_value.lit {
								Lit::Str(string) => {
									let string = string.value();
									serde_json::from_str(&string)
										.unwrap_or(Value::String(string))
								}
								Lit::Char(char) => {
									Value::String(char.value().to_string())
								}
								Lit::Int(int) => {
									Value::Number(int.base10_parse().unwrap())
								}
								Lit::Float(float) => {
									Value::Number(float.base10_parse().unwrap())
								}
								Lit::Bool(boolean) => {
									Value::Bool(boolean.value())
								}
								value => {
									return Err(Error::new(
										span,
										format!(
											"unknown value `{}` for {}",
											quote::quote!(#value),
											"preprocessor arguments"
										),
									));
								}
							};
							Ok((key, value))
						}
					}
				})
				.collect::<Result<_>>()?,
		},
	};

	Ok(preprocessors)
}
