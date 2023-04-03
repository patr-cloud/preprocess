pub mod process;

use std::{borrow::Cow, error::Error, fmt::Display};

pub use preprocess_derive::PreProcess;

#[derive(Debug, serde::Deserialize)]
pub struct PreProcessError(Cow<'static, str>);

impl PreProcessError {
	pub fn new(err: impl Into<Cow<'static, str>>) -> Self {
		Self(err.into())
	}
}

impl Display for PreProcessError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl Error for PreProcessError {}

pub trait PreProcess {
	fn preprocess(&mut self) -> Result<(), PreProcessError>;
}
