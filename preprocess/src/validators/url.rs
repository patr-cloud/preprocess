use std::fmt::Display;

use url::Url;

use crate::{PreProcessError, PreProcessor};

/// Validates whether the string given is a url
#[must_use]
pub fn validate_url(url: &str) -> bool {
	Url::try_from(url).is_ok()
}

/// Validator for whether the given string is a URL
///
/// ```rust
/// use preprocess::{validators::UrlValidator, PreProcessor};
/// use url::Url;
///
/// pub fn main() {
/// 	let url: &str = "https://google.com";
/// 	assert!(UrlValidator::from(url).preprocess().is_ok());
///
/// 	let url: &str = "https://google.com";
/// 	assert_eq!(
/// 		UrlValidator::from(url).preprocess().unwrap(),
/// 		Url::parse("https://google.com").unwrap()
/// 	);
/// }
/// ```
#[must_use]
#[derive(Debug, Clone)]
pub struct UrlValidator {
	data: String,
}

impl UrlValidator {
	/// Validates whether the given string is a valid url or not.
	///
	/// ```rust
	/// use preprocess::validators::UrlValidator;
	///
	/// pub fn main() {
	/// 	let url: &str = "https://google.com";
	/// 	assert!(UrlValidator::from(url).validate());
	/// }
	pub fn validate(&self) -> bool {
		validate_url(&self.data)
	}
}

impl<Displaylike: Display> From<Displaylike> for UrlValidator {
	/// Creates a new [`UrlValidator`] from any struct that implements the
	/// [`Display`] trait.
	///
	/// ```rust
	/// use preprocess::validators::UrlValidator;
	///
	/// pub fn main() {
	/// 	let validator = UrlValidator::from("https://google.com");
	/// 	assert_eq!(validator.validate(), true);
	/// }
	/// ```
	fn from(data: Displaylike) -> Self {
		UrlValidator {
			data: data.to_string(),
		}
	}
}

impl PreProcessor for UrlValidator {
	/// Does not require any arguments
	type Args = ();
	/// Returns a [`Url`] if the url is valid or an error if it is not.
	///
	/// ```rust
	/// use preprocess::{validators::UrlValidator, PreProcessor};
	/// use url::Url;
	///
	/// pub fn main() {
	/// 	let validated_url: Url = UrlValidator::from("https://duckduckgo.com")
	/// 		.preprocess()
	/// 		.unwrap();
	/// 	assert_eq!(validated_url, "https://duckduckgo.com".parse().unwrap());
	/// }
	/// ```
	type Processed = Url;

	/// Validates whether the given string is a valid url or not, returning an
	/// error if it is not, or a [`Url`] with the validated url if it is.
	///
	/// ```rust
	/// use preprocess::{validators::UrlValidator, PreProcessError, PreProcessor};
	/// use url::Url;
	///
	/// pub fn main() {
	/// 	let validated_url: Result<Url, PreProcessError> =
	/// 		UrlValidator::from("https://duckduckgo.com").preprocess();
	/// 	assert_eq!(
	/// 		validated_url,
	/// 		Ok(Url::parse("https://duckduckgo.com").unwrap())
	/// 	);
	/// }
	/// ```
	fn preprocess(self) -> Result<Url, PreProcessError> {
		if let Ok(url) = Url::try_from(self.data.as_str()) {
			Ok(url)
		} else {
			Err(PreProcessError {})
		}
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use super::validate_url;

	#[test]
	fn test_validate_url() {
		let tests = vec![
			("http", false),
			("https://google.com", true),
			("http://localhost:80", true),
			("ftp://localhost:80", true),
		];

		for (url, expected) in tests {
			assert_eq!(validate_url(url), expected);
		}
	}

	#[test]
	fn test_validate_url_cow() {
		let test: Cow<'static, str> = "http://localhost:80".into();
		assert!(validate_url(&test));
		let test: Cow<'static, str> =
			String::from("http://localhost:80").into();
		assert!(validate_url(&test));
		let test: Cow<'static, str> = "http".into();
		assert!(!validate_url(&test));
		let test: Cow<'static, str> = String::from("http").into();
		assert!(!validate_url(&test));
	}
}
