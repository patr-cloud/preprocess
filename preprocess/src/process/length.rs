use crate::PreProcessError;

pub trait Length {
	fn length(&self) -> usize;
}

pub fn length(
	val: &impl Length,
	min: Option<usize>,
	max: Option<usize>,
) -> Result<(), PreProcessError> {
	let val_length = val.length();

	match (min, max) {
		(None, None) => panic!("Both min and max should not be empty"),
		(None, Some(max)) if val_length > max => Err(PreProcessError::new(
			format!("input len: {}, expected len <= {}", val_length, max),
		)),
		(Some(min), None) if val_length < min => Err(PreProcessError::new(
			format!("input len: {}, expected len >= {}", val_length, min),
		)),
		(Some(min), Some(max)) if !(val_length >= min && val_length <= max) => {
			Err(PreProcessError::new(format!(
				"input len: {}, expected {} <= len <= {}",
				val_length, min, max
			)))
		}
		_ => Ok(()),
	}
}

impl Length for String {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl<'a> Length for &'a String {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl<'a> Length for &'a str {
	fn length(&self) -> usize {
		self.chars().count()
	}
}

impl<'a> Length for std::borrow::Cow<'a, str> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> Length for Vec<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<'a, T> Length for &'a Vec<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> Length for &[T] {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T, const N: usize> Length for [T; N] {
	fn length(&self) -> usize {
		N
	}
}

impl<T, const N: usize> Length for &[T; N] {
	fn length(&self) -> usize {
		N
	}
}

impl<'a, K, V, S> Length for &'a std::collections::HashMap<K, V, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<K, V, S> Length for std::collections::HashMap<K, V, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<'a, T, S> Length for &'a std::collections::HashSet<T, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T, S> Length for std::collections::HashSet<T, S> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<'a, K, V> Length for &'a std::collections::BTreeMap<K, V> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<K, V> Length for std::collections::BTreeMap<K, V> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<'a, T> Length for &'a std::collections::BTreeSet<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> Length for std::collections::BTreeSet<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> Length for std::collections::VecDeque<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> Length for std::collections::BinaryHeap<T> {
	fn length(&self) -> usize {
		self.len()
	}
}

impl<T> Length for std::collections::LinkedList<T> {
	fn length(&self) -> usize {
		self.len()
	}
}
