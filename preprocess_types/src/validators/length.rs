use std::{
	borrow::Cow,
	collections::{BTreeMap, BTreeSet, HashMap, HashSet},
};

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{PreProcessError, PreProcessor};

/// Trait to implement if one wants to make the `length` validator
/// work for more types
pub trait HasLength {
	fn length(&self) -> usize;
}

impl<H: HasLength> HasLength for &H {
	fn length(&self) -> usize {
		H::length(*self)
	}
}

impl HasLength for String {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl<'a> HasLength for &'a str {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl<'a> HasLength for Cow<'a, str> {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl<T> HasLength for Vec<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<K, V, S> HasLength for HashMap<K, V, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T, S> HasLength for HashSet<T, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<K, V> HasLength for BTreeMap<K, V> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> HasLength for BTreeSet<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<K, V> HasLength for IndexMap<K, V> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> HasLength for IndexSet<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

/// The arguments on how to validate the length of the data.
/// The min and max values are inclusive. The default values are minumum zero
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Serialize,
	Deserialize,
)]
#[serde(untagged, rename_all = "snake_case")]
pub enum LengthValidatorArgs {
	/// Validate to make sure the length is at greater than or equal to the
	/// given value
	Min {
		/// The minimum value to check against
		min: usize,
	},
	/// Validate to make sure the length is at less than or equal to the
	/// given value
	Max {
		/// The maximum value to check against
		max: usize,
	},
	/// Validate to make sure the length is in between or equal to the
	/// given values
	MinMax {
		/// The minimum value to check against
		min: usize,
		/// The maximum value to check against
		max: usize,
	},
	/// Validate to make sure the length is exactly the given value
	Exact {
		/// The value to check against
		exact: usize,
	},
}

impl Default for LengthValidatorArgs {
	fn default() -> Self {
		LengthValidatorArgs::Min { min: 0 }
	}
}

/// Validates whether the length is valid as per the given
/// [`LengthValidatorArgs`] or not
#[must_use]
pub fn validate_length(length: usize, args: LengthValidatorArgs) -> bool {
	match args {
		LengthValidatorArgs::Min { min } => length >= min,
		LengthValidatorArgs::Max { max } => length <= max,
		LengthValidatorArgs::MinMax { min, max } => {
			length >= min && length <= max
		}
		LengthValidatorArgs::Exact { exact } => length == exact,
	}
}

/// Validator for whether the length is valid as per the given
/// [`LengthValidatorArgs`]
///
/// ```rust
/// use preprocess::{
/// 	validators::{LengthValidator, LengthValidatorArgs},
/// 	PreProcessor,
/// };
///
/// pub fn main() {
/// 	let items = vec![1, 2, 3];
/// 	let mut validator = LengthValidator::from(items);
/// 	validator.set_args(LengthValidatorArgs::Min { min: 2 });
/// 	assert!(validator.preprocess().is_ok());
///
/// 	let string: &str = "hello";
/// 	let mut validator = LengthValidator::from(string);
/// 	validator.set_args(LengthValidatorArgs::Exact { exact: 4 });
/// 	assert!(validator.preprocess().is_err());
/// }
/// ```
#[must_use]
#[derive(Debug, Clone)]
pub struct LengthValidator<H: HasLength> {
	value: H,
	args: LengthValidatorArgs,
}

impl<H: HasLength> LengthValidator<H> {
	/// Validates whether the given length is valid or not.
	///
	/// ```rust
	/// use preprocess::{
	/// 	validators::{LengthValidator, LengthValidatorArgs},
	/// 	PreProcessor,
	/// };
	///
	/// pub fn main() {
	/// 	let string: &str = "hello";
	/// 	let mut validator = LengthValidator::from(string);
	/// 	validator.set_args(LengthValidatorArgs::Exact { exact: 4 });
	/// 	assert!(validator.preprocess().is_err());
	/// }
	pub fn validate(&self) -> bool {
		validate_length(self.value.length(), self.args)
	}
}

impl<H: HasLength> From<H> for LengthValidator<H> {
	/// Creates a new [`LengthValidator`] from any struct that implements the
	/// [`HasLength`] trait.
	///
	/// ```rust
	/// use preprocess::{
	/// 	validators::{LengthValidator, LengthValidatorArgs},
	/// 	PreProcessor,
	/// };
	///
	/// pub fn main() {
	/// 	let string: &str = "hello";
	/// 	let mut validator = LengthValidator::from(string);
	/// 	validator.set_args(LengthValidatorArgs::Exact { exact: 4 });
	/// 	assert!(validator.preprocess().is_err());
	/// }
	/// ```
	fn from(value: H) -> Self {
		LengthValidator {
			value,
			args: LengthValidatorArgs::default(),
		}
	}
}

impl<H: HasLength> PreProcessor for LengthValidator<H> {
	const TAKES_ARGS: bool = true;
	/// Requires the validator arguments
	type Args = LengthValidatorArgs;
	/// Returns the same item if the length is valid or an error if it is not.
	///
	/// ```rust
	/// use preprocess::{
	/// 	validators::{LengthValidator, LengthValidatorArgs},
	/// 	PreProcessor,
	/// };
	///
	/// pub fn main() {
	/// 	let string: &str = "hello";
	/// 	let mut validator = LengthValidator::from(string);
	/// 	validator.set_args(LengthValidatorArgs::Exact { exact: 5 });
	/// 	assert_eq!(validator.preprocess().unwrap(), "hello");
	/// }
	/// ```
	type Processed = H;

	/// Validates whether the given type has a valid length or not, returning an
	/// error if it is not, or the type itself if it is.
	///
	/// ```rust
	/// use preprocess::{
	/// 	validators::LengthValidator,
	/// 	PreProcessError,
	/// 	PreProcessor,
	/// };
	/// use url::Url;
	///
	/// pub fn main() {
	/// 	let validated_length: Result<Vec<u8>, PreProcessError> =
	/// 		LengthValidator::from(vec![1, 2, 3, 4, 5]).preprocess();
	/// 	assert_eq!(validated_length, Ok(vec![1, 2, 3, 4, 5]));
	/// }
	/// ```
	fn preprocess(self) -> Result<H, PreProcessError> {
		if self.validate() {
			Ok(self.value)
		} else {
			Err(PreProcessError {})
		}
	}

	fn get_args(&self) -> Self::Args {
		self.args
	}

	fn set_args(&mut self, args: Value) -> Result<(), PreProcessError> {
		self.args = serde_json::from_value(args).map_err(|_| PreProcessError {})?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use super::{validate_length, LengthValidatorArgs};

	#[test]
	fn test_validate_length_string_min_max() {
		assert!(validate_length(
			"hello".chars().count(),
			LengthValidatorArgs::MinMax { min: 1, max: 10 }
		));
	}

	#[test]
	fn test_validate_length_string_min_only() {
		assert!(!validate_length(
			"hello".chars().count(),
			LengthValidatorArgs::Min { min: 10 }
		));
	}

	#[test]
	fn test_validate_length_string_max_only() {
		assert!(!validate_length(
			"hello".chars().count(),
			LengthValidatorArgs::Max { max: 1 }
		));
	}

	#[test]
	fn test_validate_length_cow() {
		let test: Cow<'static, str> = "hello".into();
		assert!(validate_length(
			test.chars().count(),
			LengthValidatorArgs::Exact { exact: 5 }
		));

		let test: Cow<'static, str> = String::from("hello").into();
		assert!(validate_length(
			test.chars().count(),
			LengthValidatorArgs::Exact { exact: 5 }
		));
	}

	#[test]
	fn test_validate_length_vec() {
		assert!(validate_length(
			vec![1, 2, 3].len(),
			LengthValidatorArgs::Exact { exact: 3 }
		));
	}

	#[test]
	fn test_validate_length_unicode_chars() {
		assert!(validate_length(
			"日本".chars().count(),
			LengthValidatorArgs::Exact { exact: 2 }
		));
	}
}
