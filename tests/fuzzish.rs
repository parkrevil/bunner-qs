use bunner_qs::{ParseError, ParseOptions, StringifyOptions, parse, parse_with, stringify_with};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, FileFailurePersistence};
use serde::Deserialize;
use serde_json::{Map as JsonMap, Value};

#[derive(Debug, Deserialize)]
struct SeedCase {
    name: String,
    input: String,
    expect: SeedExpect,
    #[serde(default)]
    options: Option<SeedOptions>,
}

#[derive(Debug, Deserialize)]
struct SeedOptions {
    #[serde(default)]
    space_as_plus: Option<bool>,
    #[serde(default)]
    max_params: Option<usize>,
    #[serde(default)]
    max_length: Option<usize>,
    #[serde(default)]
    max_depth: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct SeedStringifyOptions {
    #[serde(default)]
    space_as_plus: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct RoundTripSeed {
    name: String,
    query: String,
    #[serde(default)]
    parse_options: Option<SeedOptions>,
    #[serde(default)]
    stringify_options: Option<SeedStringifyOptions>,
    #[serde(default)]
    normalized: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SeedExpect {
    Ok,
    DuplicateKey,
    InvalidPercentEncoding,
    InvalidCharacter,
    TooManyParameters,
    InputTooLong,
    DepthExceeded,
    UnmatchedBracket,
    UnexpectedQuestionMark,
    InvalidUtf8,
}

impl SeedCase {
    fn parse_options(&self) -> ParseOptions {
        build_parse_options(self.options.as_ref())
    }
}

impl RoundTripSeed {
    fn parse_options(&self) -> ParseOptions {
        build_parse_options(self.parse_options.as_ref())
    }

    fn stringify_options(&self) -> StringifyOptions {
        build_stringify_options(self.stringify_options.as_ref())
    }

    fn normalized_query(&self) -> Option<&str> {
        self.normalized.as_deref()
    }
}

fn build_parse_options(config: Option<&SeedOptions>) -> ParseOptions {
    let mut opts = ParseOptions::default();
    if let Some(cfg) = config {
        if let Some(space) = cfg.space_as_plus {
            opts.space_as_plus = space;
        }
        if let Some(max_params) = cfg.max_params {
            opts.max_params = Some(max_params);
        }
        if let Some(max_length) = cfg.max_length {
            opts.max_length = Some(max_length);
        }
        if let Some(max_depth) = cfg.max_depth {
            opts.max_depth = Some(max_depth);
        }
    }
    opts
}

fn build_stringify_options(config: Option<&SeedStringifyOptions>) -> StringifyOptions {
    let mut opts = StringifyOptions::default();
    if let Some(SeedStringifyOptions {
        space_as_plus: Some(space),
    }) = config
    {
        opts.space_as_plus = *space;
    }
    opts
}

fn load_cases(data: &str) -> Vec<SeedCase> {
    serde_json::from_str(data).expect("seed JSON should parse")
}

fn load_roundtrip_cases(data: &str) -> Vec<RoundTripSeed> {
    serde_json::from_str(data).expect("roundtrip seed JSON should parse")
}

fn expect_result(case: &SeedCase, result: Result<Value, ParseError>) {
    match case.expect {
        SeedExpect::Ok => {
            result.unwrap_or_else(|err| {
                panic!("case `{}` expected success but failed: {err:?}", case.name)
            });
        }
        SeedExpect::DuplicateKey => match result {
            Err(ParseError::DuplicateKey { .. }) => {}
            other => panic!(
                "case `{}` expected DuplicateKey, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::InvalidPercentEncoding => match result {
            Err(ParseError::InvalidPercentEncoding { .. }) => {}
            other => panic!(
                "case `{}` expected InvalidPercentEncoding, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::InvalidCharacter => match result {
            Err(ParseError::InvalidCharacter { .. }) => {}
            other => panic!(
                "case `{}` expected InvalidCharacter, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::TooManyParameters => match result {
            Err(ParseError::TooManyParameters { .. }) => {}
            other => panic!(
                "case `{}` expected TooManyParameters, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::InputTooLong => match result {
            Err(ParseError::InputTooLong { .. }) => {}
            other => panic!(
                "case `{}` expected InputTooLong, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::DepthExceeded => match result {
            Err(ParseError::DepthExceeded { .. }) => {}
            other => panic!(
                "case `{}` expected DepthExceeded, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::UnmatchedBracket => match result {
            Err(ParseError::UnmatchedBracket { .. }) => {}
            other => panic!(
                "case `{}` expected UnmatchedBracket, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::UnexpectedQuestionMark => match result {
            Err(ParseError::UnexpectedQuestionMark { .. }) => {}
            other => panic!(
                "case `{}` expected UnexpectedQuestionMark, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::InvalidUtf8 => match result {
            Err(ParseError::InvalidUtf8) => {}
            other => panic!("case `{}` expected InvalidUtf8, got {:?}", case.name, other),
        },
    }
}

fn normalize_empty(value: Value) -> Value {
    match value {
        Value::Null => Value::Object(JsonMap::new()),
        other => other,
    }
}

#[test]
fn seed_allow_cases() {
    const DATA: &str = include_str!("data/query_allow.json");
    for case in load_cases(DATA) {
        let opts = case.parse_options();
        let result = parse_with::<Value>(&case.input, &opts);
        expect_result(&case, result);
    }
}

#[test]
fn seed_reject_cases() {
    const DATA: &str = include_str!("data/query_reject.json");
    for case in load_cases(DATA) {
        let opts = case.parse_options();
        let result = parse_with::<Value>(&case.input, &opts);
        expect_result(&case, result);
    }
}

#[test]
fn seed_roundtrip_cases() {
    const DATA: &str = include_str!("data/query_roundtrip.json");
    for case in load_roundtrip_cases(DATA) {
        let name = &case.name;
        let parse_opts = case.parse_options();
        let stringify_opts = case.stringify_options();
        let parsed: Value = parse_with(&case.query, &parse_opts).unwrap_or_else(|err| {
            panic!("case `{name}` expected parse success but failed: {err:?}")
        });
        let normalized = stringify_with(&parsed, &stringify_opts)
            .unwrap_or_else(|err| panic!("case `{name}` failed to stringify: {err:?}"));
        if let Some(expected) = case.normalized_query() {
            let normalized_value: Value = parse(&normalized).unwrap_or_else(|err| {
                panic!("case `{name}` failed to parse normalized output: {err:?}")
            });
            let expected_value: Value = parse(expected).unwrap_or_else(|err| {
                panic!("case `{name}` failed to parse expected normalized output: {err:?}")
            });
            assert_eq!(
                normalize_empty(normalized_value),
                normalize_empty(expected_value),
                "case `{name}` normalized output mismatch"
            );
        }
        let reparsed: Value = parse_with(&normalized, &parse_opts).unwrap_or_else(|err| {
            panic!("case `{name}` failed to parse normalized output: {err:?}")
        });
        assert_eq!(
            normalize_empty(reparsed),
            normalize_empty(parsed),
            "case `{name}` round-trip altered structure"
        );
    }
}

fn allowed_char() -> impl Strategy<Value = char> {
    prop::char::range('\u{0020}', '\u{10FFFF}').prop_filter("exclude DEL", |c| *c != '\u{007F}')
}

fn unicode_value_string() -> impl Strategy<Value = String> {
    prop::collection::vec(allowed_char(), 0..6).prop_map(|chars| chars.into_iter().collect())
}

fn percent_encode(input: &str) -> String {
    input
        .as_bytes()
        .iter()
        .map(|byte| format!("%{:02X}", byte))
        .collect()
}

fn form_encode(input: &str) -> String {
    input
        .as_bytes()
        .iter()
        .map(|byte| match byte {
            b' ' => "+".to_string(),
            other => format!("%{:02X}", other),
        })
        .collect()
}

fn unicode_key_string() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::char::range('\u{0020}', '\u{10FFFF}').prop_filter("exclude DEL and brackets", |c| {
            *c != '\u{007F}' && *c != '[' && *c != ']'
        }),
        1..5,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

fn root_key_string() -> impl Strategy<Value = String> {
    const ROOT_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let choices: Vec<char> = ROOT_CHARS.chars().collect();
    prop::collection::vec(prop::sample::select(choices), 3..8)
        .prop_map(|chars| chars.into_iter().collect())
}

fn object_key_string() -> impl Strategy<Value = String> {
    const FIRST: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const REST: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
    let first_choices: Vec<char> = FIRST.chars().collect();
    let rest_choices: Vec<char> = REST.chars().collect();
    (
        prop::sample::select(first_choices),
        prop::collection::vec(prop::sample::select(rest_choices), 0..4),
    )
        .prop_map(|(first, rest)| {
            let mut s = String::new();
            s.push(first);
            for ch in rest {
                s.push(ch);
            }
            s
        })
}

fn value_depth(value: &Value) -> usize {
    match value {
        Value::String(_) => 0,
        Value::Array(items) => items.iter().map(value_depth).max().unwrap_or(0) + 1,
        Value::Object(map) => map.values().map(value_depth).max().unwrap_or(0) + 1,
        _ => 0,
    }
}

fn root_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => map.values().map(value_depth).max().unwrap_or(0),
        other => value_depth(other),
    }
}

fn estimate_params_value(value: &Value) -> usize {
    match value {
        Value::String(_) => 1,
        Value::Array(items) => items.iter().map(estimate_params_value).sum(),
        Value::Object(map) => map.values().map(estimate_params_value).sum(),
        _ => 1,
    }
}

fn estimate_params(value: &Value) -> usize {
    match value {
        Value::Object(map) => map.values().map(estimate_params_value).sum(),
        other => estimate_params_value(other),
    }
}

fn arb_query_value() -> impl Strategy<Value = Value> {
    let leaf = unicode_value_string().prop_map(Value::String);
    leaf.prop_recursive(3, 32, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 1..3).prop_map(Value::Array),
            prop::collection::vec((object_key_string(), inner), 1..3).prop_map(|pairs| {
                let mut map = JsonMap::new();
                for (key, value) in pairs {
                    map.entry(key).or_insert(value);
                }
                Value::Object(map)
            })
        ]
    })
}

fn arb_root_value() -> impl Strategy<Value = Value> {
    prop::collection::vec((root_key_string(), arb_query_value()), 0..4).prop_map(|pairs| {
        let mut map = JsonMap::new();
        for (key, value) in pairs {
            map.entry(key).or_insert(value);
        }
        Value::Object(map)
    })
}

#[derive(Debug, Clone)]
struct RoundTripConfig {
    space_as_plus: bool,
    max_params: Option<usize>,
    max_length: Option<usize>,
    max_depth: Option<usize>,
}

fn arb_roundtrip_input() -> impl Strategy<Value = (Value, RoundTripConfig)> {
    arb_root_value().prop_flat_map(|value| {
        let depth = root_depth(&value);
        let params = estimate_params(&value);
        let seed = value.clone();
        (
            any::<bool>(),
            prop::option::of(0usize..3),
            any::<bool>(),
            any::<bool>(),
        )
            .prop_map(
                move |(space_as_plus, extra_params, use_length, use_depth)| {
                    let max_params = extra_params.map(|extra| params + extra);
                    let max_length = if use_length { Some(4096) } else { None };
                    let max_depth = if use_depth { Some(depth + 2) } else { None };
                    (
                        seed.clone(),
                        RoundTripConfig {
                            space_as_plus,
                            max_params,
                            max_length,
                            max_depth,
                        },
                    )
                },
            )
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/fuzzish.proptest-regressions"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn percent_encoded_unicode_round_trip(pairs in prop::collection::vec((unicode_key_string(), unicode_value_string()), 0..4)) {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for (key, _) in &pairs {
            if !seen.insert(key) {
                prop_assume!(false);
            }
        }

        let mut query_segments = Vec::new();
        for (key, value) in &pairs {
            let encoded_key = percent_encode(key);
            let encoded_value = percent_encode(value);
            query_segments.push(format!("{}={}", encoded_key, encoded_value));
        }
        let query = query_segments.join("&");
        let parsed: Value = parse(&query).expect("percent-encoded unicode should parse");
        if pairs.is_empty() {
            prop_assert!(parsed.is_null());
        } else {
            let object = parsed.as_object().expect("parsed root should be object");
            for (idx, (key, value)) in pairs.iter().enumerate() {
                let stored = object.get(key).unwrap_or_else(|| panic!("missing key at index {idx}"));
                let s = stored.as_str().unwrap_or_else(|| panic!("expected string for value `{key}`"));
                prop_assert_eq!(s, value);
            }
        }
    }

    #[test]
    fn plus_space_transcode(value in unicode_value_string()) {
        let encoded = form_encode(&value);
        let query = format!("note={}", encoded);
        let opts = ParseOptions {
            space_as_plus: true,
            ..Default::default()
        };
        let parsed: Value = parse_with(&query, &opts).expect("should decode plus as space");
        let stored = parsed
            .as_object()
            .expect("parsed root should be object")
            .get("note")
            .expect("missing note")
            .as_str()
            .expect("note should be string");
        prop_assert_eq!(stored, value);
    }

    #[test]
    fn max_params_guard(limit in 0usize..6, extra in 1usize..4) {
        let actual_params = limit + extra;
        let mut segments = Vec::with_capacity(actual_params);
        for idx in 0..actual_params {
            segments.push(format!("k{idx}=v{idx}"));
        }
        let query = segments.join("&");
        let opts = ParseOptions {
            max_params: Some(limit),
            ..Default::default()
        };
        let result = parse_with::<Value>(&query, &opts);
        match result {
            Err(ParseError::TooManyParameters { limit: lim, actual: act }) => {
                prop_assert_eq!(lim, limit);
                prop_assert_eq!(act, limit + 1);
            }
            other => prop_assert!(false, "expected TooManyParameters, got {:?}", other),
        }
    }

    #[test]
    fn max_length_guard(value_chars in prop::collection::vec(allowed_char(), 6..24)) {
        let value: String = value_chars.into_iter().collect();
        let query = format!("len={value}");
        let limit = query.len() - 1;
        let opts = ParseOptions {
            max_length: Some(limit),
            ..Default::default()
        };
        let result = parse_with::<Value>(&query, &opts);
        match result {
            Err(ParseError::InputTooLong { limit: lim }) => {
                prop_assert_eq!(lim, limit);
            }
            other => prop_assert!(false, "expected InputTooLong, got {:?}", other),
        }
    }

    #[test]
    fn max_depth_guard(limit in 0usize..4) {
        let depth = limit + 1;
        let mut key = String::from("root");
        for _ in 0..depth {
            key.push_str("[branch]");
        }
        let query = format!("{key}=deep");
        let opts = ParseOptions {
            max_depth: Some(limit),
            ..Default::default()
        };
        let result = parse_with::<Value>(&query, &opts);
        match result {
            Err(ParseError::DepthExceeded { limit: lim, .. }) => {
                prop_assert_eq!(lim, limit);
            }
            other => prop_assert!(false, "expected DepthExceeded, got {:?}", other),
        }
    }

    #[test]
    fn max_depth_within_limit(limit in 0usize..4) {
        let key = if limit == 0 {
            String::from("flat")
        } else {
            let mut k = String::from("root");
            for _ in 0..limit {
                k.push_str("[branch]");
            }
            k
        };
        let query = format!("{key}=ok");
        let opts = ParseOptions {
            max_depth: Some(limit),
            ..Default::default()
        };
        let result = parse_with::<Value>(&query, &opts);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn nested_array_object_round_trip(
        root in root_key_string(),
        items in prop::collection::vec((unicode_value_string(), unicode_value_string()), 1..4)
    ) {
        let mut segments = Vec::new();
        segments.push(format!("{root}[meta][version]=1"));
        segments.push(format!("{root}[meta][count]={}", items.len()));
        segments.push(format!("{root}[constants][pi]=3.14159"));
        segments.push(format!("{root}[constants][tau]=6.28318"));

        for (idx, (name, note)) in items.iter().enumerate() {
            let encoded_name = percent_encode(name);
            let encoded_note = percent_encode(note);
            let flag_enabled = format!("enabled-{idx}");
            let flag_beta = format!("beta-{idx}");
            segments.push(format!("{root}[items][{idx}][name]={encoded_name}"));
            segments.push(format!("{root}[items][{idx}][note]={encoded_note}"));
            segments.push(format!("{root}[items][{idx}][flags][]={flag_enabled}"));
            segments.push(format!("{root}[items][{idx}][flags][]={flag_beta}"));
        }

        let query = segments.join("&");
        let parsed: Value = parse(&query).expect("nested structures should parse");
        let root_value = parsed
            .as_object()
            .expect("parsed root should be object")
            .get(&root)
            .expect("missing root value")
            .as_object()
            .expect("root entry should be object");

        let meta = root_value
            .get("meta")
            .expect("meta missing")
            .as_object()
            .expect("meta should be object");
        let count = meta
            .get("count")
            .expect("count missing")
            .as_str()
            .expect("count string");
        let expected_count = items.len().to_string();
        prop_assert_eq!(count, expected_count);
        let version = meta
            .get("version")
            .expect("version missing")
            .as_str()
            .expect("version string");
        prop_assert_eq!(version, "1");

        let constants = root_value
            .get("constants")
            .expect("constants missing")
            .as_object()
            .expect("constants object");
        prop_assert_eq!(
            constants
                .get("pi")
                .expect("pi missing")
                .as_str()
                .expect("pi string"),
            "3.14159"
        );
        prop_assert_eq!(
            constants
                .get("tau")
                .expect("tau missing")
                .as_str()
                .expect("tau string"),
            "6.28318"
        );

        let items_value = root_value
            .get("items")
            .expect("items missing")
            .as_array()
            .expect("items array");
        prop_assert_eq!(items_value.len(), items.len());

        for (idx, (expected_name, expected_note)) in items.iter().enumerate() {
            let entry = items_value[idx]
                .as_object()
                .expect("entry should be object");
            let name_value = entry
                .get("name")
                .expect("name missing")
                .as_str()
                .expect("name string");
            prop_assert_eq!(name_value, expected_name);
            let note_value = entry
                .get("note")
                .expect("note missing")
                .as_str()
                .expect("note string");
            prop_assert_eq!(note_value, expected_note);

            let flags = entry
                .get("flags")
                .expect("flags missing")
                .as_array()
                .expect("flags array");
            prop_assert_eq!(flags.len(), 2);
            let first = flags[0].as_str().expect("flag string");
            let second = flags[1].as_str().expect("flag string");
            prop_assert_eq!(first, format!("enabled-{idx}"));
            prop_assert_eq!(second, format!("beta-{idx}"));
        }
    }

    #[test]
    fn stringify_parse_roundtrip_survives_random_structures((map, config) in arb_roundtrip_input()) {
        let params_required = estimate_params(&map);
        let depth_required = root_depth(&map);

        let parse_options = ParseOptions {
            space_as_plus: config.space_as_plus,
            max_params: config.max_params,
            max_length: config.max_length,
            max_depth: config.max_depth,
        };
        let stringify_options = StringifyOptions {
            space_as_plus: config.space_as_plus,
        };

        if let Some(limit) = parse_options.max_params {
            prop_assume!(params_required <= limit);
        }
        if let Some(limit) = parse_options.max_depth {
            prop_assume!(depth_required <= limit);
        }

        let encoded = stringify_with(&map, &stringify_options).expect("stringify should succeed");
        if let Some(limit) = parse_options.max_length {
            prop_assume!(encoded.len() <= limit);
        }

    let reparsed: Value = parse_with(&encoded, &parse_options).expect("round trip parse should succeed");
    prop_assert_eq!(normalize_empty(reparsed), normalize_empty(map.clone()));
    }

    #[test]
    fn control_characters_in_values_are_rejected(value in prop::collection::vec(prop::char::range('\u{0000}', '\u{001F}'), 1..10)) {
        let bad_value: String = value.into_iter().collect();
        let query = format!("bad={}", percent_encode(&bad_value));
    let result = parse::<Value>(&query);
        match result {
            Err(ParseError::InvalidCharacter { .. }) => {}
            other => prop_assert!(false, "expected InvalidCharacter for control chars, got {:?}", other),
        }
    }

    #[test]
    fn deep_nesting_with_high_depth(depth in 5usize..15) {
        let mut key = String::from("root");
        for _ in 0..depth {
            key.push_str("[level]");
        }
        let query = format!("{key}=deep");
        let opts = ParseOptions {
            max_depth: Some(depth),
            ..Default::default()
        };
    let result = parse_with::<Value>(&query, &opts);
        prop_assert!(result.is_ok(), "deep nesting should succeed within limit");
    }

    #[test]
    fn large_input_with_many_keys(num_keys in 50usize..200) {
        let mut segments = Vec::new();
        for idx in 0..num_keys {
            segments.push(format!("key{idx}=value{idx}"));
        }
        let query = segments.join("&");
        let opts = ParseOptions {
            max_params: Some(num_keys),
            max_length: Some(query.len()),
            ..Default::default()
        };
    let result = parse_with::<Value>(&query, &opts);
        prop_assert!(result.is_ok(), "large input should parse within limits");
    }
}
