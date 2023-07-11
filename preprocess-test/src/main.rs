use serde::{Deserialize, Serialize};

#[preprocess::sync]
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
	#[preprocess(trim, lowercase, length(max = 64), regex = "^[a-z0-9_]+$")]
	pub username: String,
	#[preprocess(trim, length(min = 8), regex = "^[a-z0-9_]+$")]
	pub password: String,
}

fn main() {
	println!("Hello, world!");
}
