#[path = "common/fuzzish/mod.rs"]
mod fuzzish;
#[path = "common/seed/mod.rs"]
mod seed;

use bunner_qs::{
    DuplicateKeyBehavior, ParseError, ParseOptions, StringifyOptions, parse, parse_with,
    stringify_with,
};
use fuzzish::{
    allowed_char, arb_roundtrip_input, estimate_params, form_encode, percent_encode, root_depth,
    root_key_string, string_with_spaces, total_string_length, unicode_key_string,
    unicode_value_string,
};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, FileFailurePersistence};
use seed::{allow_cases, assert_case_outcome, normalize_empty, reject_cases, roundtrip_cases};
use serde_json::{Value, json};

#[test]
fn seed_allow_cases() {
    for case in allow_cases() {
        let opts = case.parse_options();
        let result = parse_with::<Value>(&case.input, &opts);
        assert_case_outcome(&case, result);
    }
}

#[test]
fn seed_reject_cases() {
    for case in reject_cases() {
        let opts = case.parse_options();
        let result = parse_with::<Value>(&case.input, &opts);
        assert_case_outcome(&case, result);
    }
}

#[test]
fn seed_roundtrip_cases() {
    for case in roundtrip_cases() {
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
            duplicate_keys: DuplicateKeyBehavior::Reject,
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
            duplicate_keys: DuplicateKeyBehavior::Reject,
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
            duplicate_keys: DuplicateKeyBehavior::Reject,
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
            duplicate_keys: DuplicateKeyBehavior::Reject,
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
            duplicate_keys: DuplicateKeyBehavior::Reject,
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
            duplicate_keys: DuplicateKeyBehavior::Reject,
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
    fn space_plus_encoding_counts_spaces(value in string_with_spaces()) {
        let options = StringifyOptions {
            space_as_plus: true,
        };
        let encoded = stringify_with(&json!({"msg": value.clone()}), &options)
            .expect("stringify should succeed");
        let plus_count = encoded.chars().filter(|c| *c == '+').count();
        let space_count = value.chars().filter(|c| *c == ' ').count();
        prop_assert_eq!(plus_count, space_count);

        let parsed: Value = parse_with(&encoded, &ParseOptions {
            space_as_plus: true,
            duplicate_keys: DuplicateKeyBehavior::Reject,
            ..Default::default()
        })
        .expect("parse should succeed");
        let reparsed = parsed
            .as_object()
            .expect("parsed root should be object")
            .get("msg")
            .expect("message missing")
            .as_str()
            .expect("message should be string");
        prop_assert_eq!(reparsed, value);
    }

    #[test]
    fn encoding_never_shorter_than_string_data((map, config) in arb_roundtrip_input()) {
        let stringify_options = StringifyOptions {
            space_as_plus: config.space_as_plus,
        };
        let encoded = stringify_with(&map, &stringify_options).expect("stringify should succeed");
        let total_len = total_string_length(&map);
        prop_assert!(
            encoded.len() >= total_len,
            "encoded length {} shorter than total string bytes {}",
            encoded.len(),
            total_len
        );
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
            duplicate_keys: DuplicateKeyBehavior::Reject,
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
            duplicate_keys: DuplicateKeyBehavior::Reject,
            ..Default::default()
        };
    let result = parse_with::<Value>(&query, &opts);
        prop_assert!(result.is_ok(), "large input should parse within limits");
    }
}
