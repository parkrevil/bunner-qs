#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/json.rs"]
mod json;
#[path = "common/options.rs"]
mod options;
#[path = "common/stringify_options.rs"]
mod stringify_options;

use asserts::{assert_str_path, assert_string_array_path};
use bunner_qs::{ParseError, ParseOptions, parse, parse_with, stringify_with};
use json::json_from_pairs;
use options::{build_parse_options, try_build_parse_options};
use serde_json::Value;
use stringify_options::build_stringify_options;

#[test]
fn parse_respects_max_params_limit() {
    let options = build_parse_options(|builder| builder.max_params(2));

    let ok: Value = parse_with("a=1&b=2", &options).expect("limit should allow two params");
    assert_str_path(&ok, &["a"], "1");
    assert_str_path(&ok, &["b"], "2");

    asserts::assert_err_matches!(
        parse_with::<Value>("a=1&b=2&c=3", &options),
        ParseError::TooManyParameters { limit, actual } => |_message| {
            assert_eq!(limit, 2);
            assert_eq!(actual, 3);
        }
    );
}

#[test]
fn parse_enforces_zero_param_limit() {
    let options = ParseOptions {
        max_params: Some(0),
        ..ParseOptions::default()
    };
    asserts::assert_err_matches!(
        parse_with::<Value>("only=one", &options),
        ParseError::TooManyParameters { limit, actual } => |_message| {
            assert_eq!(limit, 0);
            assert_eq!(actual, 1);
        }
    );
}

#[test]
fn parse_respects_max_length_boundary() {
    let query = "token=abcdef"; // length 12
    let allowed = build_parse_options(|builder| builder.max_length(query.len()));
    parse_with::<Value>(query, &allowed).expect("length at limit should parse");

    let blocked = build_parse_options(|builder| builder.max_length(query.len() - 1));
    asserts::assert_err_matches!(
        parse_with::<Value>(query, &blocked),
        ParseError::InputTooLong { limit } => |_message| {
            assert_eq!(limit, query.len() - 1);
        }
    );
}

#[test]
fn parse_respects_max_depth_boundary() {
    let within = build_parse_options(|builder| builder.max_depth(2));
    let nested: Value = parse_with("a[b][c]=ok", &within).expect("depth 2 should succeed");
    assert_str_path(&nested, &["a", "b", "c"], "ok");

    let over = build_parse_options(|builder| builder.max_depth(2));
    asserts::assert_err_matches!(
        parse_with::<Value>("a[b][c][d]=fail", &over),
        ParseError::DepthExceeded { key, limit } => |_message| {
            assert_eq!(key, "a[b][c][d]");
            assert_eq!(limit, 2);
        }
    );
}

#[test]
fn parse_options_builder_configures_all_fields() {
    let options = build_parse_options(|builder| {
        builder
            .space_as_plus(true)
            .max_params(3)
            .max_length(64)
            .max_depth(4)
    });

    assert!(options.space_as_plus);
    assert_eq!(options.max_params, Some(3));
    assert_eq!(options.max_length, Some(64));
    assert_eq!(options.max_depth, Some(4));
}

#[test]
fn parse_options_builder_rejects_zero_limits() {
    let params_err = try_build_parse_options(|builder| builder.max_params(0))
        .expect_err("zero param limit should be rejected by builder");
    let params_msg = params_err.to_string();
    assert!(
        params_msg.contains("max_params"),
        "expected `{params_msg}` to contain `max_params`"
    );

    let length_err = try_build_parse_options(|builder| builder.max_length(0))
        .expect_err("zero length limit should be rejected by builder");
    let length_msg = length_err.to_string();
    assert!(
        length_msg.contains("max_length"),
        "expected `{length_msg}` to contain `max_length`"
    );

    let depth_err = try_build_parse_options(|builder| builder.max_depth(0))
        .expect_err("zero depth limit should be rejected by builder");
    let depth_msg = depth_err.to_string();
    assert!(
        depth_msg.contains("max_depth"),
        "expected `{depth_msg}` to contain `max_depth`"
    );
}

#[test]
fn parse_combined_limits_prioritize_length_check() {
    let options = build_parse_options(|builder| builder.max_params(5).max_length(5));

    asserts::assert_err_matches!(
        parse_with::<Value>("toolong=value", &options),
        ParseError::InputTooLong { limit } => |_message| {
            assert_eq!(limit, 5);
        }
    );
}

#[test]
fn parse_combined_limits_still_enforce_params() {
    let options = build_parse_options(|builder| builder.max_params(1).max_length(64));

    asserts::assert_err_matches!(
        parse_with::<Value>("a=1&b=2", &options),
        ParseError::TooManyParameters { limit, actual } => |_message| {
            assert_eq!(limit, 1);
            assert_eq!(actual, 2);
        }
    );
}

#[test]
fn parse_combined_limits_respect_depth_even_with_param_budget() {
    let options = build_parse_options(|builder| builder.max_params(10).max_depth(1));

    asserts::assert_err_matches!(
        parse_with::<Value>("a[b][c]=1", &options),
        ParseError::DepthExceeded { key, limit } => |_message| {
            assert_eq!(key, "a[b][c]");
            assert_eq!(limit, 1);
        }
    );
}

#[test]
fn parse_handles_extremely_large_limits_without_overflow() {
    let options = build_parse_options(|builder| {
        builder
            .max_params(usize::MAX)
            .max_length(usize::MAX)
            .max_depth(usize::MAX)
    });

    let parsed: Value = parse_with("a=1&b=2", &options).expect("extreme limits should still parse");
    assert_str_path(&parsed, &["a"], "1");
    assert_str_path(&parsed, &["b"], "2");
}

#[test]
fn parse_options_builder_defaults_match_default() {
    let built = build_parse_options(|builder| builder);
    assert_eq!(built.space_as_plus, ParseOptions::default().space_as_plus);
    assert_eq!(built.max_params, ParseOptions::default().max_params);
    assert_eq!(built.max_length, ParseOptions::default().max_length);
    assert_eq!(built.max_depth, ParseOptions::default().max_depth);
}

#[test]
fn parse_with_builder_space_as_plus_decodes_plus() {
    let options = build_parse_options(|builder| builder.space_as_plus(true));

    let parsed: Value =
        parse_with("msg=hello+world", &options).expect("plus should decode to space");
    assert_str_path(&parsed, &["msg"], "hello world");
}

#[test]
fn parse_combines_space_as_plus_with_length_limit() {
    let query = "note=one+two+three";
    let permissive =
        build_parse_options(|builder| builder.space_as_plus(true).max_length(query.len()));
    let parsed: Value = parse_with(query, &permissive)
        .expect("length within limit should parse with space_as_plus");
    assert_str_path(&parsed, &["note"], "one two three");

    let strict =
        build_parse_options(|builder| builder.space_as_plus(true).max_length(query.len() - 1));
    asserts::assert_err_matches!(
        parse_with::<Value>(query, &strict),
        ParseError::InputTooLong { limit } => |_message| {
            assert_eq!(limit, query.len() - 1);
        }
    );
}

#[test]
fn stringify_options_builder_controls_space_encoding() {
    let map = json_from_pairs(&[("greeting", "hello world")]);
    let options = build_stringify_options(|builder| builder.space_as_plus(true));

    let encoded = stringify_with(&map, &options).expect("should encode with plus");
    assert_eq!(encoded, "greeting=hello+world");
}

#[test]
fn parse_treats_dots_as_literal_without_additional_option() {
    let parsed: Value = parse("profile.name=Ada&profile[meta][timezone]=UTC")
        .expect("dots should be treated as literal characters without extra configuration");
    assert_str_path(&parsed, &["profile.name"], "Ada");
    assert_str_path(&parsed, &["profile", "meta", "timezone"], "UTC");
}

#[test]
fn parse_supports_brackets_after_literal_dot_segments() {
    let parsed: Value = parse("metrics.cpu[0]=low&metrics.cpu[1]=high")
        .expect("literal key segments followed by brackets should form arrays");
    assert_string_array_path(&parsed, &["metrics.cpu"], &["low", "high"]);
}
