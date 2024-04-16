# Preprocess

<div align="center">
  <!-- Version -->
  <a href="https://crates.io/crates/preprocess">
    <img src="https://img.shields.io/crates/v/preprocess.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Docs -->
  <a href="https://docs.rs/preprocess">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <!-- Downloads
  <a href="https://crates.io/crates/preprocess">
    <img src="https://img.shields.io/crates/d/preprocess.svg?style=flat-square"
      alt="Download" />
  </a> -->
</div>

A crate to help you preprocess your structs and enums.
Can be used to validate data, or to transform it.

There are two kinds of preprocessors:

- **Validators**: They check if the given field is valid and don't modify the value. For example: a validator could check if a string is a valid email address.
- **Preprocessors**: These allow you to modify the value (and possibly type) of a field. For example: a preprocessor could trim a string, or convert it to uppercase.

## Example usage

```rust
use preprocess::prelude::*;

#[preprocess::sync]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSignUpRequest {
    // First trims the email, then converts it to lowercase, then validates it as an email address.
    #[preprocess(trim, lowercase, email)]
    pub email: String,
    // First trims the password, then checks if it's at least 8 characters long.
    #[preprocess(trim, length(min = 8))]
    pub password: String,
}

let processed_value = raw_value.preprocess()?;
```

## Inheriting derive attributes

Since the crate uses an attribute macro, it must always be the first attribute on the struct or enum. A new struct / enum will be generated with the name `{original_name}Processed`. The derive macro will inherit all the derive attributes from the original struct / enum. For example:

```rust
use preprocess::prelude::*;

#[preprocess::sync]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSignUpRequest {
    #[preprocess(trim, lowercase, email)]
    #[serde(default)]
    pub email: String,
    #[serde(alias = "pass")]
    #[preprocess(trim, length(min = 8))]
    pub password: String,
}
```

The above code will generate:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSignUpRequestProcessed {
    #[serde(default)]
    pub email: String,
    #[serde(alias = "pass")]
    pub password: String,
}
```

This way, any custom derive attributes you use (like Serde) will be inherited by the generated struct / enum. This also ensures that you can preprocess your struct / enum and send the preprocessed version to the client, without having to write any extra code.

More details about the crate can be found in the [documentation](https://docs.rs/preprocess).

## MSRV

There is no MSRV as such, and to be honest, I don't see the point of an MSRV, with how easy rust is to upgrade. I just use the latest version of rust on my machine. That being said, I don't think I've used any new rust features. So it should work on older versions of rust as well. Please open an [issue](https://github.com/patr-cloud/preprocess/issues) if you're facing any, well, issues.

## Inspiration

This crate is largely inspired by the `validator` crate. A huge thanks to [Keats](https://github.com/Keats) for creating it.

## License

This project is licensed under the [MIT license](./LICENSE).
