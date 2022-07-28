use std::{fmt::Display, net::IpAddr, str::FromStr};

use crate::{PreProcessError, PreProcessor};

/// Validates whether the given string is an IP
#[must_use]
pub fn validate_ip<T: Display>(val: T) -> bool {
	IpAddr::from_str(val.to_string().as_str()).is_ok()
}

/// Validator for whether the given string is an Ip Address. You can also choose
/// to parse the string as an Ipv4 or an Ipv6, by default it will be parsed as
/// Any (IPv4 or IPv6, whichever is valid).
///
/// ```rust
/// use preprocess::{validators::IpAddrValidator, PreProcessor};
///
/// pub fn main() {
/// 	let either: &str = "192.168.1.1";
/// 	assert!(IpAddrValidator::from(either).preprocess().is_ok());
///
/// 	let v4: &str = "192.168.1.2";
/// 	assert!(IpAddrValidator::from(v4).preprocess().is_ok());
///
/// 	let v6: &str = "2001:db8::ff00:42:8329";
/// 	assert!(IpAddrValidator::from(v6).preprocess().is_ok());
/// }
/// ```
#[must_use]
#[derive(Debug, Clone)]
pub struct IpAddrValidator {
	data: String,
}

impl IpAddrValidator {
	/// Validates whether the given string is a valid IpAddress or not.
	///
	/// ```rust
	/// use preprocess::{validators::IpAddrValidator, PreProcessor};
	///
	/// pub fn main() {
	/// 	let ip: &str = "192.168.1.2";
	/// 	assert!(IpAddrValidator::from(ip).preprocess().is_ok());
	/// }
	pub fn validate(&self) -> bool {
		IpAddr::from_str(self.data.as_str()).is_ok()
	}
}

impl<Displaylike: Display> From<Displaylike> for IpAddrValidator {
	/// Creates a new [`IpAddrValidator`] from any struct that implements the
	/// [`Display`] trait.
	///
	/// ```rust
	/// use preprocess::validators::IpAddrValidator;
	///
	/// pub fn main() {
	/// 	let validator = IpAddrValidator::from("192.168.1.3");
	/// 	assert_eq!(validator.validate(), true);
	/// }
	/// ```
	fn from(data: Displaylike) -> Self {
		IpAddrValidator {
			data: data.to_string(),
		}
	}
}

impl PreProcessor for IpAddrValidator {
	/// Can optionally mention the IpAddress type as an argument
	type Args = ();
	/// Returns a [`IpAddr`] if the IpAddress is valid or an error if it is not.
	///
	/// ```rust
	/// use std::net::IpAddr;
	///
	/// use preprocess::{validators::IpAddrValidator, PreProcessor};
	///
	/// pub fn main() {
	/// 	let validated_ip: IpAddr =
	/// 		IpAddrValidator::from("192.168.1.4").preprocess().unwrap();
	/// 	assert_eq!(validated_ip, IpAddr::from([192, 168, 1, 4]));
	/// }
	/// ```
	type Processed = IpAddr;

	/// Validates whether the given string is a valid ip or not, returning an
	/// error if it is not, or a [`IpAddr`] with the validated ip if it is.
	///
	/// ```rust
	/// use std::net::IpAddr;
	///
	/// use preprocess::{
	/// 	validators::IpAddrValidator,
	/// 	PreProcessError,
	/// 	PreProcessor,
	/// };
	///
	/// pub fn main() {
	/// 	let validated_ip: Result<IpAddr, PreProcessError> =
	/// 		IpAddrValidator::from("192.168.1.5").preprocess();
	/// 	assert_eq!(validated_ip, Ok(IpAddr::from([192, 168, 1, 5])));
	/// }
	/// ```
	fn preprocess(self) -> Result<IpAddr, PreProcessError> {
		IpAddr::from_str(self.data.as_str()).map_err(|_| PreProcessError {})
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use super::validate_ip;

	#[test]
	fn test_validate_ip() {
		let tests = vec![
			("1.1.1.1", true),
			("255.0.0.0", true),
			("0.0.0.0", true),
			("256.1.1.1", false),
			("25.1.1.", false),
			("25,1,1,1", false),
			("fe80::223:6cff:fe8a:2e8a", true),
			("::ffff:254.42.16.14", true),
			("2a02::223:6cff :fe8a:2e8a", false),
		];

		for (input, expected) in tests {
			assert_eq!(validate_ip(input), expected);
		}
	}

	#[test]
	fn test_validate_ip_cow() {
		let test: Cow<'static, str> = "1.1.1.1".into();
		assert!(validate_ip(test));
		let test: Cow<'static, str> = String::from("1.1.1.1").into();
		assert!(validate_ip(test));
		let test: Cow<'static, str> = "2a02::223:6cff :fe8a:2e8a".into();
		assert!(!validate_ip(test));
		let test: Cow<'static, str> =
			String::from("2a02::223:6cff :fe8a:2e8a").into();
		assert!(!validate_ip(test));
	}
}
