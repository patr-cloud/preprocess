use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
	Attribute,
	Error,
	Field,
	Fields,
	FieldsNamed,
	FieldsUnnamed,
	Generics,
	Ident,
	ItemStruct,
	Token,
	Type,
	Visibility,
};

use crate::{
	preprocessor::Preprocessor,
	processed_fields::{ProcessedFields, ProcessedNamed, ProcessedUnnamed},
};

pub struct ParsedStruct {
	attrs: Vec<Attribute>,
	vis: Visibility,
	struct_token: Token![struct],
	ident: Ident,
	generics: Generics,
	fields: ProcessedFields,
	semi_token: Option<Token![;]>,
	global: Vec<Preprocessor>,
}

impl TryFrom<ItemStruct> for ParsedStruct {
	type Error = Error;

	fn try_from(item: ItemStruct) -> Result<Self, Self::Error> {
		let ItemStruct {
			attrs,
			vis,
			struct_token,
			ident,
			generics,
			fields,
			semi_token,
		} = item;

		let fields = fields.try_into()?;

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
			struct_token,
			ident,
			generics,
			fields,
			semi_token,
			global,
		})
	}
}

pub fn into_processed(item: ItemStruct) -> Result<TokenStream, Error> {
	let parsed: ParsedStruct = item.try_into()?;

	let ParsedStruct {
		attrs,
		vis,
		struct_token,
		ident,
		generics,
		fields,
		semi_token,
		global,
	} = parsed;

	let processed_ident = format_ident!("{}Processed", ident);

	let new_fields = match &fields {
		ProcessedFields::Unit => Fields::Unit,
		ProcessedFields::Named(ProcessedNamed { named, brace_token }) => {
			Fields::Named(FieldsNamed {
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
			})
		}
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
							|acc, preprocessor| preprocessor.get_new_type(&acc),
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

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let global_preprocessors = global.into_iter().map(|preprocessor| {
		preprocessor.into_processor_token_stream(
			&format_ident!("value"),
			&ident.to_token_stream(),
		)
	});

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
		ProcessedFields::Unnamed(ProcessedUnnamed { unnamed, .. }) => unnamed
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
			.collect(),
	};

	Ok(quote! {
		#(#attrs)*
		#vis #struct_token #ident #generics
			#fields
		#semi_token

		#(#attrs)*
		#vis struct #processed_ident #generics
			#new_fields
		#semi_token

		impl #impl_generics #ident #ty_generics #where_clause {
			pub fn preprocess(self) -> ::std::result::Result<#processed_ident #ty_generics, ::preprocess::Error> {
				let value = self;

				#(#global_preprocessors
				)*

				let #ident
					#field_names_destructured = value;

				#(#field_preprocessors
				)*

				Ok(#processed_ident
					#field_names_destructured
				)
			}
		}
	}
	.into())
}
