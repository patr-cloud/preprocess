# Preprocess

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

## List of allowed preprocessors

| Preprocessor                                         | Description                                         |
| ---------------------------------------------------- | --------------------------------------------------- |
| [`email`](./validators/#email)                       | Validates a string to be a valid email address.     |
| [`domain`](./validators/#domain)                     | Validates a string to be a valid domain name.       |
| [`ip`](./validators/#ip)                             | Validates a string to be a valid IP Address.        |
| [`url`](./validators/#url)                           | Validates a string to be a valid URL.               |
| [`length`](./validators/#length)                     | Validates the length of a string.                   |
| [`range`](./validators/#range)                       | Validates the range of a number.                    |
| [`contains`](./validators/#contains)                 | Validates if a string contains a substring.         |
| [`does_not_contain`](./validators/#does_not_contain) | Validates if a string does not contain a substring. |
| [`regex`](./validators/#regex)                       | Validates a string using a regex.                   |
| [`type`](#enforcing-the-type-of-a-value)             | Enforces the type of a value using `TryFrom`.       |
| [`trim`](./validators/#trim)                         | Trims a string.                                     |
| [`lowercase`](./validators/#lowercase)               | Converts a string to lowercase.                     |
| [`uppercase`](./validators/#uppercase)               | Converts a string to uppercase.                     |
| [`custom`](#custom-preprocessors)                    | Validates a string using a custom function.         |

More details about each preprocessor can be found in the respective module documentation of preprocessors and validators.

### Custom preprocessors

You can use a custom function as a preprocessor. The function must have the following signature:

```rust
fn custom_preprocessor<T>(value: T) -> Result<T, Error>;
```

The function must return a `Result` with the same type as the input. If the function returns an `Err`, the error will be returned as the error of the preprocessor. If the function returns an `Ok`, the value will be returned as the output of the preprocessor.

```rust
pub fn custom_preprocessor(value: String) -> Result<String, Error> {
    if value.len() < 8 {
        return Err(Error::new("Password must be at least 8 characters long"));
    }
    Ok(value)
}

#[preprocess::sync]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSignUpRequest {
    #[preprocess(custom = "custom_preprocessor")]
    pub password: String,
}
```

### Enforcing the type of a value

You can use the `type` preprocessor to enforce the type of a value. This is useful when you want to convert a value to a different type. For example, you might want to convert a string to an integer. You can use the `type` preprocessor to do this. The `type` preprocessor uses `TryFrom` to convert the value to the desired type. If the conversion fails, the preprocessor will return an error.

```rust
#[preprocess::sync]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSignUpRequest {
    #[preprocess(type = "i32")]
    pub age: i16,
}
```

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
preprocess = "<version>"
```

Then, you can import the crate and use it like this:

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
```

## MSRV

There is no MSRV as such, and to be honest, I don't see the point of an MSRV, with how easy rust is to upgrade. I just use the latest version of rust on my machine. That being said, I don't think I've used any new rust features. So it should work on older versions of rust as well. Please open an issue if you're facing any problems.

## Inspiration

This crate is hugely inspired by the `validator` crate. A huge thanks to [Keats](https://github.com/Keats) for creating it.

## License

This project is licensed under the [MIT license](./LICENSE).
