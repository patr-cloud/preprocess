use preprocess::prelude::*;
use serde::{Deserialize, Serialize};

#[preprocess::sync]
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
	#[preprocess(trim, lowercase, length(max = 64), regex = "^[a-z0-9_]+$")]
	pub username: String,
	#[preprocess(trim, length(min = 8), regex = "^[a-z0-9_]+$")]
	pub password: String,
	#[preprocess]
	pub nested: EnumRequest,
}

#[preprocess::sync]
#[derive(Debug, Deserialize, Serialize)]
pub enum EnumRequest {
	#[allow(dead_code)]
	VariantA {
		#[preprocess(
			trim,
			lowercase,
			length(max = 64),
			regex = "^[a-z0-9_]+$"
		)]
		username: String,
		#[preprocess(trim, length(min = 8), regex = "^[a-z0-9_]+$")]
		password: String,
		#[preprocess(optional(trim))]
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
