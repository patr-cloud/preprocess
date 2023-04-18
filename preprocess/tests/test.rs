use preprocess::PreProcess;

#[test]
fn derive_test() -> Result<(), Box<dyn std::error::Error>> {
	mod append_zero_mod {
		use preprocess::PreProcessError;

		pub(crate) fn append_zero(
			val: &mut String,
		) -> Result<(), PreProcessError> {
			val.push('0');
			Ok(())
		}
	}

	#[derive(Debug, PartialEq, PreProcess)]
	struct Test {
		#[preprocess(lowercase, process_mut(append_zero_mod::append_zero))]
		name: String,
		#[preprocess(trim, length(max = 5))]
		value: String,
	}

	#[derive(Debug, PartialEq, PreProcess)]
	struct Outer {
		#[preprocess]
		test: Test,
	}

	#[derive(Debug, PartialEq, PreProcess)]
	enum Testing {
		B(
			#[preprocess(trim)] String,
			#[preprocess(regex = "^HE")] String,
		),
		D(#[preprocess] Outer),
	}

	assert_eq!(
		{
			let mut t = Testing::D(Outer {
				test: Test {
					name: "Hello".into(),
					value: "HEllo  ".into(),
				},
			});
			t.preprocess().unwrap();
			t
		},
		Testing::D(Outer {
			test: Test {
				name: "hello0".into(),
				value: "HEllo".into()
			}
		})
	);

	assert_eq!(
		{
			let mut t = Testing::B(" Hello".into(), "HEllo  ".into());
			t.preprocess().unwrap();
			t
		},
		Testing::B("Hello".into(), "HEllo  ".into())
	);

	#[derive(Debug, PartialEq, PreProcess)]
	struct T(
		#[preprocess(length(min = 2))]
		#[preprocess_item(trim, length(min = 3, max = 5))]
		Vec<String>,
	);

	assert_eq!(
		{
			let mut t = T(vec![" Hello".into(), "HEllo  ".into(),  "HEllo  ".into()]);
			t.preprocess().unwrap();
			t
		},
		T(vec!["Hello".into(), "HEllo".into(), "HEllo".into()])
	);

	Ok(())
}
