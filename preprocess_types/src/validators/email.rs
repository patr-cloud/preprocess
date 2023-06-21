use std::borrow::Cow;

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
/// Returns an error if it is not, or the email if it is.
/// ```rust
/// use preprocess::validators::validate_email;
///
/// pub fn main() {
/// 	let validated_email = validate_email("hello@example.com");
/// 	assert_eq!(validated_email, Ok("hello@example.com"));
/// }
/// ```
#[must_use]
pub fn validate_email<'a, S>(
	email: S,
	allow_ips: bool,
) -> Result<S, PreProcessError>
where
	S: Into<Cow<'a, str>>,
{
	let email_str = email.into();

	if email_str.is_empty() {
		return Err(PreProcessError {});
	}

	let Some((user_part, domain_part)) = email_str.split_once('@') else {
		return Err(PreProcessError {});
	};

	if !EMAIL_USER_RE.is_match(user_part) {
		return Err(PreProcessError {});
	}

	if validate_domain_part(domain_part, true, false).is_err() {
		// Still the possibility of an [IDN](https://en.wikipedia.org/wiki/Internationalized_domain_name)
		return match domain_to_ascii(domain_part) {
			Ok(d) => validate_domain_part(&d, true, false).map(|_| email),
			Err(_) => return Err(PreProcessError {}),
		};
	}

	Ok(email)
}

/// Validates whether the given string is a valid domain part of an email.
/// Returns an error if it is not, or the domain part if it is.
/// This function is used by [`validate_email`](fn.validate_email.html) to
/// validate the domain part of an email. It is exposed for use in other
/// contexts.
///
/// # Arguments
///
/// * `domain` - The domain to validate.
/// * `allow_ips` - Whether to allow IP addresses as the domain.
/// * `only_top_level` - Whether to only allow top level domains. If this value
///   is `true`, the domain must be a top level domain (e.g. `example.com` is
///   valid, but `hello.example.com` is not).
///
/// ```rust
/// use preprocess::validators::validate_domain_part;
///
/// pub fn main() {
/// 	let validated_domain_part = validate_domain_part("example.com");
/// 	assert_eq!(validated_domain_part, Ok("example.com"));
/// }
/// ```
#[must_use]
pub fn validate_domain_part<'a, S>(
	domain: S,
	allow_ips: bool,
	only_top_level: bool,
) -> Result<S, PreProcessError>
where
	S: Into<Cow<'a, str>>,
{
	let domain_str = domain.into();

	if !EMAIL_DOMAIN_RE.is_match(domain_str.as_ref()) {
		return Err(PreProcessError {});
	}

	if only_top_level {
		return Ok(domain);
	}

	if allow_ips {
		return validate_ip(domain_str).map(|_| domain);
	}

	Ok(domain)
}

#[cfg(test)]
mod tests {
	use crate::validators::validate_email;

	

	#[test]
	fn test_validate_email() {
		// Test cases taken from Django
		// https://github.com/django/django/blob/master/tests/validators/tests.py#L48
		let tests = vec![
			("email@here.com", true, true),
			("weirder-email@here.and.there.com", true, true),
			(r#"!def!xyz%abc@example.com"#, true, true),
			("email@[127.0.0.1]", true, true),
			("email@[2001:dB8::1]", true, true),
			("email@[2001:dB8:0:0:0:0:0:1]", true, true),
			("email@[::fffF:127.0.0.1]", true, true),
			("example@valid-----hyphens.com", true, true),
			("example@valid-with-hyphens.com", true, true),
			("test@domain.with.idn.tld.उदाहरण.परीक्षा", true, true),
			(r#""test@test"@example.com"#, true, false),
			// max length for domain name labels is 63 characters per RFC 1034
			("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", true, true),
			("a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.atm", true, true),
			(
				"a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbb.atm", true,
				true,
			),
			// 64 * a
			("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", true, false),
			("", true, false),
			("abc", true, false),
			("abc@", true, false),
			("abc@bar", true, true),
			("a @x.cz", true, false),
			("abc@.com", true, false),
			("something@@somewhere.com", true, false),
			("email@127.0.0.1", false, true),
			("email@[127.0.0.256]", false, false),
			("email@[2001:db8::12345]", false, false),
			("email@[2001:db8:0:0:0:0:1]", false, false),
			("email@[::ffff:127.0.0.256]", false, false),
			("example@invalid-.com", false, false),
			("example@-invalid.com", false, false),
			("example@invalid.com-", false, false),
			("example@inv-.alid-.com", false, false),
			("example@inv-.-alid.com", false, false),
			(r#"test@example.com\n\n<script src="x.js">"#, false, false),
			(r#""\\\011"@here.com"#, false, false),
			(r#""\\\012"@here.com"#, false, false),
			("trailingdot@shouldfail.com.", false, false),
			// Trailing newlines in username or domain not allowed
			("a@b.com\n", false, false),
			("a\n@b.com", false, false),
			(r#""test@test"\n@example.com"#, false, false),
			("a@[127.0.0.1]\n", false, false),
			// underscores are not allowed
			("John.Doe@exam_ple.com", false, false),
		];

		for (input, disallow_ips, expected) in tests {
			assert_eq!(
				validate_email(input, !disallow_ips).is_ok(),
				expected,
				"Email `{}` was not classified correctly",
				input
			);
		}
	}
}
