use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, Attribute, ItemEnum, ItemStruct, Token};

mod ext_traits;
mod preprocessor;
mod process_enum;
mod process_struct;
mod processed_fields;

enum Item {
	Struct(ItemStruct),
	Enum(ItemEnum),
}

impl Parse for Item {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;
		let vis = input.parse()?;

		if input.peek(Token![struct]) {
			input
				.parse()
				.map(|item: ItemStruct| ItemStruct { vis, attrs, ..item })
				.map(Item::Struct)
		} else {
			input
				.parse()
				.map(|item: ItemEnum| ItemEnum { attrs, vis, ..item })
				.map(Item::Enum)
		}
	}
}

impl From<Item> for TokenStream {
	fn from(val: Item) -> Self {
		match val {
			Item::Struct(item) => item.into_token_stream().into(),
			Item::Enum(item) => item.into_token_stream().into(),
		}
	}
}

impl Item {
	fn into_processed(self, strict_mode: bool) -> TokenStream {
		let result = match self {
			Item::Struct(item) => {
				process_struct::into_processed(item, strict_mode)
			}
			Item::Enum(item) => process_enum::into_processed(item, strict_mode),
		};

		match result {
			Ok(token_stream) => token_stream,
			Err(error) => error.to_compile_error().into(),
		}
	}
}

#[proc_macro_attribute]
pub fn sync(args: TokenStream, input: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(input as Item);

	let strict_mode = if !args.is_empty() {
		let meta = syn::parse_macro_input!(args as syn::Meta);
		let name_value = match meta.require_name_value() {
			Ok(name_value) => name_value,
			Err(err) => {
				return err.to_compile_error().into();
			}
		};
		if !name_value.path.is_ident("strict_mode") {
			return syn::Error::new_spanned(
				name_value.path.clone(),
				"expected `strict_mode` as the attribute argument",
			)
			.to_compile_error()
			.into();
		}

		match &name_value.value {
			syn::Expr::Lit(syn::ExprLit {
				attrs: _,
				lit: syn::Lit::Bool(lit),
			}) => lit.value,
			_ => {
				return syn::Error::new_spanned(
					name_value.value.clone(),
					"expected a boolean literal as the attribute argument",
				)
				.to_compile_error()
				.into();
			}
		}
	} else {
		false
	};

	input.into_processed(strict_mode)
}
