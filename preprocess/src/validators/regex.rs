use std::{borrow::Cow, sync::OnceLock};

use dashmap::DashMap;
use regex::Regex;

use crate::utils::Error;

static REGEX_LIST: OnceLock<DashMap<String, Regex>> = OnceLock::new();

/// Validates whether the given string matches the given regex.
/// The regex is compiled using [`regex::Regex::new`].
/// If the regex is invalid, then an error is returned.
///
/// # Example
/// ```rust
/// use preprocess::prelude::*;
///
/// #[preprocess::sync]
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct LoginRequest {
/// 	#[preprocess(custom = "validate_regex")]
/// 	pub email: String,
/// 	#[preprocess(regex = "^(?=.*[A-Za-z])(?=.*\\d)[A-Za-z\\d]{8,}$")]
/// 	pub password: String,
/// }
/// ```
#[must_use]
pub fn validate_regex<'a, T>(value: T, regex: &str) -> Result<T, Error>
where
	T: Into<Cow<'a, str>> + Clone,
{
	let val = value.clone().into();
	if regex.is_empty() {
		return Err(Error::new("regex cannot be empty"));
	}

	REGEX_LIST
		.get_or_init(|| DashMap::new())
		.entry(regex.to_string())
		.or_try_insert_with(|| {
			Regex::new(regex)
				.map_err(|err| Error::new(format!("invalid regex: {}", err)))
		})?
		.is_match(&val)
		.then(|| value)
		.ok_or_else(|| Error::new("regex validation failed"))
}
