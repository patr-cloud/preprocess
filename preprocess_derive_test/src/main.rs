use serde::Serialize;

#[preprocess::preprocess(custom = "test")]
#[derive(Serialize)]
pub struct UnitStruct;

#[preprocess::preprocess]
pub struct SignUpRequest {
	#[preprocess::preprocess(email)]
	pub username: String,
	#[preprocess::preprocess(length(min = 4, max = 64), email)]
	pub password: String,
	pub user_id: String,
	// #[preprocess::preprocess]
	// test: Test,
}

pub struct SignInRequest {
	user_id: String,
	#[preprocess::preprocess(length(min = 4, max = 64), email)]
	password: String,
}

pub struct Unnamed(String, usize, f64);

pub enum Test {
	VariantA {
		#[preprocess::preprocess(email)]
		field1: String,
		field2: String,
	},
	VariantB {
		#[preprocess::preprocess(email, length(min = 4, max = 64))]
		field3: String,
		field4: String,
	},
}

pub fn main() {}
