use std::{borrow::Cow, sync::OnceLock};

use regex::Regex;

use crate::prelude::*;

// Regex from the specs
// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
// It will mark esoteric email addresses like quoted string as invalid
#[doc(hidden)]
static EMAIL_USER_REGEX: OnceLock<Regex> = OnceLock::new();

/// Validates whether the given string is an email based on the [HTML5 spec](https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address).
/// [RFC 5322](https://tools.ietf.org/html/rfc5322) is not practical in most circumstances and allows email addresses
/// that are unfamiliar to most users.
///
/// # Example
/// ```rust
/// use preprocess::prelude::*;
///
/// #[preprocess::sync]
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct LoginRequest {
///     #[preprocess(email)]
///     pub email: String,
///     #[preprocess(custom = "validate_password")]
///     pub password: String,
/// }
/// ```
#[must_use = concat!(
	"validation returns a new value instead of mutating the input.",
	" The returned value will contain the validated value,",
	" while the input will remain unchanged"
)]
pub fn validate_email<'a, T>(input: T) -> Result<T>
where
	T: Into<Cow<'a, str>> + Clone,
{
	let val = input.clone().into();
	if val.is_empty() {
		return Err(Error::new("email cannot be empty"));
	}
	let Some((user_part, domain_part)) = val.split_once('@') else {
		return Err(Error::new("email is missing '@'"));
	};

	// validate the length of the user part of the email, BEFORE doing the regex
	// according to RFC5321 the max length of the local part is 64 characters
	// https://datatracker.ietf.org/doc/html/rfc5321#section-4.5.3.1.1
	if user_part.len() > 64 {
		return Err(Error::new("email is too long"));
	}

	if !EMAIL_USER_REGEX
		.get_or_init(|| {
			Regex::new(r"^(?i)[a-z0-9.!#$%&'*+/=?^_`{|}~-]+\z").unwrap()
		})
		.is_match(user_part)
	{
		return Err(Error::new("email has invalid username"));
	}

	validate_domain(domain_part)?;

	Ok(input)
}

#[cfg(test)]
mod tests {
	use super::validate_email;

	#[test]
	fn test_validate_email() {
		// Test cases taken from Django
		// https://github.com/django/django/blob/master/tests/validators/tests.py#L48
		let tests = vec![
			("email@here.com", true),
			("weirder-email@here.and.there.com", true),
			(r#"!def!xyz%abc@example.com"#, true),
			("email@[127.0.0.1]", false),
			("email@[2001:dB8::1]", false),
			("email@[2001:dB8:0:0:0:0:0:1]", false),
			("email@[::fffF:127.0.0.1]", false),
			("example@valid-----hyphens.com", true),
			("example@valid-with-hyphens.com", true),
			("test@domain.with.idn.tld.उदाहरण.परीक्षा", true),
			(r#""test@test"@example.com"#, false),
			// max length for domain name labels is 63 characters per RFC 1034
			(
				"a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
				true,
			),
			(
				"a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.atm",
				true,
			),
			(
				"a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbb.atm",
				true,
			),
			// 64 * a
			(
				"a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
				false,
			),
			("", false),
			("abc", false),
			("abc@", false),
			("abc@bar", true),
			("a @x.cz", false),
			("abc@.com", false),
			("something@@somewhere.com", false),
			("email@127.0.0.1", false),
			("email@[127.0.0.256]", false),
			("email@[2001:db8::12345]", false),
			("email@[2001:db8:0:0:0:0:1]", false),
			("email@[::ffff:127.0.0.256]", false),
			("example@invalid-.com", false),
			("example@-invalid.com", false),
			("example@invalid.com-", false),
			("example@inv-.alid-.com", false),
			("example@inv-.-alid.com", false),
			(r#"test@example.com\n\n<script src="x.js">"#, false),
			(r#""\\\011"@here.com"#, false),
			(r#""\\\012"@here.com"#, false),
			("trailingdot@shouldfail.com.", false),
			// Trailing newlines in username or domain not allowed
			("a@b.com\n", false),
			("a\n@b.com", false),
			(r#""test@test"\n@example.com"#, false),
			("a@[127.0.0.1]\n", false),
			// underscores are not allowed
			("John.Doe@exam_ple.com", false),
		];

		for (input, expected) in tests {
			assert_eq!(
				validate_email(input).is_ok(),
				expected,
				"Email `{}` was not classified correctly",
				input
			);
		}
	}

	#[test]
	fn test_validate_email_cow() {
		let test = "email@here.com";
		assert!(validate_email(test).is_ok());
		let test = String::from("email@here.com");
		assert!(validate_email(test).is_ok());
		let test = "a@[127.0.0.1]\n";
		assert!(validate_email(test).is_err());
		let test = String::from("a@[127.0.0.1]\n");
		assert!(validate_email(test).is_err());
	}

	#[test]
	fn test_validate_email_rfc5321() {
		// 65 character local part
		let test = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa@mail.com";
		assert!(validate_email(test).is_err());
		// 256 character domain part
		let test = "a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.com";
		assert!(validate_email(test).is_err());
	}
}
