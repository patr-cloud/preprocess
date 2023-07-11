use std::fmt::Display;

/// An error that occurs during preprocessing
/// The error contains the field that failed validation and the error message
/// that was returned by the validator.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Error {
	/// The field that failed validation.
	/// This is useful when you want to display the error message to the user.
	///
	/// Can be empty if the error is not related to a specific field.
	/// Can be set using [`Error::set_field`].
	pub field: String,
	/// The error message returned by the validator
	pub message: String,
}

impl Error {
	/// Creates a new error with the given message
	/// This does not set the field. Use [`Error::set_field`] to set the field.
	pub fn new(message: impl Into<String>) -> Self {
		Self {
			field: String::new(),
			message: message.into(),
		}
	}

	/// Sets the field which failed validation.
	pub fn set_field(mut self, field: impl Into<String>) -> Self {
		self.field = field.into();
		self
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"error preprocessing field `{}`: {}",
			self.field, self.message
		)
	}
}

impl std::error::Error for Error {}
