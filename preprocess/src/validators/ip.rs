use std::{
	borrow::Cow,
	net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use crate::utils::Error;

/// Checks if a given string is a valid ip address or not
#[must_use]
pub fn validate_ip<'a, T>(domain: T) -> Result<IpAddr, Error>
where
	T: Into<Cow<'a, str>>,
{
	domain
		.into()
		.parse()
		.map_err(|err| Error::new(format!("invalid ip address: {}", err)))
}

/// Checks if a given string is a valid ipv4 address or not
#[must_use]
pub fn validate_ipv4<'a, T>(domain: T) -> Result<Ipv4Addr, Error>
where
	T: Into<Cow<'a, str>>,
{
	domain
		.into()
		.parse()
		.map_err(|err| Error::new(format!("invalid ipv4 address: {}", err)))
}

/// Checks if a given string is a valid ipv6 address or not
#[must_use]
pub fn validate_ipv6<'a, T>(domain: T) -> Result<Ipv6Addr, Error>
where
	T: Into<Cow<'a, str>>,
{
	domain
		.into()
		.parse()
		.map_err(|err| Error::new(format!("invalid ip address: {}", err)))
}
