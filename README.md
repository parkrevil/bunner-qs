# bunner_qs

Rust utilities for parsing and serializing URL query strings that follow RFC 3986, RFC 3987, and the HTML `application/x-www-form-urlencoded` algorithm.

## Features

- **Standards first**: rejects malformed `%` sequences, stray `?`, unmatched brackets, and other non‑compliant tokens.
- **Configurable limits**: cap maximum length, number of parameters, and bracket depth when parsing.
- **Form mode toggle**: treat `+` as space only when explicitly enabled (`space_as_plus`).
- **Predictable output**: percent‑encodes using uppercase hex and optional `?` prefixing during serialization.

## Quick start

```rust
use bunner_qs::{parse_with_options, stringify_with_options, ParseOptions, StringifyOptions};

let mut parse_opts = ParseOptions::default();
parse_opts.space_as_plus = true; // HTML form mode
parse_opts.max_params = Some(32);

let params = parse_with_options("name=Jill+Doe&name=J.D.", &parse_opts)?;
assert_eq!(params.get("name"), Some(&vec!["Jill Doe".into(), "J.D.".into()]));

let mut map = bunner_qs::QueryMap::new();
map.insert("q".into(), vec!["rust qs".into()]);

let mut stringify_opts = StringifyOptions::default();
stringify_opts.add_query_prefix = true;

let query = stringify_with_options(&map, &stringify_opts)?;
assert_eq!(query, "?q=rust%20qs");
```

## License

This project is licensed under the MIT License — see the [LICENSE.md](LICENSE.md) file for details.