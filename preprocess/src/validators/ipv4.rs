use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

use crate::{PreProcessError, PreProcessor};

/// Validates whether the given string is an IP V4
#[must_use]
pub fn validate_ip_v4<T: Display>(val: T) -> bool {
	Ipv4Addr::from_str(val.to_string().as_str()).is_ok()
}

/// Validator for whether the given string is an Ip Address. You can also choose
/// to parse the string as an Ipv4 or an Ipv6, by default it will be parsed as
/// Any (IPv4 or IPv6, whichever is valid).
///
/// ```rust
/// use preprocess::{validators::Ipv4AddrValidator, PreProcessor};
///
/// pub fn main() {
/// 	let v4: &str = "192.168.1.2";
/// 	assert!(Ipv4AddrValidator::from(v4).preprocess().is_ok());
/// }
/// ```
#[must_use]
#[derive(Debug, Clone)]
pub struct Ipv4AddrValidator {
	data: String,
}

impl Ipv4AddrValidator {
	/// Validates whether the given string is a valid IpAddress or not.
	///
	/// ```rust
	/// use preprocess::{validators::Ipv4AddrValidator, PreProcessor};
	///
	/// pub fn main() {
	/// 	let ip: &str = "192.168.1.2";
	/// 	assert!(Ipv4AddrValidator::from(ip).preprocess().is_ok());
	/// }
	pub fn validate(&self) -> bool {
		validate_ip_v4(&self.data)
	}
}

impl<Displaylike: Display> From<Displaylike> for Ipv4AddrValidator {
	/// Creates a new [`Ipv4AddrValidator`] from any struct that implements the
	/// [`Display`] trait.
	///
	/// ```rust
	/// use preprocess::validators::Ipv4AddrValidator;
	///
	/// pub fn main() {
	/// 	let validator = Ipv4AddrValidator::from("192.168.1.3");
	/// 	assert_eq!(validator.validate(), true);
	/// }
	/// ```
	fn from(data: Displaylike) -> Self {
		Ipv4AddrValidator {
			data: data.to_string(),
		}
	}
}

impl PreProcessor for Ipv4AddrValidator {
	/// Can optionally mention the IpAddress type as an argument
	type Args = ();
	/// Returns a [`Ipv4Addr`] if the IpAddress is valid or an error if it is
	/// not.
	///
	/// ```rust
	/// use std::net::Ipv4Addr;
	///
	/// use preprocess::{validators::Ipv4AddrValidator, PreProcessor};
	///
	/// pub fn main() {
	/// 	let validated_ip: Ipv4Addr =
	/// 		Ipv4AddrValidator::from("192.168.1.4").preprocess().unwrap();
	/// 	assert_eq!(validated_ip, Ipv4Addr::from([192, 168, 1, 4]));
	/// }
	/// ```
	type Processed = Ipv4Addr;

	/// Validates whether the given string is a valid ip or not, returning an
	/// error if it is not, or a [`Ipv4Addr`] with the validated ip if it is.
	///
	/// ```rust
	/// use std::net::Ipv4Addr;
	///
	/// use preprocess::{
	/// 	validators::Ipv4AddrValidator,
	/// 	PreProcessError,
	/// 	PreProcessor,
	/// };
	///
	/// pub fn main() {
	/// 	let validated_ip: Result<Ipv4Addr, PreProcessError> =
	/// 		Ipv4AddrValidator::from("192.168.1.5").preprocess();
	/// 	assert_eq!(validated_ip, Ok(Ipv4Addr::from([192, 168, 1, 5])));
	/// }
	/// ```
	fn preprocess(self) -> Result<Ipv4Addr, PreProcessError> {
		Ipv4Addr::from_str(self.data.as_str()).map_err(|_| PreProcessError {})
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use super::validate_ip_v4;

	#[test]
	fn test_validate_ip_v4() {
		let tests = vec![
			("1.1.1.1", true),
			("255.0.0.0", true),
			("0.0.0.0", true),
			("256.1.1.1", false),
			("25.1.1.", false),
			("25,1,1,1", false),
			("25.1 .1.1", false),
			("1.1.1.1\n", false),
			("٧.2٥.3٣.243", false),
		];

		for (input, expected) in tests {
			assert_eq!(validate_ip_v4(input), expected);
		}
	}

	#[test]
	fn test_validate_ip_v4_cow() {
		let test: Cow<'static, str> = "1.1.1.1".into();
		assert!(validate_ip_v4(test));
		let test: Cow<'static, str> = String::from("1.1.1.1").into();
		assert!(validate_ip_v4(test));
		let test: Cow<'static, str> = "٧.2٥.3٣.243".into();
		assert!(!validate_ip_v4(test));
		let test: Cow<'static, str> = String::from("٧.2٥.3٣.243").into();
		assert!(!validate_ip_v4(test));
	}
}
