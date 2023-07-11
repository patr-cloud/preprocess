use proc_macro::TokenStream;
use syn::{parse::Parse, ItemEnum, ItemStruct, __private::ToTokens};

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
		input
			.parse::<ItemStruct>()
			.map(Item::Struct)
			.or_else(|_| input.parse::<ItemEnum>().map(Item::Enum))
	}
}

impl Into<TokenStream> for Item {
	fn into(self) -> TokenStream {
		match self {
			Item::Struct(item) => item.into_token_stream().into(),
			Item::Enum(item) => item.into_token_stream().into(),
		}
	}
}

impl Item {
	fn into_processed(self) -> TokenStream {
		let result = match self {
			Item::Struct(item) => process_struct::into_processed(item),
			Item::Enum(item) => process_enum::into_processed(item),
		};

		match result {
			Ok(token_stream) => token_stream.into(),
			Err(error) => error.to_compile_error().into(),
		}
	}
}

#[proc_macro_attribute]
pub fn sync(args: TokenStream, input: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(input as Item);

	if let Some(token) = args.into_iter().next() {
		return syn::Error::new(token.span().into(), "unexpected arguments")
			.to_compile_error()
			.into();
	}

	input.into_processed()
}
