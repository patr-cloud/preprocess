use std::{
	borrow::Cow,
	collections::{BTreeMap, BTreeSet, HashMap, HashSet},
	fmt::Display,
};

use crate::utils::Error;

/// Trait to check if a value contains a given needle
/// This is used by the `validate_contains` validator
/// to check if the needle is inside the given value
///
/// Implement this trait for your own types if you want to use the
/// `validate_contains` validator
pub trait Contains {
	/// Checks if the value contains the given needle
	#[must_use]
	fn contains(&self, needle: &str) -> bool;
}

impl Contains for String {
	fn contains(&self, needle: &str) -> bool {
		self.matches(needle).count() > 0
	}
}

impl<'a> Contains for &'a String {
	fn contains(&self, needle: &str) -> bool {
		self.matches(needle).count() > 0
	}
}

impl<'a> Contains for &'a str {
	fn contains(&self, needle: &str) -> bool {
		self.matches(needle).count() > 0
	}
}

impl<'a> Contains for Cow<'a, str> {
	fn contains(&self, needle: &str) -> bool {
		self.matches(needle).count() > 0
	}
}

impl<'a, T> Contains for Vec<T>
where
	T: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.iter().any(|v| v.to_string() == needle)
	}
}

impl<'a, T> Contains for &'a Vec<T>
where
	T: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.iter().any(|v| v.to_string() == needle)
	}
}

impl<'a, T> Contains for &[T]
where
	T: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.iter().any(|v| v.to_string() == needle)
	}
}

impl<'a, T, const N: usize> Contains for [T; N]
where
	T: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.iter().any(|v| v.to_string() == needle)
	}
}

impl<'a, T, const N: usize> Contains for &[T; N]
where
	T: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.iter().any(|v| v.to_string() == needle)
	}
}

impl<'a, K, V, S> Contains for &'a HashMap<K, V, S>
where
	K: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.keys().any(|v| v.to_string() == needle)
	}
}

impl<'a, K, V, S> Contains for HashMap<K, V, S>
where
	K: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.keys().any(|v| v.to_string() == needle)
	}
}

impl<'a, T, S> Contains for &'a HashSet<T, S>
where
	T: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.iter().any(|v| v.to_string() == needle)
	}
}

impl<'a, T, S> Contains for HashSet<T, S>
where
	T: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.iter().any(|v| v.to_string() == needle)
	}
}

impl<'a, K, V> Contains for &'a BTreeMap<K, V>
where
	K: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.keys().any(|v| v.to_string() == needle)
	}
}

impl<'a, K, V> Contains for BTreeMap<K, V>
where
	K: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.keys().any(|v| v.to_string() == needle)
	}
}

impl<'a, T> Contains for &'a BTreeSet<T>
where
	T: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.iter().any(|v| v.to_string() == needle)
	}
}

impl<'a, T> Contains for BTreeSet<T>
where
	T: Display,
{
	fn contains(&self, needle: &str) -> bool {
		self.iter().any(|v| v.to_string() == needle)
	}
}

/// Validates whether the value contains the needle
/// The value needs to implement the Contains trait, which is implement on
/// [`String`], [`str`], [`Vec`], [`HashMap<String>`] and [`BTreeMap<String>`]
/// by default.
#[must_use]
pub fn validate_contains<T: Contains>(
	val: T,
	needle: &str,
) -> Result<T, Error> {
	val.contains(needle).then(|| val).ok_or_else(|| {
		Error::new(format!("Value does not contain the needle '{}'", needle))
	})
}

#[cfg(test)]
mod tests {
	use std::{borrow::Cow, collections::HashMap};

	use super::*;

	#[test]
	fn test_validate_contains_string() {
		assert!(validate_contains("hey", "e").is_ok());
	}

	#[test]
	fn test_validate_contains_string_can_fail() {
		assert!(!validate_contains("hey", "o").is_ok());
	}

	#[test]
	fn test_validate_contains_hashmap_key() {
		let mut map = HashMap::new();
		map.insert("hey".to_string(), 1);
		assert!(validate_contains(map, "hey").is_ok());
	}

	#[test]
	fn test_validate_contains_hashmap_key_can_fail() {
		let mut map = HashMap::new();
		map.insert("hey".to_string(), 1);
		assert!(!validate_contains(map, "bob").is_ok());
	}

	#[test]
	fn test_validate_contains_cow() {
		let test: Cow<'static, str> = "hey".into();
		assert!(validate_contains(test, "e").is_ok());
		let test: Cow<'static, str> = String::from("hey").into();
		assert!(validate_contains(test, "e").is_ok());
	}

	#[test]
	fn test_validate_contains_cow_can_fail() {
		let test: Cow<'static, str> = "hey".into();
		assert!(!validate_contains(test, "o").is_ok());
		let test: Cow<'static, str> = String::from("hey").into();
		assert!(!validate_contains(test, "o").is_ok());
	}

	#[test]
	fn test_validate_contains_hashmap() {
		let test: HashMap<String, ()> =
			[("hey".into(), ())].into_iter().collect();
		assert!(!validate_contains(test, "o").is_ok());
		let test: HashMap<&'static str, ()> =
			[("hey", ())].into_iter().collect();
		assert!(!validate_contains(test, "o").is_ok());
	}
}
