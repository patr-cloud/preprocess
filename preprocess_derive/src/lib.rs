use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
	parenthesized,
	parse::{Parse, Parser},
	parse_macro_input,
	punctuated::Punctuated,
	spanned::Spanned,
	Attribute,
	Data,
	DataEnum,
	DataStruct,
	DeriveInput,
	Error,
	Expr,
	ExprLit,
	ExprPath,
	Fields,
	FieldsNamed,
	FieldsUnnamed,
	Lit,
	LitStr,
	MetaNameValue,
	Token,
};

type Result<T> = syn::Result<T>;

#[proc_macro_derive(PreProcess, attributes(preprocess))]
pub fn derive_preprocess(
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	impl_preprocess_trait(input)
		.unwrap_or_else(syn::Error::into_compile_error)
		.into()
}

fn impl_preprocess_trait(input: DeriveInput) -> Result<TokenStream> {
	let preprocess_method = match input.data {
		Data::Struct(data_struct) => impl_for_struct(data_struct),
		Data::Enum(data_enum) => impl_for_enum(data_enum),
		Data::Union(_) => {
			Err(Error::new(input.span(), "Union type is not supported"))
		}
	}?;

	let ident = input.ident;
	let (impl_generics, type_generics, where_clause) =
		input.generics.split_for_impl();
	let output = quote!(
		impl #impl_generics ::preprocess::PreProcess for #ident #type_generics #where_clause {
			fn preprocess(&mut self) -> ::std::result::Result<(), ::preprocess::PreProcessError> {
				#preprocess_method
				Ok(())
			}
		}
	);

	Ok(output)
}

fn impl_for_struct(data_struct: DataStruct) -> Result<TokenStream> {
	let (fields, impls) = impl_for_fields(data_struct.fields)?;
	let result = quote!(
		let Self #fields = self;
		#impls
	);
	Ok(result)
}

fn impl_for_enum(data_enum: DataEnum) -> Result<TokenStream> {
	let match_block = data_enum
		.variants
		.into_iter()
		.map(|varient| {
			let (fields, impls) = impl_for_fields(varient.fields)?;
			let varient_name = varient.ident;
			let result = quote!(
				Self::#varient_name #fields => {
					#impls
				}
			);
			Ok(result)
		})
		.collect::<Result<TokenStream>>()?;

	Ok(quote!(
		match self {
			#match_block
		}
	))
}

fn impl_for_fields(field: Fields) -> Result<(TokenStream, TokenStream)> {
	match field {
		Fields::Named(FieldsNamed { named, .. }) => {
			let expanded_fields =
				named.iter().filter_map(|field| field.ident.as_ref());
			let expanded_fields = quote!(
				{ #( #expanded_fields ),* }
			);

			let expanded_field_impls = named
				.into_iter()
				.filter_map(|field| Some((field.ident?, field.attrs)))
				.map(|(id, attrs)| impl_for_field(&id, attrs))
				.collect::<Result<TokenStream>>()?;

			Ok((expanded_fields, expanded_field_impls))
		}
		Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
			let expanded_fields = unnamed
				.iter()
				.enumerate()
				.map(|(idx, _)| format_ident!("f_{}", idx));
			let expanded_fields = quote!(
				( #( #expanded_fields ),* )
			);

			let expanded_field_impls = unnamed
				.into_iter()
				.enumerate()
				.map(|(idx, field)| (format_ident!("f_{}", idx), field.attrs))
				.map(|(id, attrs)| impl_for_field(&id, attrs))
				.collect::<Result<TokenStream>>()?;

			Ok((expanded_fields, expanded_field_impls))
		}
		Fields::Unit => Ok((TokenStream::new(), TokenStream::new())),
	}
}

fn impl_for_field(ident: &Ident, attrs: Vec<Attribute>) -> Result<TokenStream> {
	attrs
		.into_iter()
		.filter(|attr| attr.path().is_ident("preprocess"))
		.map(|attr| match attr.meta {
			syn::Meta::Path(_path) => Ok(quote!(
				::preprocess::PreProcess::preprocess(&mut *#ident)?;
			)),
			syn::Meta::List(list) => Ok(
				Punctuated::<AllowedOps, Token![,]>::parse_separated_nonempty
					.parse2(list.tokens)?
					.into_iter()
					.map(|ops| ops.to_tokens(ident))
					.collect(),
			),
			syn::Meta::NameValue(name_value) => Err(Error::new(
				name_value.span(),
				"Attribute format not supported",
			)),
		})
		.try_fold(TokenStream::new(), |mut accu, item| {
			accu.extend(item?);
			Ok(accu)
		})
}

enum AllowedOps {
	Trim,
	Lowercase,
	Length {
		min: Option<usize>,
		max: Option<usize>,
	},
	Regex(LitStr),
	Process(ExprPath),
	ProcessMut(ExprPath),
}

impl AllowedOps {
	fn to_tokens(&self, ident: &Ident) -> TokenStream {
		match self {
			Self::Trim => quote!(
				::preprocess::process::trim(&mut *#ident)?;
			),
			Self::Lowercase => quote!(
				::preprocess::process::lowercase(&mut *#ident)?;
			),
			Self::Length { min, max } => {
				let min = min.map_or_else(
					|| quote!(::std::option::Option::None),
					|min| quote!(::std::option::Option::Some(#min)),
				);
				let max = max.map_or_else(
					|| quote!(::std::option::Option::None),
					|max| quote!(::std::option::Option::Some(#max)),
				);

				quote!(
					::preprocess::process::length(& *#ident, #min, #max)?;
				)
			}
			Self::Regex(pattern) => {
				quote!({
					let regex_pattern = {
						static RE: ::once_cell::sync::OnceCell<::regex::Regex> = ::once_cell::sync::OnceCell::new();
						RE.get_or_init(|| ::regex::Regex::new(#pattern).unwrap())
					};
					::preprocess::process::regex(& *#ident, regex_pattern)?;
				})
			}
			Self::Process(expr) => quote!(
				::preprocess::process::process(& *#ident, #expr)?;
			),
			Self::ProcessMut(expr) => quote!(
				::preprocess::process::process_mut(&mut *#ident, #expr)?;
			),
		}
	}
}

impl Parse for AllowedOps {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let ident: Ident = input.parse()?;
		match ident.to_string().as_str() {
			"trim" => Ok(Self::Trim),
			"lowercase" => Ok(Self::Lowercase),
			"length" => {
				let inner_content;
				parenthesized!(inner_content in input);

				let parts = inner_content
					.parse_terminated(MetaNameValue::parse, Token![,])?;

				let mut min = None;
				let mut max = None;

				fn expr_to_usize(expr: &Expr) -> Result<usize> {
					match expr {
						Expr::Lit(ExprLit {
							lit: Lit::Int(int), ..
						}) => Some(int),
						_ => None,
					}
					.and_then(|int| int.base10_parse().ok())
					.ok_or_else(|| Error::new(expr.span(), "Expected int type"))
				}

				for part in &parts {
					if part.path.is_ident("min") {
						let old_value =
							min.replace(expr_to_usize(&part.value)?);
						if old_value.is_some() {
							return Err(Error::new(
								part.span(),
								"Min value already provided",
							));
						}
					} else if part.path.is_ident("max") {
						let old_value =
							max.replace(expr_to_usize(&part.value)?);
						if old_value.is_some() {
							return Err(Error::new(
								part.span(),
								"Max value already provided",
							));
						}
					} else {
						return Err(Error::new(part.span(), "Invalid expr"));
					}
				}

				if min.is_none() && max.is_none() {
					return Err(Error::new(
						parts.span(),
						"Both min and max should not be empty",
					));
				}

				Ok(Self::Length { min, max })
			}
			"regex" => {
				input.parse::<Token![=]>()?;
				let pattern = input.parse::<LitStr>()?;
				Ok(Self::Regex(pattern))
			}
			"process" => {
				let inner_content;
				parenthesized!(inner_content in input);
				Ok(Self::Process(inner_content.parse()?))
			}
			"process_mut" => {
				let inner_content;
				parenthesized!(inner_content in input);
				Ok(Self::ProcessMut(inner_content.parse()?))
			}
			_ => Err(Error::new(input.span(), "Unknown attribute value")),
		}
	}
}
