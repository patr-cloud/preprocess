use regex::Regex;

use crate::PreProcessError;

pub fn regex(
	val: &impl AsRef<str>,
	pat: &Regex,
) -> Result<(), PreProcessError> {
	if pat.is_match(val.as_ref()) {
		Ok(())
	} else {
		Err(PreProcessError::new(
			"Input does not match required pattern",
		))
	}
}
