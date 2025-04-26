use std::{
	borrow::Cow,
	collections::{BTreeMap, BTreeSet, HashMap, HashSet},
};

use crate::utils::Error;

/// Trait to get the length of a value
/// This is used by the `validate_length` validator
/// to get the length of the value given
///
/// Implement this trait for your own types if you want to use the
/// `validate_length` validator
pub trait HasLen {
	/// Returns the length of the value
	fn length(&self) -> usize;
}

impl HasLen for String {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl HasLen for &String {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl HasLen for &str {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl HasLen for Cow<'_, str> {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl<T> HasLen for Vec<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> HasLen for &Vec<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> HasLen for &[T] {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T, const N: usize> HasLen for [T; N] {
	fn length(&self) -> usize {
		N
	}
}

impl<T, const N: usize> HasLen for &[T; N] {
	fn length(&self) -> usize {
		N
	}
}

impl<K, V, S> HasLen for &HashMap<K, V, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<K, V, S> HasLen for HashMap<K, V, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T, S> HasLen for &HashSet<T, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T, S> HasLen for HashSet<T, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<K, V> HasLen for &BTreeMap<K, V> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<K, V> HasLen for BTreeMap<K, V> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> HasLen for &BTreeSet<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> HasLen for BTreeSet<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

/// Validates the length of the value given.
/// If the validator has `equal` set, it will ignore any `min` and `max` value.
///
/// If you apply it on String, don't forget that the length can be different
/// from the number of visual characters for Unicode
#[must_use = concat!(
	"validation returns a new value instead of mutating the input.",
	" The returned value will contain the validated value,",
	" while the input will remain unchanged"
)]
pub fn validate_length<T: HasLen>(
	value: T,
	min: Option<usize>,
	max: Option<usize>,
	equal: Option<usize>,
) -> Result<T, Error> {
	let val_length = value.length();

	if let Some(m) = equal {
		if val_length != m {
			return Err(Error::new(format!("length must be equal to {}", m)));
		}
	}

	if let Some(m) = min {
		if val_length < m {
			return Err(Error::new(format!(
				"length must be greater than or equal to {}",
				m
			)));
		}
	}
	if let Some(m) = max {
		if val_length > m {
			return Err(Error::new(format!(
				"length must be less than or equal to {}",
				m
			)));
		}
	}

	Ok(value)
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use super::validate_length;

	#[test]
	fn test_validate_length_equal_overrides_min_max() {
		assert!(validate_length("hello", Some(1), Some(2), Some(5)).is_err());
	}

	#[test]
	fn test_validate_length_string_min_max() {
		assert!(validate_length("hello", Some(1), Some(10), None).is_ok());
	}

	#[test]
	fn test_validate_length_string_min_only() {
		assert!(validate_length("hello", Some(10), None, None).is_err());
	}

	#[test]
	fn test_validate_length_string_max_only() {
		assert!(validate_length("hello", None, Some(1), None).is_err());
	}

	#[test]
	fn test_validate_length_cow() {
		let test: Cow<'static, str> = "hello".into();
		assert!(validate_length(test, None, None, None).is_ok());

		let test: Cow<'static, str> = String::from("hello").into();
		assert!(validate_length(test, None, None, Some(5)).is_ok());
	}

	#[test]
	fn test_validate_length_vec() {
		assert!(validate_length(vec![1, 2, 3], None, None, Some(3)).is_ok());
	}

	#[test]
	fn test_validate_length_unicode_chars() {
		assert!(validate_length("日本", None, None, Some(2)).is_ok());
	}
}
