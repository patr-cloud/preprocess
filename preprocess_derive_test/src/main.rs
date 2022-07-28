use preprocess::{validators::EmailValidator, PreProcessor};
use preprocess_derive::Preprocess;
use serde::{Deserialize, Serialize};

#[derive(Preprocess, Serialize, Deserialize)]
#[preprocess(custom = "preprocess_data")]
#[preprocess(
	length(min = 4, max = 64),
	regex(pattern = "^[a-zA-Z0-9]+$", test = "is_alphanumeric")
)]
#[preprocess(userid)]
#[serde(rename = "test")]
pub struct SignUpRequest {
	#[preprocess(email)]
	username: String,
	#[preprocess(length(min = 4, max = 64), regex = "^[a-zA-Z0-9]+$")]
	password: String,
	#[preprocess(userid(not_nil))]
	user_id: String,
}

#[derive(Preprocess, Serialize, Deserialize)]
#[preprocess(custom = "preprocess_data")]
#[preprocess]
#[serde(rename = "test")]
pub struct SignInRequest {
	#[preprocess(custom = "email_or_username_or_phone_validator")]
	user_id: String,
	#[preprocess(length(min = 4, max = 64), regex = "^[a-zA-Z0-9]+$")]
	password: String,
}

#[allow(dead_code)]
pub struct Processed {
	username: <EmailValidator as PreProcessor>::Processed,
}

pub fn main() {}
