use proc_macro::TokenStream;
use syn::{
	parse::Parse,
	Attribute,
	ItemEnum,
	ItemStruct,
	Token,
	__private::ToTokens,
};

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
	fn into_processed(self) -> TokenStream {
		let result = match self {
			Item::Struct(item) => process_struct::into_processed(item),
			Item::Enum(item) => process_enum::into_processed(item),
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

	if let Some(token) = args.into_iter().next() {
		return syn::Error::new(token.span().into(), "unexpected arguments")
			.to_compile_error()
			.into();
	}

	input.into_processed()
}
