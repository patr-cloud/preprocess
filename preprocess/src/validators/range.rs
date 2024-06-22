use crate::utils::Error;

/// Validates that the given `value` is inside the defined range. The `max` and
/// `min` parameters are optional and will only be validated if they are not
/// `None`
#[must_use = concat!(
	"validation returns a new value instead of mutating the input.",
	" The returned value will contain the validated value,",
	" while the input will remain unchanged"
)]
pub fn validate_range<T>(
	value: T,
	min: Option<T>,
	max: Option<T>,
) -> Result<T, Error>
where
	T: PartialOrd + PartialEq,
{
	if let Some(max) = max {
		if value > max {
			return Err(Error::new(
				"value must be less than or equal to the maximum",
			));
		}
	}

	if let Some(min) = min {
		if value < min {
			return Err(Error::new(
				"value must be greater than or equal to the minimum",
			));
		}
	}

	Ok(value)
}

#[cfg(test)]
mod tests {
	use super::validate_range;

	#[test]
	fn test_validate_range_generic_ok() {
		// Unspecified generic type:
		assert_eq!(validate_range(10, Some(-10), Some(10)), Ok(10));
		assert_eq!(validate_range(0.0, Some(0.0), Some(10.0)), Ok(0.0));

		// Specified type:
		assert_eq!(validate_range(5u8, Some(0), Some(255)), Ok(5u8));
		assert_eq!(validate_range(4u16, Some(0), Some(16)), Ok(4u16));
		assert_eq!(validate_range(6u32, Some(0), Some(23)), Ok(6u32));
	}

	#[test]
	fn test_validate_range_generic_fail() {
		assert!(validate_range(5, Some(17), Some(19)).is_err());
		assert!(validate_range(-1.0, Some(0.0), Some(10.0)).is_err());
	}

	#[test]
	fn test_validate_range_generic_min_only() {
		assert!(validate_range(5, Some(10), None).is_err());
		assert_eq!(validate_range(15, Some(10), None), Ok(15));
	}

	#[test]
	fn test_validate_range_generic_max_only() {
		assert_eq!(validate_range(5, None, Some(10)), Ok(5));
		assert!(validate_range(15, None, Some(10)).is_err());
	}
}
