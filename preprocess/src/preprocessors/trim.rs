use std::borrow::Cow;

use crate::utils::Error;

/// Preprocesses the given string and removes all leading and trailing
/// whitespaces. Returns a Cow<'a, str> to avoid unnecessary allocations.
///
/// # Example
/// ```rust
/// use preprocess::prelude::*;
///
/// #[preprocess::sync]
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct LoginRequest {
///     #[preprocess(trim)]
///     pub email: String,
///     #[preprocess(regex = "^(?=.*[A-Za-z])(?=.*\\d)[A-Za-z\\d]{8,}$")]
///     pub password: String,
/// }
/// ```
#[must_use = concat!(
	"validation returns a new value instead of mutating the input.",
	" The returned value will contain the validated value,",
	" while the input will remain unchanged"
)]
pub fn preprocess_trim<'a, T>(value: T) -> Result<Cow<'a, str>, Error>
where
	T: Into<Cow<'a, str>>,
{
	Ok(value.into().trim().to_string().into())
}
