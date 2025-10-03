#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/json.rs"]
mod json;
#[path = "common/options.rs"]
mod options;
#[path = "common/stringify_options.rs"]
mod stringify_options;

use asserts::assert_str_path;
use bunner_qs::{ParseError, ParseOptions, StringifyOptions, parse, parse_with, stringify_with};
use json::json_from_pairs;
use options::try_build_parse_options;
use serde_json::Value;
use stringify_options::try_build_stringify_options;

const BUILD_OK: &str = "parse options builder should succeed";
const STRINGIFY_BUILD_OK: &str = "stringify options builder should succeed";

fn build_parse_options<F>(configure: F) -> ParseOptions
where
    F: FnOnce(ParseOptionsBuilder) -> ParseOptionsBuilder,
{
    try_build_parse_options(configure).expect(BUILD_OK)
}

fn parse_with_options(query: &str, options: &ParseOptions) -> Value {
    parse_with(query, options).expect("parse should succeed with provided options")
}

fn parse_value(query: &str) -> Value {
    parse(query).expect("parse should succeed")
}

fn expect_too_many_parameters(query: &str, options: &ParseOptions) -> (usize, usize) {
    match parse_with::<Value>(query, options).expect_err("should exceed parameter limit") {
        ParseError::TooManyParameters { limit, actual } => (limit, actual),
        other => panic!("expected too many parameters error, got {other:?}"),
    }
}

fn expect_input_too_long(query: &str, options: &ParseOptions) -> usize {
    match parse_with::<Value>(query, options).expect_err("should exceed length limit") {
        ParseError::InputTooLong { limit } => limit,
        other => panic!("expected input too long error, got {other:?}"),
    }
}

fn expect_depth_exceeded(query: &str, options: &ParseOptions) -> (String, usize) {
    match parse_with::<Value>(query, options).expect_err("should exceed depth limit") {
        ParseError::DepthExceeded { key, limit } => (key, limit),
        other => panic!("expected depth exceeded error, got {other:?}"),
    }
}

fn expect_builder_error<F>(configure: F) -> String
where
    F: FnOnce(ParseOptionsBuilder) -> ParseOptionsBuilder,
{
    try_build_parse_options(configure)
        .expect_err("builder should reject configuration")
        .to_string()
}

fn build_stringify_options<F>(configure: F) -> StringifyOptions
where
    F: FnOnce(StringifyOptionsBuilder) -> StringifyOptionsBuilder,
{
    try_build_stringify_options(configure).expect(STRINGIFY_BUILD_OK)
}

fn stringify_with_options_map<F>(map: &Value, configure: F) -> String
where
    F: FnOnce(StringifyOptionsBuilder) -> StringifyOptionsBuilder,
{
    let options = build_stringify_options(configure);
    stringify_with(map, &options).expect("stringify should succeed")
}

type ParseOptionsBuilder = bunner_qs::ParseOptionsBuilder;
type StringifyOptionsBuilder = bunner_qs::StringifyOptionsBuilder;

mod parse_limits_tests {
    use super::*;

    #[test]
    fn should_parse_within_max_params_limit_when_limits_allow() {
        let options = build_parse_options(|builder| builder.max_params(2));

        let parsed = parse_with_options("a=1&b=2", &options);
        let (limit, actual) = expect_too_many_parameters("a=1&b=2&c=3", &options);

        assert_str_path(&parsed, &["a"], "1");
        assert_str_path(&parsed, &["b"], "2");
        assert_eq!((limit, actual), (2, 3));
    }

    #[test]
    fn should_error_on_first_pair_when_max_params_is_zero() {
        let options = ParseOptions {
            max_params: Some(0),
            ..ParseOptions::default()
        };

        let (limit, actual) = expect_too_many_parameters("only=one", &options);

        assert_eq!((limit, actual), (0, 1));
    }

    #[test]
    fn should_respect_length_boundary_when_limit_blocks_overflow() {
        let query = "token=abcdef";
        let allowed = build_parse_options(|builder| builder.max_length(query.len()));
        let blocked = build_parse_options(|builder| builder.max_length(query.len() - 1));

        let parsed = parse_with_options(query, &allowed);
        let limit = expect_input_too_long(query, &blocked);

        assert_str_path(&parsed, &["token"], "abcdef");
        assert_eq!(limit, query.len() - 1);
    }

    #[test]
    fn should_report_error_when_depth_limit_is_exceeded() {
        let options = build_parse_options(|builder| builder.max_depth(2));

        let nested = parse_with_options("a[b][c]=ok", &options);
        let (key, limit) = expect_depth_exceeded("a[b][c][d]=fail", &options);

        assert_str_path(&nested, &["a", "b", "c"], "ok");
        assert_eq!(key, "a[b][c][d]");
        assert_eq!(limit, 2);
    }

    #[test]
    fn should_prioritize_length_limit_when_length_and_params_conflict() {
        let options = build_parse_options(|builder| builder.max_params(5).max_length(5));

        let limit = expect_input_too_long("toolong=value", &options);

        assert_eq!(limit, 5);
    }

    #[test]
    fn should_error_on_parameters_when_param_limit_is_low() {
        let options = build_parse_options(|builder| builder.max_params(1).max_length(64));

        let (limit, actual) = expect_too_many_parameters("a=1&b=2", &options);

        assert_eq!((limit, actual), (1, 2));
    }

    #[test]
    fn should_error_when_depth_limit_combines_with_param_budget() {
        let options = build_parse_options(|builder| builder.max_params(10).max_depth(1));

        let (key, limit) = expect_depth_exceeded("a[b][c]=1", &options);

        assert_eq!(key, "a[b][c]");
        assert_eq!(limit, 1);
    }

    #[test]
    fn should_parse_successfully_when_limits_are_extreme() {
        let options = build_parse_options(|builder| {
            builder
                .max_params(usize::MAX)
                .max_length(usize::MAX)
                .max_depth(usize::MAX)
        });

        let parsed = parse_with_options("a=1&b=2", &options);

        assert_str_path(&parsed, &["a"], "1");
        assert_str_path(&parsed, &["b"], "2");
    }

    #[test]
    fn should_error_on_parameter_limit_even_when_length_threshold_is_met() {
        let query = "a=1&b=second";
        let options = build_parse_options(|builder| builder.max_params(1).max_length(query.len()));

        let (limit, actual) = expect_too_many_parameters(query, &options);

        assert_eq!((limit, actual), (1, 2));
    }
}

mod parse_builder_tests {
    use super::*;

    #[test]
    fn should_store_values_when_builder_sets_all_fields() {
        let options = build_parse_options(|builder| {
            builder
                .space_as_plus(true)
                .max_params(3)
                .max_length(64)
                .max_depth(4)
        });

        let extracted = (
            options.space_as_plus,
            options.max_params,
            options.max_length,
            options.max_depth,
        );

        assert_eq!(extracted, (true, Some(3), Some(64), Some(4)));
    }

    #[test]
    fn should_fail_when_builder_receives_zero_limits() {
        let params_msg = expect_builder_error(|builder| builder.max_params(0));
        let length_msg = expect_builder_error(|builder| builder.max_length(0));
        let depth_msg = expect_builder_error(|builder| builder.max_depth(0));

        let matches = [
            params_msg.contains("max_params"),
            length_msg.contains("max_length"),
            depth_msg.contains("max_depth"),
        ];

        assert_eq!(matches, [true, true, true]);
    }

    #[test]
    fn should_match_defaults_when_builder_uses_defaults() {
        let built = build_parse_options(|builder| builder);
        let defaults = ParseOptions::default();

        let comparisons = (
            built.space_as_plus == defaults.space_as_plus,
            built.max_params == defaults.max_params,
            built.max_length == defaults.max_length,
            built.max_depth == defaults.max_depth,
        );

        assert_eq!(comparisons, (true, true, true, true));
    }

    #[test]
    fn should_decode_plus_when_space_as_plus_is_enabled() {
        let options = build_parse_options(|builder| builder.space_as_plus(true));

        let parsed = parse_with_options("msg=hello+world", &options);

        assert_str_path(&parsed, &["msg"], "hello world");
    }

    #[test]
    fn should_enforce_length_when_space_as_plus_is_combined_with_limit() {
        let query = "note=one+two+three";
        let permissive =
            build_parse_options(|builder| builder.space_as_plus(true).max_length(query.len()));
        let strict =
            build_parse_options(|builder| builder.space_as_plus(true).max_length(query.len() - 1));

        let parsed = parse_with_options(query, &permissive);
        let limit = expect_input_too_long(query, &strict);

        assert_str_path(&parsed, &["note"], "one two three");
        assert_eq!(limit, query.len() - 1);
    }
}

mod stringify_option_tests {
    use super::*;

    #[test]
    fn should_emit_plus_when_stringify_space_as_plus_is_enabled() {
        let map = json_from_pairs(&[("greeting", "hello world")]);

        let encoded = stringify_with_options_map(&map, |builder| builder.space_as_plus(true));

        assert_eq!(encoded, "greeting=hello+world");
    }
}

mod literal_behavior_tests {
    use super::*;

    #[test]
    fn should_treat_keys_with_dots_as_literals_when_no_brackets_follow() {
        let parsed = parse_value("profile.name=Ada&profile[meta][timezone]=UTC");

        let values = (
            parsed.get("profile.name").and_then(Value::as_str),
            parsed
                .get("profile")
                .and_then(Value::as_object)
                .and_then(|profile| profile.get("meta"))
                .and_then(Value::as_object)
                .and_then(|meta| meta.get("timezone"))
                .and_then(Value::as_str),
        );

        assert_eq!(values.0, Some("Ada"));
        assert_eq!(values.1, Some("UTC"));
    }

    #[test]
    fn should_form_arrays_when_dotted_keys_are_followed_by_brackets() {
        let parsed = parse_value("metrics.cpu[0]=low&metrics.cpu[1]=high");

        let array = parsed
            .get("metrics.cpu")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        assert_eq!(array.len(), 2);
        assert_eq!(array[0], Value::from("low"));
        assert_eq!(array[1], Value::from("high"));
    }
}
