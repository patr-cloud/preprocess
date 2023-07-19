#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! # Preprocess
//!
//! A crate to help you preprocess your structs and enums.
//! Can be used to validate data, or to transform it.
//!
//! There are two kinds of preprocessors:
//!
//! - **Validators**: They check if the given field is valid and don't modify
//!   the value. For example: a validator could check if a string is a valid
//!   email address.
//! - **Preprocessors**: These allow you to modify the value (and possibly type)
//!   of a field. For example: a preprocessor could trim a string, or convert it
//!   to uppercase.
//!
//! ## Example usage
//!
//! ```rust
//! use preprocess::prelude::*;
//!
//! #[preprocess::sync]
//! #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
//! pub struct UserSignUpRequest {
//!     // First trims the email, then converts it to lowercase, then validates it as an email address.
//!     #[preprocess(trim, lowercase, email)]
//!     pub email: String,
//!     // First trims the password, then checks if it's at least 8 characters long.
//!     #[preprocess(trim, length(min = 8))]
//!     pub password: String,
//! }
//! ```
//!
//! ## Inheriting derive attributes
//!
//! Since the crate uses an attribute macro, it must always be the first
//! attribute on the struct or enum. A new struct / enum will be generated with
//! the name `{original_name}Processed`. The derive macro will inherit all the
//! derive attributes from the original struct / enum. For example:
//!
//! ```rust
//! use preprocess::prelude::*;
//!
//! #[preprocess::sync]
//! #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
//! #[serde(rename_all = "camelCase")]
//! pub struct UserSignUpRequest {
//! 	#[preprocess(trim, lowercase, email)]
//! 	#[serde(default)]
//! 	pub email: String,
//! 	#[serde(alias = "pass")]
//! 	#[preprocess(trim, length(min = 8))]
//! 	pub password: String,
//! }
//! ```
//!
//! The above code will generate:
//!
//! ```rust
//! #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
//! #[serde(rename_all = "camelCase")]
//! pub struct UserSignUpRequestProcessed {
//! 	#[serde(default)]
//! 	pub email: String,
//! 	#[serde(alias = "pass")]
//! 	pub password: String,
//! }
//! ```
//!
//! This way, any custom derive attributes you use (like Serde) will be
//! inherited by the generated struct / enum. This also ensures that you can
//! preprocess your struct / enum and send the preprocessed version to the
//! client, without having to write any extra code.
//!
//! ## List of allowed preprocessors
//!
//! | Preprocessor                                               | Description                                         |
//! | ---------------------------------------------------------- | --------------------------------------------------- |
//! | [`email`](`crate::validators#email`)                       | Validates a string to be a valid email address.     |
//! | [`domain`](`crate::validators#domain`)                     | Validates a string to be a valid domain name.       |
//! | [`ip`](`crate::validators#ip`)                             | Validates a string to be a valid IP Address.        |
//! | [`url`](`crate::validators#url`)                           | Validates a string to be a valid URL.               |
//! | [`length`](`crate::validators#length`)                     | Validates the length of a string.                   |
//! | [`range`](`crate::validators#range`)                       | Validates the range of a number.                    |
//! | [`contains`](`crate::validators#contains`)                 | Validates if a string contains a substring.         |
//! | [`does_not_contain`](`crate::validators#does_not_contain`) | Validates if a string does not contain a substring. |
//! | [`regex`](`crate::validators#regex`)                       | Validates a string using a regex.                   |
//! | [`type`](#enforcing-the-type-of-a-value)                   | Enforces the type of a value using `TryFrom`.       |
//! | [`trim`](`crate::validators#trim`)                         | Trims a string.                                     |
//! | [`lowercase`](`crate::validators#lowercase`)               | Converts a string to lowercase.                     |
//! | [`uppercase`](`crate::validators#uppercase`)               | Converts a string to uppercase.                     |
//! | [`custom`](#custom-preprocessors)                          | Validates a string using a custom function.         |
//!
//! More details about each preprocessor can be found in the respective module
//! documentation of [preprocessors](crate::preprocessors) and
//! [validators](crate::validators).
//!
//! ### Custom preprocessors
//!
//! You can use a custom function as a preprocessor. The function must have the
//! following signature:
//!
//! ```rust
//! fn custom_preprocessor<T>(value: T) -> Result<T, Error>;
//! ```
//!
//! The function must return a `Result` with the same type as the input. If the
//! function returns an `Err`, the error will be returned as the error of the
//! preprocessor. If the function returns an `Ok`, the value will be returned as
//! the output of the preprocessor.
//!
//! ```rust
//! pub fn custom_preprocessor(value: String) -> Result<String, Error> {
//! 	if value.len() < 8 {
//! 		return Err(Error::new(
//! 			"Password must be at least 8 characters long",
//! 		));
//! 	}
//! 	Ok(value)
//! }
//!
//! #[preprocess::sync]
//! #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
//! pub struct UserSignUpRequest {
//! 	#[preprocess(custom = "custom_preprocessor")]
//! 	pub password: String,
//! }
//! ```
//!
//! ### Enforcing the type of a value
//!
//! You can use the `type` preprocessor to enforce the type of a value. This is
//! useful when you want to convert a value to a different type. For example,
//! you might want to convert a string to an integer. You can use the `type`
//! preprocessor to do this. The `type` preprocessor uses [`TryFrom`] to convert
//! the value to the desired type. If the conversion fails, the preprocessor
//! will return an error.
//!
//! ```rust
//! #[preprocess::sync]
//! #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
//! pub struct UserSignUpRequest {
//! 	#[preprocess(type = "i32")]
//! 	pub age: i16,
//! }
//! ```
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! preprocess = "<version>"
//! ```
//!
//! Then, you can import the crate and use it like this:
//!
//! ```rust
//! use preprocess::prelude::*;
//!
//! #[preprocess::sync]
//! #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
//! pub struct UserSignUpRequest {
//!     // First trims the email, then converts it to lowercase, then validates it as an email address.
//!     #[preprocess(trim, lowercase, email)]
//!     pub email: String,
//!     // First trims the password, then checks if it's at least 8 characters long.
//!     #[preprocess(trim, length(min = 8))]
//!     pub password: String,
//! }
//! ```
//!
//! ## MSRV
//!
//! There is no MSRV as such, and to be honest, I don't see the point of an
//! MSRV, with how easy rust is to upgrade. I just use the latest version of
//! rust on my machine. That being said, I don't think I've used any new rust
//! features. So it should work on older versions of rust as well. Please open
//! an [issue](https://github.com/patr-cloud/preprocess/issues) if you're facing
//! any, well, issues.

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
