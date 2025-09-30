#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/json.rs"]
mod json;
#[path = "common/options.rs"]
mod options;
#[path = "common/serde_helpers.rs"]
mod serde_helpers;

use asserts::{assert_str_path, assert_string_array_path};
use bunner_qs::{ParseError, ParseOptions, SerdeQueryError, parse, parse_with};
use json::json_from_pairs;
use options::try_build_parse_options;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_helpers::assert_parse_roundtrip;
use serde_json::{Value, json};
use std::fmt::Debug;

const BUILD_OK: &str = "parse options builder should succeed";

fn build_parse_options<F>(configure: F) -> ParseOptions
where
    F: FnOnce(ParseOptionsBuilder) -> ParseOptionsBuilder,
{
    try_build_parse_options(configure).expect(BUILD_OK)
}

fn parse_value(query: &str) -> Value {
    parse(query).expect("parse should succeed")
}

fn parse_with_options(query: &str, options: &ParseOptions) -> Value {
    parse_with(query, options).expect("parse should succeed with options")
}

fn expect_invalid_percent_encoding(query: &str) -> (usize, String) {
    let err = parse::<Value>(query).expect_err("expected invalid percent encoding error");
    let message = err.to_string();
    match err {
        ParseError::InvalidPercentEncoding { index } => (index, message),
        other => panic!("expected invalid percent encoding, got {other:?}"),
    }
}

fn expect_unmatched_bracket(query: &str) -> (String, String) {
    let err = parse::<Value>(query).expect_err("expected unmatched bracket error");
    let message = err.to_string();
    match err {
        ParseError::UnmatchedBracket { key } => (key, message),
        other => panic!("expected unmatched bracket, got {other:?}"),
    }
}

fn expect_invalid_character(query: &str) -> (char, usize, String) {
    let err = parse::<Value>(query).expect_err("expected invalid character error");
    let message = err.to_string();
    match err {
        ParseError::InvalidCharacter { character, index } => (character, index, message),
        other => panic!("expected invalid character, got {other:?}"),
    }
}

fn expect_unexpected_question_mark(query: &str) -> (usize, String) {
    let err = parse::<Value>(query).expect_err("expected unexpected question mark error");
    let message = err.to_string();
    match err {
        ParseError::UnexpectedQuestionMark { index } => (index, message),
        other => panic!("expected unexpected question mark, got {other:?}"),
    }
}

fn expect_duplicate_key(query: &str) -> (String, String) {
    let err = parse::<Value>(query).expect_err("expected duplicate key error");
    let message = err.to_string();
    match err {
        ParseError::DuplicateKey { key } => (key, message),
        other => panic!("expected duplicate key, got {other:?}"),
    }
}

fn expect_depth_exceeded(query: &str, options: &ParseOptions) -> (String, usize, String) {
    let err = parse_with::<Value>(query, options).expect_err("expected depth exceeded error");
    let message = err.to_string();
    match err {
        ParseError::DepthExceeded { key, limit } => (key, limit, message),
        other => panic!("expected depth exceeded, got {other:?}"),
    }
}

fn expect_too_many_parameters(query: &str, options: &ParseOptions) -> (usize, usize, String) {
    let err = parse_with::<Value>(query, options).expect_err("expected too many parameters error");
    let message = err.to_string();
    match err {
        ParseError::TooManyParameters { limit, actual } => (limit, actual, message),
        other => panic!("expected too many parameters, got {other:?}"),
    }
}

fn expect_input_too_long(query: &str, options: &ParseOptions) -> (usize, String) {
    let err = parse_with::<Value>(query, options).expect_err("expected input too long error");
    let message = err.to_string();
    match err {
        ParseError::InputTooLong { limit } => (limit, message),
        other => panic!("expected input too long, got {other:?}"),
    }
}

fn expect_invalid_utf8(query: &str) -> String {
    let err = parse::<Value>(query).expect_err("expected invalid utf8 error");
    let message = err.to_string();
    match err {
        ParseError::InvalidUtf8 => message,
        other => panic!("expected invalid utf8, got {other:?}"),
    }
}

fn expect_serde_error<T>(query: &str) -> (String, SerdeQueryError)
where
    T: DeserializeOwned + Default + Debug + 'static,
{
    let err = parse::<T>(query).expect_err("expected serde error");
    let message = err.to_string();
    match err {
        ParseError::Serde(source) => (message, source),
        other => panic!("expected serde error, got {other:?}"),
    }
}

type ParseOptionsBuilder = bunner_qs::ParseOptionsBuilder;

mod basic_parsing_tests {
    use super::*;

    #[test]
    fn when_basic_pairs_are_parsed_it_should_build_expected_map() {
        // Arrange
        let query = "a=1&b=two";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_eq!(parsed, json!({ "a": "1", "b": "two" }));
    }

    #[test]
    fn when_percent_encoded_text_is_parsed_it_should_decode_unicode() {
        // Arrange
        let query = concat!(
            "name=J%C3%BCrgen",
            "&emoji=%F0%9F%98%80",
            "&cyrillic=%D0%9F%D1%80%D0%B8%D0%B2%D0%B5%D1%82",
            "&arabic=%D9%85%D8%B1%D8%AD%D8%A8%D8%A7",
            "&combining=Cafe%CC%81",
            "&thai=%E0%B8%AA%E0%B8%A7%E0%B8%B1%E0%B8%AA%E0%B8%94%E0%B8%B5",
        );

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_str_path(&parsed, &["name"], "Jürgen");
        assert_str_path(&parsed, &["emoji"], "😀");
        assert_str_path(&parsed, &["cyrillic"], "Привет");
        assert_str_path(&parsed, &["arabic"], "مرحبا");
        assert_str_path(&parsed, &["combining"], "Café");
        assert_str_path(&parsed, &["thai"], "สวัสดี");
    }

    #[test]
    fn when_extended_unicode_keys_are_percent_encoded_it_should_roundtrip() {
        // Arrange
        use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

        let key_one = "ключ🌌";
        let value_one = "नमस्ते";
        let key_two = "combinação";
        let value_two = "שָׁלוֹם";
        let query = format!(
            "{}={}&{}={}",
            utf8_percent_encode(key_one, NON_ALPHANUMERIC),
            utf8_percent_encode(value_one, NON_ALPHANUMERIC),
            utf8_percent_encode(key_two, NON_ALPHANUMERIC),
            utf8_percent_encode(value_two, NON_ALPHANUMERIC)
        );

        // Act
        let parsed = parse_value(&query);

        // Assert
        assert_str_path(&parsed, &[key_one], value_one);
        assert_str_path(&parsed, &[key_two], value_two);
    }

    #[test]
    fn when_input_is_empty_it_should_return_null() {
        // Arrange
        let query = "";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_eq!(parsed, Value::Null);
    }

    #[test]
    fn when_query_is_lone_question_mark_it_should_return_null() {
        // Arrange
        let query = "?";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_eq!(parsed, Value::Null);
    }

    #[test]
    fn when_query_has_leading_question_mark_it_should_ignore_prefix() {
        // Arrange
        let query = "?foo=bar&baz=qux";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_eq!(parsed, json_from_pairs(&[("foo", "bar"), ("baz", "qux")]));
    }

    #[test]
    fn when_flag_is_missing_value_it_should_store_empty_string() {
        // Arrange
        let query = "flag";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_str_path(&parsed, &["flag"], "");
    }

    #[test]
    fn when_pairs_include_empty_key_it_should_keep_entry() {
        // Arrange
        let query = "=1&foo=bar";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_eq!(parsed, json_from_pairs(&[("", "1"), ("foo", "bar")]));
    }

    #[test]
    fn when_values_are_explicitly_empty_it_should_store_empty_strings() {
        // Arrange
        let query = "a=&b=2";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_str_path(&parsed, &["a"], "");
        assert_str_path(&parsed, &["b"], "2");
    }

    #[test]
    fn when_flags_and_pairs_mix_it_should_assign_empty_string() {
        // Arrange
        let query = "a=1&b&c=3";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_str_path(&parsed, &["a"], "1");
        assert_str_path(&parsed, &["b"], "");
        assert_str_path(&parsed, &["c"], "3");
    }

    #[test]
    fn when_trailing_ampersands_exist_it_should_ignore_them() {
        // Arrange
        let query = "alpha=beta&&";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_eq!(parsed, json_from_pairs(&[("alpha", "beta")]));
    }
}

mod structure_parsing_tests {
    use super::*;

    #[test]
    fn when_numeric_segment_is_followed_by_field_it_should_parse_object_entry() {
        // Arrange
        let query = "a[0]b=1";

        // Act
        let parsed = parse_value(query);
        let array = parsed
            .get("a")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        // Assert
        assert_eq!(array.len(), 1);
        assert_eq!(array[0].get("b").and_then(Value::as_str), Some("1"));
    }

    #[test]
    fn when_nested_empty_brackets_are_used_it_should_treat_them_as_literals() {
        // Arrange
        let query = "a[[]]=1";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_str_path(&parsed, &["a", "[", "]"], "1");
    }

    #[test]
    fn when_percent_encoded_equals_appears_in_segment_it_should_preserve_literal() {
        // Arrange
        let query = "profile[key%3Dname]=alice";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_str_path(&parsed, &["profile", "key=name"], "alice");
    }

    #[test]
    fn when_nested_objects_and_arrays_are_parsed_it_should_retain_structure() {
        // Arrange
        let query =
            "user[name]=Alice&user[stats][age]=30&user[hobbies][]=reading&user[hobbies][]=coding";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_str_path(&parsed, &["user", "name"], "Alice");
        assert_str_path(&parsed, &["user", "stats", "age"], "30");
        assert_string_array_path(&parsed, &["user", "hobbies"], &["reading", "coding"]);
    }

    #[test]
    fn when_complex_structure_is_round_tripped_it_should_remain_equivalent() {
        // Arrange
        let query = "data[users][0][name]=Alice&data[users][1][name]=Bob&data[meta][version]=1";

        // Act
        assert_parse_roundtrip(query);

        // Assert
        // Assert handled by helper to ensure equality after roundtrip
    }
}

mod option_behavior_tests {
    use super::*;

    #[test]
    fn when_space_as_plus_is_enabled_it_should_convert_plus_to_space() {
        // Arrange
        let options = build_parse_options(|builder| builder.space_as_plus(true));
        let query = "note=one+two";

        // Act
        let with_option = parse_with_options(query, &options);
        let without_option = parse_value(query);

        // Assert
        assert_str_path(&with_option, &["note"], "one two");
        assert_str_path(&without_option, &["note"], "one+two");
    }

    #[test]
    fn when_builder_sets_limits_it_should_store_configuration() {
        // Arrange
        let options = build_parse_options(|builder| {
            builder
                .space_as_plus(true)
                .max_params(3)
                .max_length(128)
                .max_depth(2)
        });

        // Act
        let extracted = (
            options.space_as_plus,
            options.max_params,
            options.max_length,
            options.max_depth,
        );

        // Assert
        assert_eq!(extracted, (true, Some(3), Some(128), Some(2)));
    }

    #[test]
    fn when_parameter_and_length_limits_are_enforced_it_should_return_errors() {
        // Arrange
        let param_limited = build_parse_options(|builder| builder.max_params(1));
        let length_limited = build_parse_options(|builder| builder.max_length(5));

        // Act
        let param_error = expect_too_many_parameters("a=1&b=2", &param_limited);
        let length_error = expect_input_too_long("toolong=1", &length_limited);

        // Assert
        assert_eq!(param_error.0, 1);
        assert_eq!(param_error.1, 2);
        assert_eq!(param_error.2, "too many parameters: received 2, limit 1");
        assert_eq!(length_error.0, 5);
        assert_eq!(
            length_error.1,
            "input exceeds maximum length of 5 characters"
        );
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn when_percent_encoding_is_incomplete_it_should_report_index() {
        // Arrange
        let query = "bad=%2";

        // Act
        let (index, message) = expect_invalid_percent_encoding(query);

        // Assert
        assert_eq!(index, 4);
        assert_eq!(message, "invalid percent-encoding at byte offset 4");
    }

    #[test]
    fn when_percent_encoding_has_invalid_digits_it_should_report_index() {
        // Arrange
        let query = "bad=%ZZ";

        // Act
        let (index, _) = expect_invalid_percent_encoding(query);

        // Assert
        assert_eq!(index, 4);
    }

    #[test]
    fn when_closing_bracket_is_unmatched_it_should_return_key() {
        // Arrange
        let query = "a]=1";

        // Act
        let (key, _) = expect_unmatched_bracket(query);

        // Assert
        assert_eq!(key, "a]");
    }

    #[test]
    fn when_unencoded_equals_inside_segment_it_should_report_unmatched_bracket() {
        // Arrange
        let query = "profile[key=name]=alice";

        // Act
        let (key, _) = expect_unmatched_bracket(query);

        // Assert
        assert_eq!(key, "profile[key");
    }

    #[test]
    fn when_control_character_is_present_it_should_report_position() {
        // Arrange
        let input = format!("bad{}key=1", '\u{0007}');

        // Act
        let (character, index, message) = expect_invalid_character(&input);

        // Assert
        assert_eq!(character, '\u{0007}');
        assert_eq!(index, 3);
        assert_eq!(
            message,
            "query contains invalid character `\u{7}` at byte offset 3"
        );
    }

    #[test]
    fn when_question_mark_appears_in_key_it_should_report_unexpected_character() {
        // Arrange
        let query = "foo?bar=1";

        // Act
        let (index, message) = expect_unexpected_question_mark(query);

        // Assert
        assert_eq!(index, 3);
        assert_eq!(
            message,
            "unexpected '?' character inside query at byte offset 3"
        );
    }

    #[test]
    fn when_raw_space_is_in_key_it_should_return_invalid_character_error() {
        // Arrange
        let query = "bad key=1";

        // Act
        let (character, index, message) = expect_invalid_character(query);

        // Assert
        assert_eq!(character, ' ');
        assert_eq!(index, 3);
        assert_eq!(
            message,
            "query contains invalid character ` ` at byte offset 3"
        );
    }

    #[test]
    fn when_brackets_are_unmatched_or_depth_exceeds_limit_it_should_report_errors() {
        // Arrange
        let depth_limited = build_parse_options(|builder| builder.max_depth(1));

        // Act
        let (key, message) = expect_unmatched_bracket("a[=1");
        let (depth_key, limit, depth_message) = expect_depth_exceeded("a[b][c]=1", &depth_limited);

        // Assert
        assert_eq!(key, "a[");
        assert_eq!(message, "unmatched bracket sequence in key 'a['");
        assert_eq!(depth_key, "a[b][c]");
        assert_eq!(limit, 1);
        assert_eq!(
            depth_message,
            "maximum bracket depth exceeded in key 'a[b][c]' (limit 1)"
        );
    }

    #[test]
    fn when_duplicate_keys_appear_it_should_report_conflict() {
        // Arrange
        let query = "color=red&color=blue";

        // Act
        let (key, message) = expect_duplicate_key(query);

        // Assert
        assert_eq!(key, "color");
        assert_eq!(message, "duplicate key 'color' not allowed");
    }

    #[test]
    fn when_array_indices_are_sparse_it_should_return_duplicate_key_error() {
        // Arrange
        let query = "items[0]=apple&items[2]=cherry";

        // Act
        let (key, _) = expect_duplicate_key(query);

        // Assert
        assert_eq!(key, "items");
    }

    #[test]
    fn when_percent_decoding_yields_invalid_utf8_it_should_report_failure() {
        // Arrange
        let query = "bad=%FF";

        // Act
        let message = expect_invalid_utf8(query);

        // Assert
        assert_eq!(message, "decoded component is not valid UTF-8");
    }
}

mod serde_integration_tests {
    use super::*;

    #[test]
    fn when_deserializing_into_struct_it_should_report_human_readable_error() {
        // Arrange
        #[derive(Debug, Deserialize, Default)]
        struct NumericTarget {
            #[serde(rename = "count")]
            _count: u32,
        }

        // Act
        let (message, source) = expect_serde_error::<NumericTarget>("count=abc");

        // Assert
        assert_eq!(
            message,
            "failed to deserialize parsed query into target type: failed to deserialize query map: invalid number literal `abc`"
        );
        match source {
            SerdeQueryError::Deserialize(inner) => {
                assert_eq!(inner.to_string(), "invalid number literal `abc`");
            }
            other => panic!("unexpected inner serde error: {other:?}"),
        }
    }
}
