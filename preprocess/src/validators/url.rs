use std::borrow::Cow;

use crate::utils::Error;

/// Checks if the given string is a valid Url or not
/// IP addresses are not allowed. To validate IP addresses, use the
/// [`validate_ip`] validator.
///
/// # Example
/// ```rust
/// use preprocess::prelude::*;
///
/// #[preprocess::sync]
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct SetUrlRequest {
/// 	#[preprocess(url)]
/// 	pub url: String,
/// }
/// ```
///
/// [`validate_ip`]: crate::validators::ip::validate_ip
#[must_use]
pub fn validate_url<'a, T>(domain: T) -> Result<crate::types::Url, Error>
where
	T: Into<Cow<'a, str>>,
{
	domain
		.into()
		.parse()
		.map_err(|err| Error::new(format!("invalid url: {}", err)))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_validate_domain() {
		let test_cases = vec![
			("com".to_string(), true),
			("org".to_string(), true),
			("net".to_string(), true),
			("google.com".to_string(), true),
			("wikipedia.org".to_string(), true),
			("stackoverflow.net".to_string(), true),
			("mail.google.com".to_string(), true),
			("en.wikipedia.org".to_string(), true),
			("forums.stackoverflow.net".to_string(), true),
			("open-ai.com".to_string(), true),
			("sub_domain.domain.org".to_string(), true),
			("münchen.de".to_string(), true),
			("россия.рф".to_string(), true),
			("".to_string(), false),
			("goo gle.com".to_string(), false),
			("-google.com".to_string(), false),
			("google-.com".to_string(), false),
			("#google.com".to_string(), false),
			("google@.com".to_string(), false),
			("google..com".to_string(), false),
			("0.com".to_string(), true),
			("1.net".to_string(), true),
			("a--a.com".to_string(), true),
			("xn--80akhbyknj4f.com".to_string(), true),
			// Add a test case for domain exceeding 253 characters overall,
			// and domain with a single label exceeding 63 characters.
			("xn--80akhbyknj4f.com".to_string(), true),
			// Domain with more than 253 characters
			(format!("{}{}", "a".repeat(250), ".com"), false),
			// Domain with a single label exceeding 63 characters
			(format!("{}.com", "a".repeat(64)), false),
			// Domain with a single label equal to 63 characters (which should
			// be valid)
			(format!("{}.com", "a".repeat(63)), true),
			// Domain with total characters equal to 253 (which should be
			// valid)
			(format!("{}.{}", "a".repeat(63), "a".repeat(187)), true),
		];

		for (domain, expected) in test_cases {
			assert_eq!(validate_url(domain).is_ok(), expected);
		}
	}
}
