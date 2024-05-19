/// Empty validator. This validator will always return `Ok` with the given
/// value. This is used for the `#[preprocess(none)]` attribute.
pub fn validate_empty<T>(val: T) -> Result<T, crate::Error> {
	Ok(val)
}
