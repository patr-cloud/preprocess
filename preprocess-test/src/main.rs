//! Test for the `preprocess` crate.

use preprocess::prelude::*;
use serde::{Deserialize, Serialize};

/// Random test struct
#[preprocess::sync]
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
	#[preprocess(trim, lowercase, length(max = 64), regex = "^[a-z0-9_]+$")]
	/// Random test field
	pub username: String,
	#[preprocess(trim, length(min = 8), regex = "^[a-z0-9_]+$")]
	/// Random test field
	pub password: String,
	#[preprocess]
	/// Random test field
	pub nested: EnumRequest,
}

/// Random test enum
#[preprocess::sync]
#[derive(Debug, Deserialize, Serialize)]
pub enum EnumRequest {
	#[allow(dead_code)]
	/// Random test variant
	VariantA {
		#[preprocess(
			trim,
			lowercase,
			length(max = 64),
			regex = "^[a-z0-9_]+$"
		)]
		/// Random test field
		username: String,
		#[preprocess(trim, length(min = 8), regex = "^[a-z0-9_]+$")]
		/// Random test field
		password: String,
		#[preprocess(optional(trim))]
		/// Random test field
		optional: Option<String>,
	},
}

fn main() {
	let _processed: LoginRequestProcessed =
		Preprocessable::preprocess(LoginRequest {
			username: "  HelloWorld  ".to_string(),
			password: "  HelloWorld  ".to_string(),
			nested: EnumRequest::VariantA {
				username: "  HelloWorld  ".to_string(),
				password: "  HelloWorld  ".to_string(),
				optional: Some("  HelloWorld  ".to_string()),
			},
		})
		.unwrap();
	println!("Hello, world!");
}
