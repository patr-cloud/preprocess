//! # Validators
//! This module contains a set of preprocessors that are commonly used to
//! validate inputs, such as validating an email address, validating a phone
//! number, etc.
//!
//! ## The list of all available validators are below:

mod email;
mod ip;
mod ipv4;
mod ipv6;
mod length;
mod url;

pub use self::{
	email::{validate_domain_part, validate_email},
	ip::{validate_ip, IpAddrValidator},
	ipv4::{validate_ip_v4, Ipv4AddrValidator},
	ipv6::{validate_ip_v6, Ipv6AddrValidator},
	length::{
		validate_length,
		HasLength,
		LengthValidator,
		LengthValidatorArgs,
	},
	url::{validate_url, UrlValidator},
};
