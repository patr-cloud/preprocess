use preprocess::PreProcess;
use serde::Serialize;

#[derive(PreProcess)]
#[preprocess(custom = "test")]
pub struct UnitStruct;

#[derive(PreProcess)]
pub struct SignUpRequest {
	#[preprocess(email)]
	pub username: String,
	#[preprocess(length(min = 4, max = 64), email)]
	pub password: String,
	pub user_id: String,
	// #[preprocess]
	// test: Test,
}

#[derive(PreProcess)]
pub struct SignInRequest {
	user_id: String,
	#[preprocess(length(min = 4, max = 64), email)]
	password: String,
}

#[derive(PreProcess)]
pub struct Unnamed(String, usize, f64);

#[derive(PreProcess)]
pub enum Test {
	VariantA {
		#[preprocess(email)]
		field1: String,
		field2: String,
	},
	VariantB {
		#[preprocess(email, length(min = 4, max = 64))]
		field3: String,
		field4: String,
	},
}

pub fn main() {}
