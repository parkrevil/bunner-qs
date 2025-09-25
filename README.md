# bunner_qs

Rust utilities for parsing and serializing URL query strings that follow RFC 3986, RFC 3987, and the HTML `application/x-www-form-urlencoded` algorithm.

## Features

- **Standards first**: rejects malformed `%` sequences, stray `?`, unmatched brackets, and other non‑compliant tokens.
- **Configurable limits**: cap maximum length, number of parameters, and bracket depth when parsing.
- **Form mode toggle**: treat `+` as space only when explicitly enabled (`space_as_plus`).
- **Security by default**: duplicate keys are rejected to prevent HTTP parameter pollution (HPP).
- **Optional Serde bridge**: enable the `serde` feature to round‑trip structs with `Serialize`/`Deserialize`.

### Key naming policy

- Query keys and their nested bracket segments accept any UTF‑8 string, matching RFC 3986/3987 expectations.
- Percent‑encoded characters are decoded as Unicode; only control characters (`U+0000`–`U+001F`, `U+007F`) are rejected.
- No additional identifier rules (such as ASCII-only or first-character restrictions) are applied, so existing integrations continue to work unchanged.

## Quick start

```rust
use bunner_qs::{
	parse, stringify, ParseOptions, StringifyOptions, QueryMap, Value,
};

let mut parse_opts = ParseOptions::default();
parse_opts.space_as_plus = true; // HTML form mode
parse_opts.max_params = Some(32);

let params = parse("name=Jill+Doe&city=Seoul", Some(parse_opts.clone()))?;
assert_eq!(
	params
		.get("name")
		.and_then(|value| value.as_str()),
	Some("Jill Doe"),
);

let mut map = QueryMap::new();
map.insert("q".into(), Value::String("rust qs".into()));

let mut stringify_opts = StringifyOptions::default();
stringify_opts.add_query_prefix = true;

let query = stringify(&map, Some(stringify_opts))?;
assert_eq!(query, "?q=rust%20qs");
```

### Serde integration (enabled by default)

This crate ships with serde support out of the box. If you prefer to disable it, declare the
dependency without default features:

```toml
[dependencies]
bunner_qs = { version = "0.1", default-features = false }
```

```rust
use bunner_qs::{from_query_map, to_query_map, parse, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Form {
	title: String,
	tags: Vec<String>,
}

let parsed = parse("title=Post&tags[0]=rust&tags[1]=web", None)?;
let form: Form = from_query_map(&parsed)?;
assert_eq!(form.tags, vec!["rust", "web"]);

let rebuilt = to_query_map(&form)?;
assert_eq!(
	rebuilt
		.get("title")
		.and_then(|value| value.as_str()),
	Some("Post"),
);
assert_eq!(
	rebuilt
		.get("tags")
		.and_then(|value| value.as_array())
		.map(|items| items.iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>()),
	Some(vec!["rust", "web"]),
);
```

## License

This project is licensed under the MIT License — see the [LICENSE.md](LICENSE.md) file for details.