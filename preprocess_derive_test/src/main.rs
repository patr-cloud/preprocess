use preprocess::{validators::EmailValidator, PreProcessor};
use preprocess_derive::Preprocess;
use serde::{Deserialize, Serialize};

#[derive(Preprocess)]
pub struct SignUpRequest {
	#[preprocess(email)]
	username: String,
	#[preprocess(length(min = 4, max = 64), email)]
	password: String,
	user_id: String,
}

#[derive(Preprocess)]
pub struct SignInRequest {
	user_id: String,
	#[preprocess(length(min = 4, max = 64), email)]
	password: String,
}

#[allow(dead_code)]
pub struct Processed {
	username: <EmailValidator as PreProcessor>::Processed,
}

pub struct Unnamed(String, usize, f64);

#[derive(Preprocess)]
pub enum Test {
	VariantA {
		#[preprocess(email)]
		field1: String,
		field2: String,
	},
	VariantB {
		#[preprocess(length(min = 4, max = 64), email)]
		field3: String,
		field4: String,
	},
}

pub fn main() {}
