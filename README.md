# PreProcess

## Available processors:
- valid_length => min and max supported
- regex
- trim
- lowercase
- process => runs the given fn with ref of field
- process_mut => runs the given fn with mut ref of field

## TODO:
- [ ] Allow processing each item in collection
- [ ] Better error for nested struct
- [ ] Use feature flags for serde and regex
- [ ] Seriable erros if serde flag is enabled
- [ ] Use serde name for fields if compiled with serde flag
- [ ] Add common functions and methods
- [ ] Allow process and process mut for whole struct or enum in attribute
