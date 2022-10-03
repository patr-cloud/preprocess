use super::{NamedFieldProcessor, UnnamedFieldProcessor};

#[derive(Debug)]
pub enum FieldValues {
	NoFields,
	Named(Vec<NamedFieldProcessor>),
	Unnamed(Vec<UnnamedFieldProcessor>),
}
