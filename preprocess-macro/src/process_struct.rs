use std::collections::HashMap;

use proc_macro::TokenStream;
use syn::{
	Attribute,
	Field,
	Fields,
	FieldsUnnamed,
	Generics,
	Ident,
	ItemStruct,
	Token,
	Visibility,
	__private::quote::quote,
};

pub struct ParsedStruct {
	attrs: Vec<Attribute>,
	vis: Visibility,
	struct_token: Token![struct],
	ident: Ident,
	generics: Generics,
	fields: Fields,
	semi_token: Token![;],
	
}

pub fn into_processed(item: ItemStruct) -> TokenStream {
	let ItemStruct {
		attrs,
		vis,
		struct_token,
		ident,
		generics,
		fields,
		semi_token,
	} = item;

	let mut processors = HashMap::<Ident, Vec<String>>::new();

	let fields = match fields {
		Fields::Named(_) => todo!(),
		Fields::Unnamed(fields) => Fields::Unnamed(FieldsUnnamed {
			unnamed: fields
				.unnamed
				.into_iter()
				.map(|field| {
					let Field {
						attrs,
						vis,
						ty,
						ident,
						colon_token,
						mutability,
					} = field;

					Field {
						attrs: attrs
							.into_iter()
							.filter(|attr| {
								match attr.meta {}
								true
							})
							.collect(),
						vis,
						ty,
						ident,
						colon_token,
						mutability,
					}
				})
				.collect(),
			paren_token: fields.paren_token,
		}),
		Fields::Unit => Fields::Unit,
	};

	quote! {
		#(#attrs)*
		#vis #struct_token #ident #generics
			#fields
		#semi_token

		impl #ident #generics {
			pub fn preprocess(self) -> #ident {

			}
		}
	}
	.into()
}
