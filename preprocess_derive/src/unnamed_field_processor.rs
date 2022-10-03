use syn::{Attribute, Type, Visibility};

use super::Preprocessor;

#[derive(Debug)]
pub struct UnnamedFieldProcessor {
	pub attrs: Vec<Attribute>,
	pub vis: Visibility,
	pub ty: Type,
	pub preprocessors: Vec<Preprocessor>,
}
