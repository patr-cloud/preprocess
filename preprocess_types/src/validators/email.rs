use std::fmt::Display;

use idna::domain_to_ascii;
use regex::Regex;

use super::ip::validate_ip;
use crate::PreProcessError;

lazy_static::lazy_static! {
	static ref EMAIL_USER_RE: Regex = Regex::new(r"^(?i)[a-z0-9.!#$%&'*+/=?^_`{|}~-]+\z").unwrap();
	static ref EMAIL_DOMAIN_RE: Regex = Regex::new(
		r"(?i)^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*$"
	).unwrap();
	// literal form, ipv4 or ipv6 address (SMTP 4.1.3)
	static ref EMAIL_LITERAL_RE: Regex = Regex::new(r"(?i)\[([A-f0-9:\.]+)\]\z").unwrap();
}

/// Validates whether the given string is an email based on the [HTML5 spec](https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address).
/// [RFC 5322](https://tools.ietf.org/html/rfc5322) is not practical in most circumstances and allows email addresses
/// that are unfamiliar to most users.
#[must_use]
pub fn validate_email(email: &str) -> bool {
	if email.is_empty() {
		return false;
	}
	let (user_part, domain_part) = if let Some(parts) = email.rsplit_once('@') {
		parts
	} else {
		return false;
	};

	if !EMAIL_USER_RE.is_match(user_part) {
		return false;
	}

	if !validate_domain_part(domain_part) {
		// Still the possibility of an [IDN](https://en.wikipedia.org/wiki/Internationalized_domain_name)
		return match domain_to_ascii(domain_part) {
			Ok(d) => validate_domain_part(&d),
			Err(_) => false,
		};
	}

	true
}

fn validate_domain_part(domain_part: &str) -> bool {
	if EMAIL_DOMAIN_RE.is_match(domain_part) {
		return true;
	}

	// maybe we have an ip as a domain?
	match EMAIL_LITERAL_RE.captures(domain_part) {
		Some(caps) => match caps.get(1) {
			Some(c) => validate_ip(c.as_str()),
			None => false,
		},
		None => false,
	}
}

/// Validator for whether the given string is an email based on the [HTML5 spec](https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address).
/// [RFC 5322](https://tools.ietf.org/html/rfc5322) is not practical in most circumstances and allows email addresses
/// that are unfamiliar to most users.
///
/// ```rust
/// use preprocess::{validators::EmailValidator, PreProcessor};
///
/// pub fn main() {
/// 	let email: &str = "hello@example.com";
/// 	assert!(EmailValidator::from(email).preprocess().is_ok());
///
/// 	let email: &str = "hello@example.com";
/// 	assert_eq!(
/// 		EmailValidator::from(email).preprocess().unwrap(),
/// 		"hello@example.com".to_string()
/// 	);
/// }
/// ```
#[must_use]
pub trait EmailValidator: Display + Sized {
	fn validate_email(self) -> Result<Self, PreProcessError>;
}

impl<Displaylike: Display> EmailValidator for Displaylike {
	fn validate_email(self) -> Result<Self, PreProcessError> {
		if validate_email(&self.to_string()) {
			Ok(self)
		} else {
			Err(PreProcessError {})
		}
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use super::EmailValidator;
	use crate::PreProcessor;

	#[test]
	fn test_validate_email() {
		// Test cases taken from Django
		// https://github.com/django/django/blob/master/tests/validators/tests.py#L48
		let tests =
			vec![
			("email@here.com", true),
			("weirder-email@here.and.there.com", true),
			(r#"!def!xyz%abc@example.com"#, true),
			("email@[127.0.0.1]", true),
			("email@[2001:dB8::1]", true),
			("email@[2001:dB8:0:0:0:0:0:1]", true),
			("email@[::fffF:127.0.0.1]", true),
			("example@valid-----hyphens.com", true),
			("example@valid-with-hyphens.com", true),
			("test@domain.with.idn.tld.उदाहरण.परीक्षा", true),
			(r#""test@test"@example.com"#, false),
			// max length for domain name labels is 63 characters per RFC 1034
			("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", true),
			("a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.atm", true),
			(
				"a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbb.atm",
				true,
			),
			// 64 * a
			("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", false),
			("", false),
			("abc", false),
			("abc@", false),
			("abc@bar", true),
			("a @x.cz", false),
			("abc@.com", false),
			("something@@somewhere.com", false),
			("email@127.0.0.1", true),
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
				EmailValidator::from(input).preprocess().is_ok(),
				expected,
				"Email `{}` was not classified correctly",
				input
			);
		}
	}

	#[test]
	fn test_validate_email_cow() {
		let test: Cow<'static, str> = "email@here.com".into();
		assert!(EmailValidator::from(test).preprocess().is_ok());
		let test: Cow<'static, str> = String::from("email@here.com").into();
		assert!(EmailValidator::from(test).preprocess().is_ok());
		let test: Cow<'static, str> = "a@[127.0.0.1]\n".into();
		assert!(EmailValidator::from(test).preprocess().is_err());
		let test: Cow<'static, str> = String::from("a@[127.0.0.1]\n").into();
		assert!(EmailValidator::from(test).preprocess().is_err());
	}
}
