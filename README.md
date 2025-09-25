# bunner_qs

Rust utilities for parsing and serializing URL query strings that follow RFC 3986, RFC 3987, and the HTML `application/x-www-form-urlencoded` algorithm.

## Features

- **Standards first**: rejects malformed `%` sequences, stray `?`, unmatched brackets, and other non‑compliant tokens.
- **Configurable limits**: cap maximum length, number of parameters, and bracket depth when parsing.
- **Form mode toggle**: treat `+` as space only when explicitly enabled (`space_as_plus`).
- **Builder ergonomics**: `ParseOptions::builder()` and `StringifyOptions::builder()` expose fluent configuration with safe defaults.
- **Security by default**: duplicate keys are rejected to prevent HTTP parameter pollution (HPP).
- **Optional Serde bridge**: enable the `serde` feature to round‑trip structs with `Serialize`/`Deserialize`.

### Key naming policy

- Query keys and their nested bracket segments accept any UTF‑8 string, matching RFC 3986/3987 expectations.
- Percent‑encoded characters are decoded as Unicode; only control characters (`U+0000`–`U+001F`, `U+007F`) are rejected.
- No additional identifier rules (such as ASCII-only or first-character restrictions) are applied, so existing integrations continue to work unchanged.

## Quick start

```rust
use bunner_qs::{
	parse, parse_with, stringify, stringify_with, ParseOptions, StringifyOptions, QueryMap, Value,
};

let parse_opts = ParseOptions::builder()
	.space_as_plus(true) // HTML form mode
	.max_params(32)
	.build()?;

let params = parse_with("name=Jill+Doe&city=Seoul", &parse_opts)?;
assert_eq!(
	params
		.get("name")
		.and_then(|value| value.as_str()),
	Some("Jill Doe"),
);

let mut map = QueryMap::new();
map.insert("q".into(), Value::String("rust qs".into()));

let stringify_opts = StringifyOptions::builder()
	.space_as_plus(true)
	.build()?;

let query = stringify_with(&map, &stringify_opts)?;
assert_eq!(query, "q=rust+qs");
```

### Option defaults at a glance

- `ParseOptions::default()`
	- `space_as_plus = false` (no HTML form conversion unless you opt in)
	- `max_params = None` (no parameter-count limit)
	- `max_length = None` (no cumulative length limit)
	- `max_depth = None` (no bracket nesting limit)
- `StringifyOptions::default()`
	- `space_as_plus = false`
	- outputs never receive an automatic `?` prefix; prepend one yourself if you need it for URLs

### Serde integration (enabled by default)

This crate ships with serde support out of the box. If you prefer to disable it, declare the
dependency without default features:

```toml
[dependencies]
bunner_qs = { version = "0.1", default-features = false }
```

```rust
use bunner_qs::{parse, QueryMap, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Form {
	title: String,
	tags: Vec<String>,
}

let parsed = parse("title=Post&tags[0]=rust&tags[1]=web")?;
let form: Form = parsed.to_struct()?;
assert_eq!(form.tags, vec!["rust", "web"]);

let rebuilt = QueryMap::from_struct(&form)?;
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