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

// Regression: `#[cfg]` on fields must ride along onto the destructure pattern
// and per-field preprocessor statements in the generated `preprocess()` body.
// Uses `cfg(any())` / `cfg(not(any()))` so the test is deterministic on every
// platform without needing a feature flag.

/// Cfg-gated regression struct.
#[preprocess::sync]
#[derive(Debug, Deserialize, Serialize)]
pub struct CfgPath {
	#[cfg(any())]
	/// The `cfg`-disabled branch — never compiled in this build, but the macro
	/// must still emit a destructure pattern that accommodates it.
	pub workspace_id: u64,
	#[cfg(not(any()))]
	#[preprocess(trim, length(min = 1))]
	/// The `cfg`-enabled branch.
	pub workspace_id: String,
	#[preprocess(trim)]
	/// Always-on field, exercises the non-cfg path alongside.
	pub repo_name: String,
}

/// Cfg-gated regression enum variant.
#[preprocess::sync]
#[derive(Debug, Deserialize, Serialize)]
pub enum CfgEnum {
	#[allow(dead_code)]
	/// Cfg-gated regression variant.
	Variant {
		#[cfg(any())]
		/// Disabled branch — never present in this build.
		unused: u64,
		#[cfg(not(any()))]
		#[preprocess(trim)]
		/// Enabled branch.
		name: String,
	},
}

/// Generic wrapper regression: nested `#[preprocess]` on a type-param field.
/// Pre-fix the macro emitted a bogus `QProcessed` field type instead of
/// projecting through `<Q as Preprocessable>::Processed`.
#[preprocess::sync]
pub struct Wrapper<Q: Preprocessable = ()> {
	#[preprocess(range(min = 1, max = 100))]
	/// A plain validated field alongside the generic one.
	pub count: usize,
	#[preprocess]
	/// The generic payload, preprocessed through the trait projection.
	pub inner: Q,
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

	let _cfg_path: CfgPathProcessed = Preprocessable::preprocess(CfgPath {
		workspace_id: "  abc  ".to_string(),
		repo_name: "  repo  ".to_string(),
	})
	.unwrap();

	let _cfg_enum: CfgEnumProcessed =
		Preprocessable::preprocess(CfgEnum::Variant {
			name: "  hello  ".to_string(),
		})
		.unwrap();

	let _wrapper: WrapperProcessed<LoginRequest> =
		Preprocessable::preprocess(Wrapper {
			count: 42,
			inner: LoginRequest {
				username: "  HelloWorld  ".to_string(),
				password: "  HelloWorld  ".to_string(),
				nested: EnumRequest::VariantA {
					username: "  HelloWorld  ".to_string(),
					password: "  HelloWorld  ".to_string(),
					optional: Some("  HelloWorld  ".to_string()),
				},
			},
		})
		.unwrap();

	println!("Hello, world!");
}
