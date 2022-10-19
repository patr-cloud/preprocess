use proc_macro::TokenStream;

mod enum_variant;
mod field_values;
mod named_field_processor;
mod preprocessor;
mod preprocessor_derive;
mod unnamed_field_processor;

use self::{
	enum_variant::*,
	field_values::*,
	named_field_processor::*,
	preprocessor::*,
	preprocessor_derive::*,
	unnamed_field_processor::*,
};

#[proc_macro_derive(PreProcess, attributes(preprocess, validate, map))]
pub fn preprocess(input: TokenStream) -> TokenStream {
	parse(input)
}

fn parse(input: TokenStream) -> TokenStream {
	let derive = match PreprocessorDerive::try_from(input) {
		Ok(derive) => derive,
		Err(err) => return err.into_compile_error().into(),
	};

	let value = derive.preprocess_tokens().into();

	println!("{}", value);

	value
}
