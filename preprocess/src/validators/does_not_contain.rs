use super::Contains;

/// Validates that the value does not contain the given needle.
/// This is the opposite of [`Contains`].
pub fn validate_does_not_contain<T: Contains>(value: &T, substr: &str) -> bool {
	!value.contains(substr)
}
