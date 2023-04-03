use crate::PreProcessError;

mod regex;
mod valid_length;

pub use valid_length::valid_length;

pub use crate::process::regex::regex;

pub fn trim(val: &mut String) -> Result<(), PreProcessError> {
	let trimmed_val = val.trim();
	if trimmed_val.len() != val.len() {
		*val = trimmed_val.to_owned();
	}
	Ok(())
}

pub fn lowercase(val: &mut String) -> Result<(), PreProcessError> {
	*val = val.to_lowercase();
	Ok(())
}

pub fn process<T>(
	t: &T,
	f: impl FnOnce(&T) -> Result<(), PreProcessError>,
) -> Result<(), PreProcessError> {
	f(t)
}

pub fn process_mut<T>(
	t: &mut T,
	f: impl FnOnce(&mut T) -> Result<(), PreProcessError>,
) -> Result<(), PreProcessError> {
	f(&mut *t)
}
