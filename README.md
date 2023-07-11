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

## List of allowed preprocessors

| Preprocessor       | Description                                         |
| ------------------ | --------------------------------------------------- |
| `email`            | Validates a string to be a valid email address.     |
| `domain`           | Validates a string to be a valid domain name.       |
| `url`              | Validates a string to be a valid URL.               |
| `length`           | Validates the length of a string.                   |
| `range`            | Validates the range of a number.                    |
| `contains`         | Validates if a string contains a substring.         |
| `does_not_contain` | Validates if a string does not contain a substring. |
| `regex`            | Validates a string using a regex.                   |
| `type`             | Enforces the type of a value using `TryFrom`.       |
| `trim`             | Trims a string.                                     |
| `lowercase`        | Converts a string to lowercase.                     |
| `uppercase`        | Converts a string to uppercase.                     |
| `custom`           | Validates a string using a custom function.         |
