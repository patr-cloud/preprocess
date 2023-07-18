#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../../README.md")]

/// An attribute macro for preprocessing structs
pub use preprocess_macro::sync;

/// Error type for the library. This type is used to return errors from the
/// preprocessors and validators.
pub use crate::utils::Error;

/// List of all the preprocessors that mutates the given field, including
/// changing the type if required.
pub mod preprocessors;
/// Utility module for the library.
pub mod utils;
/// List of all the validators that validates the given field without mutating
/// it. The type of the field may still be changed. For example, the
/// [`ip`](crate::validators::validate_ip) validator will change the type
/// of the field to [`IpAddr`](std::net::IpAddr).
pub mod validators;

/// Prelude module for the library. This module re-exports all the important
/// types and traits from the library. This module is useful when you want to
/// use the library without importing the individual modules.
///
/// # Example
/// ```rust
/// use preprocess::prelude::*;
///
/// #[preprocess::sync]
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct LoginRequest {
/// 	#[preprocess(email)]
/// 	pub email: String,
/// 	#[preprocess(custom = "validate_password")]
/// 	pub password: String,
/// }
/// ```
pub mod prelude {
	pub use crate::{preprocessors::*, utils::*, validators::*};

	/// An alias for [`std::result::Result`] with the error type set to
	/// [`Error`].
	pub type Result<T> = std::result::Result<T, Error>;
}

/// A list of all the types that are re-exported from supporting crates. Used by
/// the preprocessor to set the types for a field if required.
pub mod types {
	pub use url::Url;
}
