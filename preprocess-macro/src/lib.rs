use proc_macro::TokenStream;
use syn::{parse::Parse, ItemEnum, ItemStruct, __private::ToTokens};

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
		match self {
			Item::Struct(item) => into_processed_struct(item),
			Item::Enum(item) => into_processed_enum(item),
		}
	}
}

#[proc_macro_attribute]
pub fn process(args: TokenStream, input: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(input as Item);

	if let Some(token) = args.into_iter().next() {
		return syn::Error::new(token.span().into(), "unexpected arguments")
			.to_compile_error()
			.into();
	}

	input.into_processed()
}
