use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::format_ident;
use serde_json::Value;
use syn::{DeriveInput, Ident};

pub struct PreProcessorData {
	pub preprocessor_type: String,
	pub preprocessor_args: HashMap<String, Value>,
}

pub fn parse(input: TokenStream) -> TokenStream {
	let DeriveInput {
		attrs,
		vis,
		ident,
		generics,
		data,
	} = syn::parse::<DeriveInput>(input).unwrap();

	let global_args = attrs
		.into_iter()
		.filter(|attr| matches!(attr.style, syn::AttrStyle::Outer))
		.map(|attr| {
            let path = attr.path;
            println!("{}", quote::quote!(#path));
		})
		.collect::<Vec<_>>();

	let processed_type_name = format!("{ident}Processed");
	let processed_type = match data {
		syn::Data::Struct(_) => format_ident!("struct"),
		syn::Data::Enum(_) => format_ident!("enum"),
		syn::Data::Union(_) => todo!(),
	};

	let processed_data = match data {
		syn::Data::Struct(data) => data.fields,
		syn::Data::Enum(data) => {
			data.variants.into_iter().next().unwrap().fields
		}
		syn::Data::Union(_) => todo!(),
	};

	quote::quote! {
		impl PreProcessor for #ident {
			type Args = ();
			type Processed = #processed_type_name;

			fn preprocess(self) -> Result<Self::Processed, PreProcessError> {

			}
		}

		#vis #processed_type #processed_type_name {
			#processed_data
		}
	}
	.into()
}
