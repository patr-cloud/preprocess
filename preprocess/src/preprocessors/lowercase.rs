use std::borrow::Cow;

use crate::utils::Error;

/// Preprocesses the given string and converts it to lowercase using the
/// `to_lowercase` method. Returns a Cow<'a, str> to avoid unnecessary
/// allocations.
///
/// # Example
/// ```rust
/// use preprocess::prelude::*;
///
/// #[preprocess::sync]
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct LoginRequest {
/// 	#[preprocess(lowercase)]
/// 	pub email: String,
/// 	#[preprocess(regex = "^(?=.*[A-Za-z])(?=.*\\d)[A-Za-z\\d]{8,}$")]
/// 	pub password: String,
/// }
/// ```
#[must_use]
pub fn preprocess_lowercase<'a, T>(value: T) -> Result<Cow<'a, str>, Error>
where
	T: Into<Cow<'a, str>>,
{
	Ok(value.into().to_lowercase().into())
}
