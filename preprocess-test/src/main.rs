use serde::{Deserialize, Serialize};

#[preprocess::process]
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
	pub username: String,
	pub password: String,
}

fn main() {
	println!("Hello, world!");
}
