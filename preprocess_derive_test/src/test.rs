#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use preprocess::{validators::EmailValidator, PreProcessor};
use preprocess_derive::Preprocess;
use serde::{Deserialize, Serialize};
#[preprocess(custom = "preprocess_data")]
#[preprocess(
    length(min = 4, max = 64),
    regex(pattern = "^[a-zA-Z0-9]+$", test = "is_alphanumeric")
)]
#[preprocess(userid)]
pub struct SignUpRequest {
    #[preprocess(email)]
    username: String,
    #[preprocess(length(min = 4, max = 64), regex(pattern = "^[a-zA-Z0-9]+$"))]
    password: String,
    #[preprocess(userid)]
    user_id: String,
}
impl preprocess::PreProcessor for SignUpRequest {
    type Args = ();
    type Processed = SignUpRequestProcessed;
    fn preprocess(self) -> Result<Self::Processed, preprocess::PreProcessError> {
        let SignUpRequest {
            username,
            password,
            user_id,
        } = self;
        let username = {
            let preprocessor = email::from(username);
            preprocessor.preprocess()?
        };
        let preprocessed_password_0 = {
            let mut preprocessor = length::from(password);
            preprocessor.set_args(serde_json::Value::Object({
                let mut map = serde_json::Map::new();
                map.insert("max".to_string(), serde_json::Value::Number(64u64.into()));
                map.insert("min".to_string(), serde_json::Value::Number(4u64.into()));
                map
            }));
            preprocessor.preprocess()?
        };
        let password = {
            let mut preprocessor = regex::from(preprocessed_password_0);
            preprocessor.set_args(serde_json::Value::Object({
                let mut map = serde_json::Map::new();
                map.insert(
                    "pattern".to_string(),
                    serde_json::Value::String("^[a-zA-Z0-9]+$".to_string()),
                );
                map
            }));
            preprocessor.preprocess()?
        };
        let user_id = {
            let preprocessor = userid::from(user_id);
            preprocessor.preprocess()?
        };
        Ok(SignUpRequestProcessed {
            username,
            password,
            user_id,
        })
    }
}
pub struct SignUpRequestProcessed {
    username: <email as preprocess::PreProcessor>::Processed,
    password: <regex as preprocess::PreProcessor>::Processed,
    user_id: <userid as preprocess::PreProcessor>::Processed,
}
#[preprocess(custom = "preprocess_data")]
#[preprocess]
pub struct SignInRequest {
    #[preprocess(custom = "email_or_username_or_phone_validator")]
    user_id: String,
    #[preprocess(length(min = 4, max = 64), regex(pattern = "^[a-zA-Z0-9]+$"))]
    password: String,
}
impl preprocess::PreProcessor for SignInRequest {
    type Args = ();
    type Processed = SignInRequestProcessed;
    fn preprocess(self) -> Result<Self::Processed, preprocess::PreProcessError> {
        let SignInRequest { user_id, password } = self;
        let user_id = {
            let mut preprocessor = custom::from(user_id);
            preprocessor.set_args(serde_json::Value::Object({
                let mut map = serde_json::Map::new();
                map.insert(
                    "function".to_string(),
                    serde_json::Value::String("email_or_username_or_phone_validator".to_string()),
                );
                map
            }));
            preprocessor.preprocess()?
        };
        let preprocessed_password_0 = {
            let mut preprocessor = length::from(password);
            preprocessor.set_args(serde_json::Value::Object({
                let mut map = serde_json::Map::new();
                map.insert("max".to_string(), serde_json::Value::Number(64u64.into()));
                map.insert("min".to_string(), serde_json::Value::Number(4u64.into()));
                map
            }));
            preprocessor.preprocess()?
        };
        let password = {
            let mut preprocessor = regex::from(preprocessed_password_0);
            preprocessor.set_args(serde_json::Value::Object({
                let mut map = serde_json::Map::new();
                map.insert(
                    "pattern".to_string(),
                    serde_json::Value::String("^[a-zA-Z0-9]+$".to_string()),
                );
                map
            }));
            preprocessor.preprocess()?
        };
        Ok(SignInRequestProcessed { user_id, password })
    }
}
pub struct SignInRequestProcessed {
    user_id: <custom as preprocess::PreProcessor>::Processed,
    password: <regex as preprocess::PreProcessor>::Processed,
}
#[allow(dead_code)]
pub struct Processed {
    username: <EmailValidator as PreProcessor>::Processed,
}
pub struct Unnamed(String, usize, f64);
#[preprocess(custom = "preprocess_data")]
pub enum Test {
    VariantA {
        #[preprocess(email)]
        field1: String,
        #[preprocess(custom = "email_or_username_or_phone_validator")]
        field2: String,
    },
    VariantB {
        #[preprocess(length(min = 4, max = 64), regex(pattern = "^[a-zA-Z0-9]+$"))]
        field3: String,
        field4: String,
    },
}
impl preprocess::PreProcessor for Test {
    type Args = ();
    type Processed = TestProcessed;
    fn preprocess(self) -> Result<Self::Processed, preprocess::PreProcessError> {
        match self {
            Test::VariantA { field1, field2 } => {
                let field1 = {
                    let preprocessor = email::from(field1);
                    preprocessor.preprocess()?
                };
                let field2 = {
                    let mut preprocessor = custom::from(field2);
                    preprocessor.set_args(serde_json::Value::Object({
                        let mut map = serde_json::Map::new();
                        map.insert(
                            "function".to_string(),
                            serde_json::Value::String(
                                "email_or_username_or_phone_validator".to_string(),
                            ),
                        );
                        map
                    }));
                    preprocessor.preprocess()?
                };
                Ok(TestProcessed::VariantA { field1, field2 })
            }
            Test::VariantB { field3, field4 } => {
                let preprocessed_field3_0 = {
                    let mut preprocessor = length::from(field3);
                    preprocessor.set_args(serde_json::Value::Object({
                        let mut map = serde_json::Map::new();
                        map.insert("max".to_string(), serde_json::Value::Number(64u64.into()));
                        map.insert("min".to_string(), serde_json::Value::Number(4u64.into()));
                        map
                    }));
                    preprocessor.preprocess()?
                };
                let field3 = {
                    let mut preprocessor = regex::from(preprocessed_field3_0);
                    preprocessor.set_args(serde_json::Value::Object({
                        let mut map = serde_json::Map::new();
                        map.insert(
                            "pattern".to_string(),
                            serde_json::Value::String("^[a-zA-Z0-9]+$".to_string()),
                        );
                        map
                    }));
                    preprocessor.preprocess()?
                };
                Ok(TestProcessed::VariantB { field3, field4 })
            }
        }
    }
}
pub enum TestProcessed {
    VariantA {
        field1: <email as preprocess::PreProcessor>::Processed,
        field2: <custom as preprocess::PreProcessor>::Processed,
    },
    VariantB {
        field3: <regex as preprocess::PreProcessor>::Processed,
        field4: String,
    },
}
pub fn main() {}
