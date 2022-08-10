use std::collections::HashMap;

use preprocess_types::validators::LengthValidatorArgs;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use serde_json::{Map, Value};
use syn::{
	spanned::Spanned,
	token::{Brace, Colon, Paren},
	Attribute,
	Data,
	DeriveInput,
	Error,
	Fields,
	Lit,
	Meta::{self, List, NameValue, Path},
	NestedMeta,
	Result,
	Type,
	Visibility,
};

#[derive(Debug)]
pub struct PreProcessorAttribute {
	pub preprocessor_type: TokenStream2,
	pub output_type: Option<String>,
	pub args: Map<String, Value>,
}

pub fn parse(input: TokenStream) -> TokenStream {
	let DeriveInput {
		attrs,
		vis,
		ident,
		generics,
		data,
	} = syn::parse::<DeriveInput>(input).unwrap();

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let _global_args = match parse_attributes(attrs, ident.to_string(), false) {
		Ok(attrs) => attrs,
		Err(err) => {
			return err.into_compile_error().into();
		}
	};

	let processed_type_name = format_ident!("{ident}Processed");
	let processed_type = match data {
		Data::Struct(_) => format_ident!("struct"),
		Data::Enum(_) => format_ident!("enum"),
		Data::Union(_) => {
			return Error::new(
				ident.span(),
				"unions are currently not supported",
			)
			.into_compile_error()
			.into()
		}
	};

	let mut field_args = match data {
		Data::Struct(data) => {
			let span = data.struct_token.span();
			let result = get_preprocessors_from_fields(span, data.fields);
			let result = match result {
				Ok(value) => value,
				Err(error) => return error.into_compile_error().into(),
			};

			let mut map = HashMap::new();
			map.insert("-".to_string(), result);
			map
		}
		Data::Enum(data) => {
			let result = data
				.variants
				.into_iter()
				.map(|variant| {
					let span = variant.span();
					let result =
						get_preprocessors_from_fields(span, variant.fields)?;

					Ok((variant.ident.to_string(), result))
				})
				.collect::<Result<_>>();
			match result {
				Ok(value) => value,
				Err(error) => return error.into_compile_error().into(),
			}
		}
		Data::Union(_) => unreachable!(),
	};

	let (processor, new_type_data) = if let Some((_, paren, fields)) =
		field_args.remove("-")
	{
		// Process it as a struct
		let destructure_struct_name = format_ident!("{}", ident);
		let og_field_names =
			fields.iter().enumerate().map(|(index, (_, ident, ..))| {
				format_ident!(
					"{}",
					ident
						.as_ref()
						.map(|ident| ident.to_string())
						.unwrap_or(format!("field_{}", index))
				)
			});
		let populate_variables = og_field_names.clone();
		let destructor_preprocessors = if paren.is_some() {
			quote! {
				let #destructure_struct_name(
					#(#og_field_names),*
				) = self;
			}
		} else {
			quote! {
				let #destructure_struct_name {
					#(#og_field_names),*
				} = self;
			}
		};
		let field_processors = fields
			.iter()
			.enumerate()
			.map(|(index, (_, ident, _, _, _, attrs))| {
				let field_name = format_ident!(
					"{}",
					ident
						.as_ref()
						.map(|ident| ident.to_string())
						.unwrap_or(format!("field_{}", index))
				);
				attrs
					.iter()
					.enumerate()
					.map(|(index, attr)| {
						// Preprocess each field
						let resulting_name = if index == attrs.len() - 1 {
							format_ident!("{}", field_name)
						} else {
							format_ident!(
								"preprocessed_{}_{}",
								field_name,
								index
							)
						};
						let preprocessor = attr.preprocessor_type.clone();
						let from_name = if index == 0 {
							format_ident!("{}", field_name)
						} else {
							format_ident!(
								"preprocessed_{}_{}",
								field_name,
								index - 1
							)
						};
						let (mutable, args) = if attr.args.is_empty() {
							(quote! {}, quote! {})
						} else {
							let args = attr_args_to_map(Value::Object(
								attr.args.clone(),
							));
							(
								quote! {mut},
								quote! {
									preprocessor.set_args(#args);
								},
							)
						};
						quote! {
							let #resulting_name = {
								let #mutable preprocessor = #preprocessor ::from(#from_name);
								#args
								preprocessor.preprocess()?
							};
						}
					})
					.collect::<TokenStream2>()
			})
			.collect::<TokenStream2>();

		let new_type_fields = fields
			.iter()
			.map(|(vis, ident, _, ty, _, processors)| {
				if processors.is_empty() {
					if let Some(ident) = ident {
						quote! {
							#ident: #ty,
						}
					} else {
						quote! {
							#ty,
						}
					}
				} else {
					let last_preprocessor_name =
						processors.last().unwrap().preprocessor_type.clone();
					if let Some(ident) = ident {
						quote! {
							#vis #ident: <#last_preprocessor_name as preprocess::PreProcessor>::Processed,
						}
					} else {
						quote! {
							#vis <#last_preprocessor_name as preprocess::PreProcessor>::Processed,
						}
					}
				}
			})
			.collect::<TokenStream2>();
		(
			quote! {
				#destructor_preprocessors
				#field_processors
				Ok(#processed_type_name {
					#(#populate_variables),*
				})
			},
			if paren.is_some() {
				quote! {
					#vis #processed_type #processed_type_name #ty_generics #where_clause (
						#new_type_fields
					)
				}
			} else {
				quote! {
					#vis #processed_type #processed_type_name #ty_generics #where_clause {
						#new_type_fields
					}
				}
			},
		)
	} else {
		// Process it as an enum
		let variant_processors = field_args
			.iter()
			.map(|(variant, (_, paren, fields))| {
				let variant_name = format_ident!("{}", variant);
				let og_field_names =
					fields.iter().enumerate().map(|(index, (_, ident, ..))| {
						format_ident!(
							"{}",
							ident
								.as_ref()
								.map(|ident| ident.to_string())
								.unwrap_or(format!("field_{}", index))
						)
					});
				let populate_variables = og_field_names.clone();
				let field_processors = fields
					.iter()
					.enumerate()
					.map(|(index, (_, ident, _, _, _, attrs))| {
						let field_name = format_ident!(
							"{}",
							ident
								.as_ref()
								.map(|ident| ident.to_string())
								.unwrap_or(format!("field_{}", index))
						);
						attrs
							.iter()
							.enumerate()
							.map(|(index, attr)| {
								// Preprocess each field
								let resulting_name = if index == attrs.len() - 1
								{
									format_ident!("{}", field_name)
								} else {
									format_ident!(
										"preprocessed_{}_{}",
										field_name,
										index
									)
								};
								let preprocessor =
									attr.preprocessor_type.clone();
								let from_name = if index == 0 {
									format_ident!("{}", field_name)
								} else {
									format_ident!(
										"preprocessed_{}_{}",
										field_name,
										index - 1
									)
								};
								let (mutable, args) = if attr.args.is_empty() {
									(quote! {}, quote! {})
								} else {
									let args = attr_args_to_map(Value::Object(
										attr.args.clone(),
									));
									(
										quote! {mut},
										quote! {
											preprocessor.set_args(#args)?;
										},
									)
								};
								quote! {
									let #resulting_name = {
										let #mutable preprocessor = #preprocessor ::from(#from_name);
										#args
										preprocessor.preprocess()?
									};
								}
							})
							.collect::<TokenStream2>()
					})
					.collect::<TokenStream2>();

				if paren.is_some() {
					quote! {
						#ident :: #variant_name (
							#(#og_field_names),*
						) => {
							#field_processors
							Ok(#processed_type_name :: #variant_name (
								#(#populate_variables),*
							))
						}
					}
				} else {
					quote! {
						#ident :: #variant_name {
							#(#og_field_names),*
						} => {
							#field_processors
							Ok(#processed_type_name :: #variant_name {
								#(#populate_variables),*
							})
						}
					}
				}
			})
			.collect::<TokenStream2>();

		let new_type = field_args
			.iter()
			.map(|(variant, (_, paren, fields))| {
				let variant_name = format_ident!("{}", variant);
				let new_variant_fields = fields
					.iter()
					.map(|(vis, ident, _, ty, _, processors)| {
						if processors.is_empty() {
							if let Some(ident) = ident {
								quote! {
									#ident: #ty,
								}
							} else {
								quote! {
									#ty,
								}
							}
						} else {
							let last_preprocessor_name = processors
								.last()
								.unwrap()
								.preprocessor_type
								.clone();
							if let Some(ident) = ident {
								quote! {
									#vis #ident: <#last_preprocessor_name as preprocess::PreProcessor>::Processed,
								}
							} else {
								quote! {
									#vis <#last_preprocessor_name as preprocess::PreProcessor>::Processed,
								}
							}
						}
					})
					.collect::<TokenStream2>();
				if paren.is_some() {
					quote! {
						#variant_name (
							#new_variant_fields
						)
					}
				} else {
					quote! {
						#variant_name {
							#new_variant_fields
						}
					}
				}
			})
			.collect::<TokenStream2>();
		(
			quote! {
				match self {
					#variant_processors
				}
			},
			quote! {
				#vis #processed_type #processed_type_name #ty_generics #where_clause {
					#new_type
				}
			},
		)
	};

	quote! {
		impl #impl_generics preprocess::PreProcessor for #ident #ty_generics #where_clause {
			type Args = ();
			type Processed = #processed_type_name;

			fn preprocess(self) -> Result<Self::Processed, preprocess::PreProcessError> {
				#processor
			}
		}

		#new_type_data
	}
	.into()
}

fn get_preprocessors_from_fields(
	span: Span,
	fields: Fields,
) -> Result<(
	Option<Brace>,
	Option<Paren>,
	Vec<(
		Visibility,
		Option<proc_macro2::Ident>,
		Option<Colon>,
		Type,
		Vec<Attribute>,
		Vec<PreProcessorAttribute>,
	)>,
)> {
	match fields {
		Fields::Named(named) => {
			let fields = named
				.named
				.into_iter()
				.map(|field| {
					Ok((
						field.vis,
						field.ident.clone(),
						field.colon_token,
						field.ty.clone(),
						field.attrs.clone(),
						parse_attributes(
							field.attrs,
							{
								let ty = field.ty;
								quote!(#ty).to_string()
							},
							false,
						)?,
					))
				})
				.collect::<Result<Vec<_>>>()?;
			Ok((Some(named.brace_token), None, fields))
		}
		Fields::Unnamed(unnamed) => {
			let fields = unnamed
				.unnamed
				.into_iter()
				.map(|field| {
					Ok((
						field.vis,
						field.ident.clone(),
						field.colon_token,
						field.ty.clone(),
						field.attrs.clone(),
						parse_attributes(
							field.attrs,
							{
								let ty = field.ty;
								quote!(#ty).to_string()
							},
							false,
						)?,
					))
				})
				.collect::<Result<Vec<_>>>()?;
			Ok((None, Some(unnamed.paren_token), fields))
		}
		Fields::Unit => {
			return Err(Error::new(
				span,
				"unit structs are currently not supported",
			));
		}
	}
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
					preprocessor_type: format_ident!("{}", process_on_type)
						.to_token_stream(),
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
									parse_preprocessor(meta, &process_on_type)
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

fn parse_preprocessor(
	meta: Meta,
	type_name: &str,
) -> Result<PreProcessorAttribute> {
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
				preprocessor_type: quote!(custom),
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
			let (name, args) = preprocess_preprocessor(
				path.span(),
				name,
				type_name,
				Default::default(),
			)?;
			PreProcessorAttribute {
				preprocessor_type: name,
				output_type: None,
				args,
			}
		}
		List(list) => {
			let name = list.path.get_ident().unwrap().to_string();
			let span = list.span();
			let args = list
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
											quote!(#value),
											"preprocessor arguments"
										),
									));
								}
							};
							Ok((key, value))
						}
					}
				})
				.collect::<Result<_>>()?;
			let (name, args) =
				preprocess_preprocessor(span, name, type_name, args)?;
			PreProcessorAttribute {
				preprocessor_type: name,
				output_type: None,
				args,
			}
		}
	};

	Ok(preprocessors)
}

fn attr_args_to_map(value: Value) -> TokenStream2 {
	match value {
		Value::Null => unreachable!(),
		Value::Bool(value) => quote! {
			serde_json::Value::Bool(#value)
		},
		Value::Number(number) => {
			let number = if let Some(num) = number.as_u64() {
				quote!(#num)
			} else if let Some(num) = number.as_i64() {
				quote!(#num)
			} else if let Some(num) = number.as_f64() {
				quote!(#num)
			} else {
				unreachable!()
			};
			quote! {
				serde_json::Value::Number(#number.into())
			}
		}
		Value::String(string) => quote! {
			serde_json::Value::String(#string.to_string())
		},
		Value::Array(items) => {
			let vector = items
				.into_iter()
				.filter_map(|item| {
					if item.is_null() {
						None
					} else {
						let item = attr_args_to_map(item);
						Some(quote! {
							#item,
						})
					}
				})
				.collect::<TokenStream2>();
			quote! {
				serde_json::Value::Array(vec![
					#vector
				])
			}
		}
		Value::Object(map) => {
			let map = map
				.into_iter()
				.filter_map(|(key, value)| {
					if value.is_null() {
						None
					} else {
						let value = attr_args_to_map(value);
						Some(quote! {
							map.insert(#key.to_string(), #value);
						})
					}
				})
				.collect::<TokenStream2>();
			quote! {
				serde_json::Value::Object({
					let mut map = serde_json::Map::new();
					#map
					map
				})
			}
		}
	}
}

fn preprocess_preprocessor(
	span: Span,
	name: String,
	type_name: &str,
	args: Map<String, Value>,
) -> Result<(TokenStream2, Map<String, Value>)> {
	let result = match name.as_str() {
		"email" => (
			quote!{
				preprocess::validators::EmailValidator
			},
			if args.is_empty() {
				args
			} else {
				return Err(Error::new(
					span,
					"email preprocessor takes no arguments",
				));
			},
		),
		"length" => (
			{
				let ident = format_ident!("{}", type_name);
				quote! {
					preprocess::validators::LengthValidator::<#ident>
				}
			},
			{
				let args = serde_json::from_value::<LengthValidatorArgs>(
					Value::Object(args),
				)
				.map_err(|e| {
					Error::new(
						span,
						format!(
							"Unable to parse length preprocessor arguments: {}",
							e
						),
					)
				})?;
				if let Value::Object(map) = serde_json::to_value(args).unwrap()
				{
					map
				} else {
					unreachable!()
				}
			},
		),
		_ => (format_ident!("{}", name).to_token_stream(), args),
	};

	Ok(result)
}
