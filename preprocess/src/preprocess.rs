use serde::de::DeserializeOwned;

use crate::PreProcessError;

pub trait PreProcessor {
	const TAKES_ARGS: bool = false;

	type Args: DeserializeOwned + Default + Clone;
	type Processed;

	fn preprocess(self) -> Result<Self::Processed, PreProcessError>;

	#[allow(unused_variables)]
	fn set_args(&mut self, args: Self::Args) {}
	fn get_args(&self) -> Self::Args {
		Default::default()
	}
}

impl<Processable> PreProcessor for Option<Processable>
where
	Processable: PreProcessor,
{
	type Args = ();
	type Processed = Option<Processable::Processed>;

	fn preprocess(self) -> Result<Self::Processed, PreProcessError> {
		self.map(|value| PreProcessor::preprocess(value))
			.transpose()
	}
}

impl<Processable> PreProcessor for Vec<Processable>
where
	Processable: PreProcessor,
{
	type Args = ();
	type Processed = Vec<Processable::Processed>;

	fn preprocess(self) -> Result<Self::Processed, PreProcessError> {
		self.into_iter()
			.map(|value| PreProcessor::preprocess(value))
			.collect()
	}
}

impl<Processable, const LENGTH: usize> PreProcessor for [Processable; LENGTH]
where
	Processable: PreProcessor,
	Processable::Processed: Default + Copy,
{
	type Args = ();
	type Processed = [Processable::Processed; LENGTH];

	fn preprocess(self) -> Result<Self::Processed, PreProcessError> {
		let mut result: Self::Processed = [Default::default(); LENGTH];
		for (i, item) in self.into_iter().enumerate() {
			result[i] = PreProcessor::preprocess(item)?;
		}
		Ok(result)
	}
}
