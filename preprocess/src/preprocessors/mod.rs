//! A list of all the preprocessors that preprocess the given field, mutating it
//! if required. The type of the field may be changed. For example, the
//! [`lowercase`](crate::preprocessors::preprocess_lowercase) preprocessor will
//! change the type of the field to [`String`].
//!
//! # Lowercase
//!
//! The `lowercase` preprocessor converts all the characters in the given value
//! to lowercase using the [`to_lowercase`](`str::to_lowercase`) method.
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//! 	#[preprocess(lowercase)]
//! 	pub my_string: String,
//! }
//! ```
//!
//! # Uppercase
//!
//! The `uppercase` preprocessor converts all the characters in the given value
//! to uppercase using the [`to_uppercase`](`str::to_uppercase`) method.
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//! 	#[preprocess(uppercase)]
//! 	pub my_string: String,
//! }
//! ```
//!
//! # Trim
//!
//! The `trim` preprocessor trims the given value using the
//! [`trim`](`str::trim`) method.
//!
//! ## Usage
//!
//! ```rust
//! #[preprocess::sync]
//! pub struct MyStruct {
//! 	#[preprocess(trim)]
//! 	pub my_string: String,
//! }
//! ```

mod lowercase;
mod trim;
mod uppercase;

pub use self::{lowercase::*, trim::*, uppercase::*};
