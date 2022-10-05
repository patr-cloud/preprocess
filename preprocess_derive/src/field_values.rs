use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{NamedFieldProcessor, UnnamedFieldProcessor};

#[derive(Debug)]
pub enum FieldValues {
	NoFields,
	Named(Vec<NamedFieldProcessor>),
	#[allow(dead_code)]
	Unnamed(Vec<UnnamedFieldProcessor>),
}

impl FieldValues {
	pub fn get_field_definitions(&self) -> TokenStream {
		match self {
			FieldValues::NoFields => quote! {},
			FieldValues::Named(fields) => {
				let fields = fields
					.iter()
					.map(|field| {
						let final_type = field
							.preprocessors
							.last()
							.map(|last| last.resultant_type.clone())
							.unwrap_or(format!(
								"{}",
								field.ty.to_token_stream()
							));
						let vis = field.vis.clone();
						let _attrs = field
							.attrs
							.iter()
							.filter(|attr| !attr.path.is_ident("preprocess"));
						let name = format_ident!("{}", field.name);
						let ty = format_ident!("{}", final_type);

						quote! {
							// #(#attrs) *
							#vis #name : #ty,
						}
					})
					.collect::<Vec<_>>();
				quote! {
					#(#fields) *
				}
			}
			FieldValues::Unnamed(fields) => {
				let fields = fields
					.iter()
					.map(|field| {
						let final_type = field
							.preprocessors
							.last()
							.map(|last| last.resultant_type.clone())
							.unwrap_or(format!(
								"{}",
								field.ty.to_token_stream()
							));
						let vis = field.vis.clone();
						let _attrs = field
							.attrs
							.iter()
							.filter(|attr| !attr.path.is_ident("preprocess"));
						let ty = format_ident!("{}", final_type);

						quote! {
							// #(#attrs) *
							#vis #ty,
						}
					})
					.collect::<Vec<_>>();
				quote! {
					#(#fields) *
				}
			}
		}
	}

	pub fn get_fields_comma_separated(&self) -> TokenStream {
		match self {
			FieldValues::NoFields => quote! {},
			FieldValues::Named(fields) => {
				let fields = fields
					.iter()
					.map(|field| {
						let _attrs = field
							.attrs
							.iter()
							.filter(|attr| !attr.path.is_ident("preprocess"));
						let name = format_ident!("{}", field.name);

						quote! {
							// #(#attrs) *
							#name,
						}
					})
					.collect::<Vec<_>>();
				quote! {
					#(#fields) *
				}
			}
			FieldValues::Unnamed(fields) => {
				let fields = fields
					.iter()
					.enumerate()
					.map(|(index, field)| {
						let _attrs = field
							.attrs
							.iter()
							.filter(|attr| !attr.path.is_ident("preprocess"));
						let name = format_ident!("field_{}", index);

						quote! {
							//#(#attrs) *
							#name,
						}
					})
					.collect::<Vec<_>>();
				quote! {
					#(#fields) *
				}
			}
		}
	}
}
