use preprocess::PreProcess;

#[derive(PreProcess)]
pub struct SignUpRequest {
	#[preprocess(email)]
	username: String,
	#[preprocess(length(min = 4, max = 64), email)]
	password: String,
	user_id: String,
	#[preprocess]
	test: Test,
}

#[derive(PreProcess)]
pub struct SignInRequest {
	user_id: String,
	#[preprocess(length(min = 4, max = 64), email)]
	password: String,
}

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
