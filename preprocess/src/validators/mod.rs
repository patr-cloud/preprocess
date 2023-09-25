//! A list of all the validators that only validates the given field without
//! mutating it. The type of the field may still be changed. For example, the
//! [`ip`](crate::validators::validate_ip) validator will change the type
//! of the field to [`IpAddr`](std::net::IpAddr).
//!
//! # Contains
//!
//! The `contains` validator checks if the given value contains the given
//! substring using the [`Contains`](crate::validators::Contains) trait. By
//! default, this trait is implemented for the following types:
//!
//! - [`String`](std::string::String)
//! - [`&str`](str)
//! - [`Cow<'a, str>`](std::borrow::Cow)
//! - [`Vec<T>`](std::vec::Vec) where `T: Display`
//! - [`&[T]`](std::slice) where `T: Display`
//! - [`[T; N]`] where `T: Display` and `N` is any constant
//! - [`HashMap<K, V>`](std::collections::HashMap) where `K: Display`
//! - [`HashSet<T>`](std::collections::HashSet) where `T: Display`
//! - [`BTreeMap<K, V>`](std::collections::BTreeMap) where `K: Display`
//! - [`BTreeSet<T>`](std::collections::BTreeSet) where `T: Display`
//!
//! You can extend this trait to your own types by implementing the trait for
//! your type. For example, if you want to implement the trait for your own
//! struct `MyString`, you can do so by implementing the trait for the type:
//!
//! ```rust
//! use preprocess::validators::Contains;
//!
//! pub struct MyString(String);
//!
//! impl Contains for MyString {
//!     fn contains(&self, needle: &str) -> bool {
//!         self.0.to_string() == needle
//!     }
//! }
//! ```
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocess(contains = "foo")]
//!     pub my_string: String,
//! }
//! ```
//!
//! # Does Not Contain
//!
//! The `does_not_contain` validator checks if the given value does not contain
//! the given substring using the [`Contains`](crate::validators::Contains)
//! trait. This is the opposite of the
//! [`contains`](crate::validators::validate_contains) validator.
//! For all practical purposes, this validator is basically `!contains`.
//!
//! The `lowercase` preprocessor converts all the characters in the given value
//! to lowercase using the [`to_lowercase`](`str::to_lowercase`) method.
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocessor(lowercase)]
//!     pub my_string: String,
//! }
//! ```
//!
//! # Domain
//!
//! The `domain` validator checks if the given value is a valid domain name.
//! IP addresses are not allowed. To validate IP addresses, use the
//! [`validate_ip`](crate::validators::validate_ip) validator.
//!
//! Examples of valid domain names:
//!
//! - `google.com`
//! - `wikipedia.org`
//! - `stackoverflow.net`
//! - `mail.google.com`
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocess(domain)]
//!     pub domain: String,
//! }
//! ```
//!
//! # Email
//!
//! The `email` validator checks if the given value is a valid email address.
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocess(email)]
//!     pub email: String,
//! }
//! ```
//!
//! # IP
//!
//! The `ip` validator checks if the given value is a valid IP address. This
//! validator will change the type of the field to [`IpAddr`](std::net::IpAddr)
//! if the validation is successful. Both IPv4 and IPv6 addresses are supported.
//! For specific validation, use the
//! [`validate_ipv4`](crate::validators::validate_ipv4) and
//! [`validate_ipv6`](crate::validators::validate_ipv6) validators (using the
//! `ipv4` and `ipv6` preprocessor respectively).
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocess(ip)]
//!     pub ip: String, // This type will be changed to IpAddr
//! }
//! ```
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocessor(ipv4)]
//!     pub ipv4: String, // This type will be changed to Ipv4Addr
//! }
//! ```
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocessor(ipv6)]
//!     pub ipv6: String, // This type will be changed to Ipv6Addr
//! }
//! ```
//!
//! # Length
//!
//! The `length` validator checks if the length of the given value is within the
//! given range, or exactly equal to the given value. The length of the value is
//! calculated using the [`HasLen`](crate::validators::HasLen) trait. By
//! default, this trait is implemented for the following types:
//!
//! - [`String`](std::string::String)
//! - [`&str`](str)
//! - [`Cow<'a, str>`](std::borrow::Cow)
//! - [`Vec<T>`](std::vec::Vec) where `T: Display`
//! - [`&[T]`](std::slice) where `T: Display`
//! - [`[T; N]`] where `T: Display` and `N` is any constant
//! - [`HashMap<K, V>`](std::collections::HashMap) where `K: Display`
//! - [`HashSet<T>`](std::collections::HashSet) where `T: Display`
//! - [`BTreeMap<K, V>`](std::collections::BTreeMap) where `K: Display`
//! - [`BTreeSet<T>`](std::collections::BTreeSet) where `T: Display`
//!
//! You can extend this trait to your own types by implementing the trait for
//! your type. For example, if you want to implement the trait for your own
//! struct `MyString`, you can do so by implementing the trait for the type:
//!
//! ```rust
//! use preprocess::validators::HasLen;
//!
//! pub struct MyString(String);
//!
//! impl HasLen for MyString {
//!     fn length(&self) -> usize {
//!         self.0.to_string().len()
//!     }
//! }
//! ```
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocess(length(min = 5, max = 10))]
//!     pub my_string: String,
//! }
//! ```
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocess(length(equal = 5))]
//!     // You can also use #[preprocess(length = 5)] as a shorthand
//!     pub my_string: String,
//! }
//! ```
//!
//! __Note:__ At least one of `min`, `max` or `equal` must be specified.
//!
//! # Range
//!
//! The `range` validator checks if the given value is within the given range.
//! The range is exclusive of both the start and end values. The range is
//! checked using the [`PartialOrd`] trait.
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocess(range(min = 5, max = 10))]
//!     pub my_string: String,
//! }
//! ```
//!
//! # Regex
//!
//! The `regex` validator checks if the given value matches the given regular
//! expression. The regular expression is checked using the
//! [`Regex`](::regex::Regex) type. The regex is stored in a global, thread-safe
//! map to avoid recompiling the regex every time. This way, the regex is only
//! compiled once and then reused for every validation.
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocess(regex = r"^[a-zA-Z0-9]+$")]
//!     pub my_string: String,
//! }
//! ```
//!
//! # URL
//!
//! The `url` validator checks if the given value is a valid URL. This validator
//! will change the type of the field to [`Url`](::url::Url) if the validation
//! is successful.
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//!     #[preprocess(url)]
//!     pub url: String, // This type will be changed to Url
//! }
//! ```

mod contains;
mod does_not_contain;
mod domain;
mod email;
mod ip;
mod length;
mod range;
mod regex;
mod url;

pub use self::{
	contains::*,
	does_not_contain::*,
	domain::*,
	email::*,
	ip::*,
	length::*,
	range::*,
	regex::*,
	url::*,
};
