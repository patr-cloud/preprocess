use syn::Ident;

use super::{FieldValues, Preprocessor};

#[derive(Debug)]
pub struct EnumVariant {
	pub name: Ident,
	pub preprocessors: Vec<Preprocessor>,
	pub fields: FieldValues,
}
