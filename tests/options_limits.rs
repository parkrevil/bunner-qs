mod common;

use bunner_qs::{ParseError, ParseOptions, StringifyOptions, parse_with, stringify_with};
use common::{assert_str_entry, expect_object, json_from_pairs};
use serde_json::Value;

#[test]
fn parse_respects_max_params_limit() {
    let options = ParseOptions {
        max_params: Some(2),
        ..ParseOptions::default()
    };

    let ok: Value = parse_with("a=1&b=2", &options).expect("limit should allow two params");
    let ok_obj = expect_object(&ok);
    assert_str_entry(ok_obj, "a", "1");
    assert_str_entry(ok_obj, "b", "2");

    let error = parse_with::<Value>("a=1&b=2&c=3", &options).expect_err("third param should fail");
    match error {
        ParseError::TooManyParameters { limit, actual } => {
            assert_eq!(limit, 2);
            assert_eq!(actual, 3);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_enforces_zero_param_limit() {
    let options = ParseOptions {
        max_params: Some(0),
        ..ParseOptions::default()
    };
    let error =
        parse_with::<Value>("only=one", &options).expect_err("zero limit should reject first pair");
    match error {
        ParseError::TooManyParameters { limit, actual } => {
            assert_eq!(limit, 0);
            assert_eq!(actual, 1);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_respects_max_length_boundary() {
    let query = "token=abcdef"; // length 12
    let allowed = ParseOptions {
        max_length: Some(query.len()),
        ..ParseOptions::default()
    };
    parse_with::<Value>(query, &allowed).expect("length at limit should parse");

    let blocked = ParseOptions {
        max_length: Some(query.len() - 1),
        ..ParseOptions::default()
    };
    let error = parse_with::<Value>(query, &blocked).expect_err("length over limit should fail");
    match error {
        ParseError::InputTooLong { limit } => assert_eq!(limit, query.len() - 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_respects_max_depth_boundary() {
    let within = ParseOptions {
        max_depth: Some(2),
        ..ParseOptions::default()
    };
    parse_with::<Value>("a[b][c]=ok", &within).expect("depth 2 should succeed");

    let over = ParseOptions {
        max_depth: Some(2),
        ..ParseOptions::default()
    };
    let error =
        parse_with::<Value>("a[b][c][d]=fail", &over).expect_err("depth beyond limit should fail");
    match error {
        ParseError::DepthExceeded { key, limit } => {
            assert_eq!(key, "a[b][c][d]");
            assert_eq!(limit, 2);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_options_builder_configures_all_fields() {
    let options = ParseOptions::builder()
        .space_as_plus(true)
        .max_params(3)
        .max_length(64)
        .max_depth(4)
        .build()
        .expect("builder should succeed");

    assert!(options.space_as_plus);
    assert_eq!(options.max_params, Some(3));
    assert_eq!(options.max_length, Some(64));
    assert_eq!(options.max_depth, Some(4));
}

#[test]
fn parse_options_builder_rejects_zero_limits() {
    let params_err = ParseOptions::builder()
        .max_params(0)
        .build()
        .expect_err("zero param limit should be rejected by builder");
    let params_msg = params_err.to_string();
    assert!(
        params_msg.contains("max_params"),
        "expected `{params_msg}` to contain `max_params`"
    );

    let length_err = ParseOptions::builder()
        .max_length(0)
        .build()
        .expect_err("zero length limit should be rejected by builder");
    let length_msg = length_err.to_string();
    assert!(
        length_msg.contains("max_length"),
        "expected `{length_msg}` to contain `max_length`"
    );

    let depth_err = ParseOptions::builder()
        .max_depth(0)
        .build()
        .expect_err("zero depth limit should be rejected by builder");
    let depth_msg = depth_err.to_string();
    assert!(
        depth_msg.contains("max_depth"),
        "expected `{depth_msg}` to contain `max_depth`"
    );
}

#[test]
fn parse_combined_limits_prioritize_length_check() {
    let options = ParseOptions {
        max_params: Some(5),
        max_length: Some(5),
        ..ParseOptions::default()
    };

    let error = parse_with::<Value>("toolong=value", &options)
        .expect_err("length limit should surface before param counting");
    match error {
        ParseError::InputTooLong { limit } => assert_eq!(limit, 5),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_combined_limits_still_enforce_params() {
    let options = ParseOptions {
        max_params: Some(1),
        max_length: Some(64),
        ..ParseOptions::default()
    };

    let error = parse_with::<Value>("a=1&b=2", &options)
        .expect_err("second parameter should breach param limit");
    match error {
        ParseError::TooManyParameters { limit, actual } => {
            assert_eq!(limit, 1);
            assert_eq!(actual, 2);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_combined_limits_respect_depth_even_with_param_budget() {
    let options = ParseOptions {
        max_params: Some(10),
        max_depth: Some(1),
        ..ParseOptions::default()
    };

    let error = parse_with::<Value>("a[b][c]=1", &options)
        .expect_err("depth limit should trigger ahead of parameter budget");
    match error {
        ParseError::DepthExceeded { key, limit } => {
            assert_eq!(key, "a[b][c]");
            assert_eq!(limit, 1);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_handles_extremely_large_limits_without_overflow() {
    let options = ParseOptions {
        max_params: Some(usize::MAX),
        max_length: Some(usize::MAX),
        max_depth: Some(usize::MAX),
        ..ParseOptions::default()
    };

    let parsed: Value = parse_with("a=1&b=2", &options).expect("extreme limits should still parse");
    let obj = expect_object(&parsed);
    assert_str_entry(obj, "a", "1");
    assert_str_entry(obj, "b", "2");
}

#[test]
fn parse_options_builder_defaults_match_default() {
    let built = ParseOptions::builder()
        .build()
        .expect("builder without overrides should succeed");
    assert_eq!(built.space_as_plus, ParseOptions::default().space_as_plus);
    assert_eq!(built.max_params, ParseOptions::default().max_params);
    assert_eq!(built.max_length, ParseOptions::default().max_length);
    assert_eq!(built.max_depth, ParseOptions::default().max_depth);
}

#[test]
fn parse_with_builder_space_as_plus_decodes_plus() {
    let options = ParseOptions::builder()
        .space_as_plus(true)
        .build()
        .expect("builder should succeed");

    let parsed: Value =
        parse_with("msg=hello+world", &options).expect("plus should decode to space");
    let obj = expect_object(&parsed);
    assert_str_entry(obj, "msg", "hello world");
}

#[test]
fn stringify_options_builder_controls_space_encoding() {
    let map = json_from_pairs(&[("greeting", "hello world")]);
    let options = StringifyOptions::builder()
        .space_as_plus(true)
        .build()
        .expect("stringify builder should succeed");

    let encoded = stringify_with(&map, &options).expect("should encode with plus");
    assert_eq!(encoded, "greeting=hello+world");
}
