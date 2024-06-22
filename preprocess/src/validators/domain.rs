use std::borrow::Cow;

use crate::utils::Error;

/// Checks if the domain is a valid domain or not
/// IP addresses are not allowed. To validate IP addresses, use the
/// [`validate_ip`] validator.
///
/// # Example
/// ```rust
/// use preprocess::prelude::*;
///
/// #[preprocess::sync]
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct AddDomainRequest {
///     #[preprocess(domain)]
///     pub domain: String,
/// }
/// ```
///
/// [`validate_ip`]: crate::validators::ip::validate_ip
#[must_use = concat!(
	"validation returns a new value instead of mutating the input.",
	" The returned value will contain the validated value,",
	" while the input will remain unchanged"
)]
pub fn validate_domain<'a, T>(domain: T) -> Result<T, Error>
where
	T: Into<Cow<'a, str>> + Clone,
{
	let val = domain.clone().into();
	if val.len() > 253 {
		return Err(Error::new("domain name too long"));
	}

	if val.is_empty() {
		return Err(Error::new("domain name cannot be empty"));
	}

	if val.contains("..") {
		return Err(Error::new(
			"domain name cannot contain two consecutive dots",
		));
	}

	idna::domain_to_ascii_cow(val.as_bytes(), idna::AsciiDenyList::URL)
		.map_err(|err| Error::new(format!("invalid domain: {}", err)))?;

	Ok(domain)
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
			("-google.com".to_string(), true),
			("google-.com".to_string(), true),
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
			// Domain with a single label equal to 63 characters (which should
			// be valid)
			(format!("{}.com", "a".repeat(63)), true),
			// Domain with total characters equal to 253 (which should be
			// valid)
			(format!("{}.{}", "a".repeat(63), "a".repeat(187)), true),
		];

		for (domain, expected) in test_cases {
			assert_eq!(validate_domain(domain).is_ok(), expected);
		}
	}
}
