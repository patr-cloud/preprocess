use super::Contains;
use crate::utils::Error;

/// Validates that the value does not contain the given needle.
/// This is the opposite of [`Contains`].
#[must_use = concat!(
	"validation returns a new value instead of mutating the input.",
	" The returned value will contain the validated value,",
	" while the input will remain unchanged"
)]
pub fn validate_does_not_contain<T: Contains>(value: T, needle: &str) -> Result<T, Error> {
	(!value.contains(needle)).then_some(value).ok_or_else(|| {
		Error::new(format!("Value does not contain the needle '{}'", needle))
	})
}
