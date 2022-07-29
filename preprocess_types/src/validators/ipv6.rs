use std::{fmt::Display, net::Ipv6Addr, str::FromStr};

use crate::{PreProcessError, PreProcessor};

/// Validates whether the given string is an IP V6
#[must_use]
pub fn validate_ip_v6<T: Display>(val: T) -> bool {
	Ipv6Addr::from_str(val.to_string().as_str()).is_ok()
}

/// Validator for whether the given string is an Ip Address. You can also choose
/// to parse the string as an Ipv4 or an Ipv6, by default it will be parsed as
/// Any (IPv4 or IPv6, whichever is valid).
///
/// ```rust
/// use preprocess::{validators::Ipv6AddrValidator, PreProcessor};
///
/// pub fn main() {
/// 	let v6: &str = "2001:db8::ff00:42:8329";
/// 	assert!(Ipv6AddrValidator::from(v6).preprocess().is_ok());
/// }
/// ```
#[must_use]
#[derive(Debug, Clone)]
pub struct Ipv6AddrValidator {
	data: String,
}

impl Ipv6AddrValidator {
	/// Validates whether the given string is a valid IpAddress or not.
	///
	/// ```rust
	/// use preprocess::{validators::Ipv6AddrValidator, PreProcessor};
	///
	/// pub fn main() {
	/// 	let v6: &str = "2001:db8::ff00:42:8329";
	/// 	assert!(Ipv6AddrValidator::from(v6).preprocess().is_ok());
	/// }
	pub fn validate(&self) -> bool {
		validate_ip_v6(&self.data)
	}
}

impl<Displaylike: Display> From<Displaylike> for Ipv6AddrValidator {
	/// Creates a new [`Ipv6AddrValidator`] from any struct that implements the
	/// [`Display`] trait.
	///
	/// ```rust
	/// use preprocess::validators::Ipv6AddrValidator;
	///
	/// pub fn main() {
	/// 	let validator = Ipv6AddrValidator::from("2001:db8::ff00:42:8329");
	/// 	assert_eq!(validator.validate(), true);
	/// }
	/// ```
	fn from(data: Displaylike) -> Self {
		Ipv6AddrValidator {
			data: data.to_string(),
		}
	}
}

impl PreProcessor for Ipv6AddrValidator {
	/// Can optionally mention the IpAddress type as an argument
	type Args = ();
	/// Returns a [`Ipv6Addr`] if the IpAddress is valid or an error if it is
	/// not.
	///
	/// ```rust
	/// use std::{net::Ipv6Addr, str::FromStr};
	///
	/// use preprocess::{validators::Ipv6AddrValidator, PreProcessor};
	///
	/// pub fn main() {
	/// 	let validated_ip: Ipv6Addr =
	/// 		Ipv6AddrValidator::from("2001:db8::ff00:42:8329")
	/// 			.preprocess()
	/// 			.unwrap();
	/// 	assert_eq!(
	/// 		validated_ip,
	/// 		Ipv6Addr::from([8193, 3512, 0, 0, 0, 65280, 66, 33577,])
	/// 	);
	/// }
	/// ```
	type Processed = Ipv6Addr;

	/// Validates whether the given string is a valid ip or not, returning an
	/// error if it is not, or a [`Ipv6Addr`] with the validated ip if it is.
	///
	/// ```rust
	/// use std::net::Ipv6Addr;
	///
	/// use preprocess::{
	/// 	validators::Ipv6AddrValidator,
	/// 	PreProcessError,
	/// 	PreProcessor,
	/// };
	///
	/// pub fn main() {
	/// 	let validated_ip: Result<Ipv6Addr, PreProcessError> =
	/// 		Ipv6AddrValidator::from("2001:db8::ff00:42:8329").preprocess();
	/// 	assert_eq!(validated_ip.is_ok(), true);
	/// }
	/// ```
	fn preprocess(self) -> Result<Ipv6Addr, PreProcessError> {
		if let Ok(ip_addr) = self.data.parse::<Ipv6Addr>() {
			Ok(ip_addr)
		} else {
			Err(PreProcessError {})
		}
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use super::validate_ip_v6;

	#[test]
	fn test_validate_ip_v6() {
		let tests = vec![
			("fe80::223:6cff:fe8a:2e8a", true),
			("2a02::223:6cff:fe8a:2e8a", true),
			("1::2:3:4:5:6:7", true),
			("::", true),
			("::a", true),
			("2::", true),
			("::ffff:254.42.16.14", true),
			("::ffff:0a0a:0a0a", true),
			("::254.42.16.14", true),
			("::0a0a:0a0a", true),
			("foo", false),
			("127.0.0.1", false),
			("12345::", false),
			("1::2::3::4", false),
			("1::zzz", false),
			("1:2", false),
			("fe80::223: 6cff:fe8a:2e8a", false),
			("2a02::223:6cff :fe8a:2e8a", false),
			("::ffff:999.42.16.14", false),
			("::ffff:zzzz:0a0a", false),
		];

		for (input, expected) in tests {
			assert_eq!(validate_ip_v6(input), expected);
		}
	}

	#[test]
	fn test_validate_ip_v6_cow() {
		let test: Cow<'static, str> = "fe80::223:6cff:fe8a:2e8a".into();
		assert!(validate_ip_v6(test));
		let test: Cow<'static, str> =
			String::from("fe80::223:6cff:fe8a:2e8a").into();
		assert!(validate_ip_v6(test));
		let test: Cow<'static, str> = "::ffff:zzzz:0a0a".into();
		assert!(!validate_ip_v6(test));
		let test: Cow<'static, str> = String::from("::ffff:zzzz:0a0a").into();
		assert!(!validate_ip_v6(test));
	}
}
