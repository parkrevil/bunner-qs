mod common;

use bunner_qs::{ParseError, ParseOptions, StringifyOptions, parse_with, stringify_with};
use common::{assert_str_entry, map_from_pairs};

#[test]
fn parse_respects_max_params_limit() {
    let options = ParseOptions {
        max_params: Some(2),
        ..ParseOptions::default()
    };

    let ok = parse_with("a=1&b=2", &options).expect("limit should allow two params");
    assert_str_entry(&ok, "a", "1");
    assert_str_entry(&ok, "b", "2");

    let error = parse_with("a=1&b=2&c=3", &options).expect_err("third param should fail");
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
    let error = parse_with("only=one", &options).expect_err("zero limit should reject first pair");
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
    parse_with(query, &allowed).expect("length at limit should parse");

    let blocked = ParseOptions {
        max_length: Some(query.len() - 1),
        ..ParseOptions::default()
    };
    let error = parse_with(query, &blocked).expect_err("length over limit should fail");
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
    parse_with("a[b][c]=ok", &within).expect("depth 2 should succeed");

    let over = ParseOptions {
        max_depth: Some(2),
        ..ParseOptions::default()
    };
    let error = parse_with("a[b][c][d]=fail", &over).expect_err("depth beyond limit should fail");
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

    let parsed = parse_with("msg=hello+world", &options).expect("plus should decode to space");
    assert_str_entry(&parsed, "msg", "hello world");
}

#[test]
fn stringify_options_builder_controls_space_encoding() {
    let map = map_from_pairs(&[("greeting", "hello world")]);
    let options = StringifyOptions::builder()
        .space_as_plus(true)
        .build()
        .expect("stringify builder should succeed");

    let encoded = stringify_with(&map, &options).expect("should encode with plus");
    assert_eq!(encoded, "greeting=hello+world");
}
