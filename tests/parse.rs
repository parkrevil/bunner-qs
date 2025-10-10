#[path = "common/api.rs"]
mod api;
#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/json.rs"]
mod json;
#[path = "common/serde_helpers.rs"]
mod serde_helpers;

use api::{build_parse_options as api_build_parse_options, parse_default, parse_query};
use asserts::{assert_str_path, assert_string_array_path};
use bunner_qs_rs::parsing::ParseError;
use bunner_qs_rs::parsing::errors::DeserializeError;
use bunner_qs_rs::parsing::errors::ParseLocation;
use bunner_qs_rs::{DuplicateKeyBehavior, ParseOptions, QsParseError};
use json::json_from_pairs;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_helpers::assert_parse_roundtrip;
use serde_json::{Value, json};
use std::fmt::Debug;

const BUILD_OK: &str = "parse options configuration should succeed";

fn build_parse_options<F>(configure: F) -> ParseOptions
where
    F: FnOnce(ParseOptions) -> ParseOptions,
{
    api_build_parse_options(configure).expect(BUILD_OK)
}

fn parse_value(query: &str) -> Value {
    parse_default(query).unwrap_or_else(|e| match e {
        QsParseError::Parse(err) => panic!("parse should succeed but got: {}", err),
        QsParseError::MissingParseOptions => unreachable!(),
    })
}

fn parse_with_options(query: &str, options: &ParseOptions) -> Value {
    parse_query(query, options).unwrap_or_else(|e| match e {
        QsParseError::Parse(err) => panic!("parse should succeed with options but got: {}", err),
        QsParseError::MissingParseOptions => unreachable!(),
    })
}

fn expect_invalid_percent_encoding(query: &str) -> (usize, ParseLocation, String) {
    let err = parse_default::<Value>(query).expect_err("expected invalid percent encoding error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::InvalidPercentEncoding { index, location }) => {
            (index, location, message)
        }
        other => panic!("expected invalid percent encoding, got {other:?}"),
    }
}

fn expect_unmatched_bracket(query: &str) -> (String, char, String) {
    let err = parse_default::<Value>(query).expect_err("expected unmatched bracket error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::UnmatchedBracket { key, bracket }) => {
            (key, bracket, message)
        }
        other => panic!("expected unmatched bracket, got {other:?}"),
    }
}

fn expect_invalid_character(query: &str) -> (char, usize, ParseLocation, String) {
    let err = parse_default::<Value>(query).expect_err("expected invalid character error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::InvalidCharacter {
            character,
            index,
            location,
        }) => (character, index, location, message),
        other => panic!("expected invalid character, got {other:?}"),
    }
}

fn expect_unexpected_question_mark(query: &str) -> (usize, ParseLocation, String) {
    let err = parse_default::<Value>(query).expect_err("expected unexpected question mark error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::UnexpectedQuestionMark { index, location }) => {
            (index, location, message)
        }
        other => panic!("expected unexpected question mark, got {other:?}"),
    }
}

fn expect_duplicate_key(query: &str) -> (String, String) {
    let err = parse_default::<Value>(query).expect_err("expected duplicate key error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::DuplicateRootKey { key }) => (key, message),
        QsParseError::Parse(ParseError::DuplicateMapEntry { segment, .. }) => (segment, message),
        QsParseError::Parse(ParseError::DuplicateSequenceIndex { index, .. }) => {
            (index.to_string(), message)
        }
        QsParseError::Parse(ParseError::InvalidSequenceIndex { segment, .. }) => (segment, message),
        QsParseError::Parse(ParseError::NestedValueConflict { parent }) => (parent, message),
        QsParseError::Parse(ParseError::KeyPatternConflict { segment, .. }) => (segment, message),
        other => panic!("expected duplicate key, got {other:?}"),
    }
}

fn expect_depth_exceeded(query: &str, options: &ParseOptions) -> (String, usize, usize, String) {
    let err = parse_query::<Value>(query, options).expect_err("expected depth exceeded error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::DepthExceeded { key, limit, depth }) => {
            (key, limit, depth, message)
        }
        other => panic!("expected depth exceeded, got {other:?}"),
    }
}

fn expect_too_many_parameters(query: &str, options: &ParseOptions) -> (usize, usize, String) {
    let err = parse_query::<Value>(query, options).expect_err("expected too many parameters error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::TooManyParameters { limit, actual }) => {
            (limit, actual, message)
        }
        other => panic!("expected too many parameters, got {other:?}"),
    }
}

fn expect_input_too_long(query: &str, options: &ParseOptions) -> (usize, usize, String) {
    let err = parse_query::<Value>(query, options).expect_err("expected input too long error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::InputTooLong { limit, actual }) => (limit, actual, message),
        other => panic!("expected input too long, got {other:?}"),
    }
}

fn expect_invalid_utf8(query: &str) -> (ParseLocation, String) {
    let err = parse_default::<Value>(query).expect_err("expected invalid utf8 error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::InvalidUtf8 { location }) => (location, message),
        other => panic!("expected invalid utf8, got {other:?}"),
    }
}

fn expect_serde_error<T>(query: &str) -> (String, DeserializeError)
where
    T: DeserializeOwned + Default + Debug + 'static,
{
    let err = parse_default::<T>(query).expect_err("expected serde error");
    let message = err.to_string();
    match err {
        QsParseError::Parse(ParseError::Serde(source)) => (message, source),
        other => panic!("expected serde error, got {other:?}"),
    }
}

mod basic_parsing_tests {
    use super::*;

    #[test]
    fn should_parse_basic_pairs_into_expected_map_when_query_contains_two_pairs_then_produce_flat_object_with_pairs()
     {
        let query = "a=1&b=two";

        let parsed = parse_value(query);

        assert_eq!(parsed, json!({ "a": "1", "b": "two" }));
    }

    #[test]
    fn should_decode_unicode_percent_encoded_text_when_query_contains_international_values_then_decode_international_characters_correctly()
     {
        let query = concat!(
            "name=J%C3%BCrgen",
            "&emoji=%F0%9F%98%80",
            "&cyrillic=%D0%9F%D1%80%D0%B8%D0%B2%D0%B5%D1%82",
            "&arabic=%D9%85%D8%B1%D8%AD%D8%A8%D8%A7",
            "&combining=Cafe%CC%81",
            "&thai=%E0%B8%AA%E0%B8%A7%E0%B8%B1%E0%B8%AA%E0%B8%94%E0%B8%B5",
        );

        let parsed = parse_value(query);

        assert_str_path(&parsed, &["name"], "Jürgen");
        assert_str_path(&parsed, &["emoji"], "😀");
        assert_str_path(&parsed, &["cyrillic"], "Привет");
        assert_str_path(&parsed, &["arabic"], "مرحبا");
        assert_str_path(&parsed, &["combining"], "Café");
        assert_str_path(&parsed, &["thai"], "สวัสดี");
    }

    #[test]
    fn should_roundtrip_percent_encoded_unicode_keys_when_query_contains_multilingual_pairs_then_restore_original_multilingual_keys_and_values()
     {
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

        let parsed = parse_value(&query);

        assert_str_path(&parsed, &[key_one], value_one);
        assert_str_path(&parsed, &[key_two], value_two);
    }

    #[test]
    fn should_parse_null_value_when_query_is_empty_then_return_null_value() {
        let query = "";

        let parsed = parse_value(query);

        assert_eq!(parsed, Value::Null);
    }

    #[test]
    fn should_parse_null_value_when_query_is_lone_question_mark_then_return_null_value_for_prefix_only()
     {
        let query = "?";

        let parsed = parse_value(query);

        assert_eq!(parsed, Value::Null);
    }

    #[test]
    fn should_ignore_leading_question_mark_when_query_has_prefix_then_strip_prefix_and_parse_pairs()
    {
        let query = "?foo=bar&baz=qux";

        let parsed = parse_value(query);

        assert_eq!(parsed, json_from_pairs(&[("foo", "bar"), ("baz", "qux")]));
    }

    #[test]
    fn should_store_empty_string_when_flag_without_value_present_then_store_empty_string_for_flag()
    {
        let query = "flag";

        let parsed = parse_value(query);

        assert_str_path(&parsed, &["flag"], "");
    }

    #[test]
    fn should_keep_entry_for_empty_key_when_query_starts_with_equals_then_preserve_entry_with_empty_key()
     {
        let query = "=1&foo=bar";

        let parsed = parse_value(query);

        assert_eq!(parsed, json_from_pairs(&[("", "1"), ("foo", "bar")]));
    }

    #[test]
    fn should_store_empty_strings_when_values_are_explicitly_empty_then_store_empty_string_values()
    {
        let query = "a=&b=2";

        let parsed = parse_value(query);

        assert_str_path(&parsed, &["a"], "");
        assert_str_path(&parsed, &["b"], "2");
    }

    #[test]
    fn should_assign_empty_string_when_flags_and_pairs_are_mixed_then_assign_empty_string_to_flag_entries()
     {
        let query = "a=1&b&c=3";

        let parsed = parse_value(query);

        assert_str_path(&parsed, &["a"], "1");
        assert_str_path(&parsed, &["b"], "");
        assert_str_path(&parsed, &["c"], "3");
    }

    #[test]
    fn should_ignore_trailing_ampersands_when_query_has_extra_separators_then_ignore_trailing_separators()
     {
        let query = "alpha=beta&&";

        let parsed = parse_value(query);

        assert_eq!(parsed, json_from_pairs(&[("alpha", "beta")]));
    }
}

mod structure_parsing_tests {
    use super::*;

    #[test]
    fn should_parse_object_entry_after_numeric_segment_when_nested_key_present_then_attach_nested_object_to_numeric_index()
     {
        let query = "a[0]b=1";

        let parsed = parse_value(query);
        let array = parsed
            .get("a")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        assert_eq!(array.len(), 1);
        assert_eq!(array[0].get("b").and_then(Value::as_str), Some("1"));
    }

    #[test]
    fn should_treat_nested_empty_brackets_as_literals_when_parsing_keys_then_treat_empty_brackets_as_literal_segments()
     {
        let query = "a[[]]=1";

        let parsed = parse_value(query);

        assert_str_path(&parsed, &["a", "[", "]"], "1");
    }

    #[test]
    fn should_preserve_percent_encoded_equals_when_decoding_segments_then_decode_key_with_percent_encoded_equals()
     {
        let query = "profile[key%3Dname]=alice";

        let parsed = parse_value(query);

        assert_str_path(&parsed, &["profile", "key=name"], "alice");
    }

    #[test]
    fn should_retain_structure_when_nested_objects_and_arrays_present_then_reconstruct_nested_objects_and_arrays()
     {
        let query =
            "user[name]=Alice&user[stats][age]=30&user[hobbies][]=reading&user[hobbies][]=coding";

        let parsed = parse_value(query);

        assert_str_path(&parsed, &["user", "name"], "Alice");
        assert_str_path(&parsed, &["user", "stats", "age"], "30");
        assert_string_array_path(&parsed, &["user", "hobbies"], &["reading", "coding"]);
    }

    #[test]
    fn should_roundtrip_complex_structure_when_query_contains_nested_data_then_roundtrip_complex_nested_query()
     {
        let query = "data[users][0][name]=Alice&data[users][1][name]=Bob&data[meta][version]=1";

        assert_parse_roundtrip(query);
    }
}

mod option_behavior_tests {
    use super::*;

    #[test]
    fn should_convert_plus_to_space_when_space_as_plus_option_enabled_then_decode_plus_as_space_only_with_option()
     {
        let options = build_parse_options(|builder| builder.space_as_plus(true));
        let query = "note=one+two";

        let with_option = parse_with_options(query, &options);
        let without_option = parse_value(query);

        assert_str_path(&with_option, &["note"], "one two");
        assert_str_path(&without_option, &["note"], "one+two");
    }

    #[test]
    fn should_store_configuration_when_builder_sets_multiple_limits_then_capture_configured_limit_options()
     {
        let options = build_parse_options(|builder| {
            builder
                .space_as_plus(true)
                .max_params(3)
                .max_length(128)
                .max_depth(2)
        });

        let extracted = (
            options.space_as_plus,
            options.max_params,
            options.max_length,
            options.max_depth,
        );

        assert_eq!(extracted, (true, Some(3), Some(128), Some(2)));
    }

    #[test]
    fn should_return_errors_when_parameter_and_length_limits_are_enforced_then_report_parameter_and_length_violations()
     {
        let param_limited = build_parse_options(|builder| builder.max_params(1));
        let length_limited = build_parse_options(|builder| builder.max_length(5));

        let param_error = expect_too_many_parameters("a=1&b=2", &param_limited);
        let length_error = expect_input_too_long("toolong=1", &length_limited);

        let (param_limit, param_actual, param_message) = param_error;
        let (length_limit, length_actual, length_message) = length_error;

        assert_eq!(param_limit, 1);
        assert_eq!(param_actual, 2);
        assert_eq!(param_message, "too many parameters: received 2, limit 1");
        assert_eq!(length_limit, 5);
        assert_eq!(length_actual, "toolong=1".len());
        assert_eq!(
            length_message,
            format!(
                "input exceeds maximum length of 5 characters (received {})",
                length_actual
            )
        );
    }

    #[test]
    fn should_keep_initial_values_when_duplicate_keys_use_first_wins_then_keep_first_value_for_duplicates()
     {
        let options =
            build_parse_options(|builder| builder.duplicate_keys(DuplicateKeyBehavior::FirstWins));
        let query = "color=red&color=blue&user[name]=Alice&user[name]=Bob";

        let parsed = parse_with_options(query, &options);

        assert_str_path(&parsed, &["color"], "red");
        assert_str_path(&parsed, &["user", "name"], "Alice");
    }

    #[test]
    fn should_replace_with_latest_values_when_duplicate_keys_use_last_wins_then_keep_last_value_for_duplicates()
     {
        let options =
            build_parse_options(|builder| builder.duplicate_keys(DuplicateKeyBehavior::LastWins));
        let query = "color=red&color=blue&user[name]=Alice&user[name]=Bob";

        let parsed = parse_with_options(query, &options);

        assert_str_path(&parsed, &["color"], "blue");
        assert_str_path(&parsed, &["user", "name"], "Bob");
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn should_report_index_when_percent_encoding_is_incomplete_then_report_offset_for_incomplete_percent_encoding()
     {
        let query = "bad=%2";

        let (index, location, message) = expect_invalid_percent_encoding(query);

        assert_eq!(index, 4);
        assert_eq!(location, ParseLocation::Value);
        assert_eq!(
            message,
            "invalid percent-encoding in value at byte offset 4"
        );
    }

    #[test]
    fn should_report_index_when_percent_encoding_has_invalid_digits_then_report_offset_for_invalid_percent_digits()
     {
        let query = "bad=%ZZ";

        let (index, location, _) = expect_invalid_percent_encoding(query);

        assert_eq!(index, 4);
        assert_eq!(location, ParseLocation::Value);
    }

    #[test]
    fn should_return_key_when_closing_bracket_is_unmatched_then_return_problematic_key_for_unmatched_bracket()
     {
        let query = "a]=1";

        let (key, bracket, _) = expect_unmatched_bracket(query);

        assert_eq!(key, "a]");
        assert_eq!(bracket, ']');
    }

    #[test]
    fn should_report_unmatched_bracket_when_equals_is_unencoded_then_return_prefix_key_for_unclosed_bracket()
     {
        let query = "profile[key=name]=alice";

        let (key, bracket, _) = expect_unmatched_bracket(query);

        assert_eq!(key, "profile[key");
        assert_eq!(bracket, '[');
    }

    #[test]
    fn should_report_position_when_control_character_present_in_key_then_report_offset_for_control_character_in_key()
     {
        let input = format!("bad{}key=1", '\u{0007}');

        let (character, index, location, message) = expect_invalid_character(&input);

        assert_eq!(character, '\u{0007}');
        assert_eq!(index, 3);
        assert_eq!(location, ParseLocation::Query);
        assert_eq!(
            message,
            "invalid character `\u{7}` in query at byte offset 3"
        );
    }

    #[test]
    fn should_report_position_when_percent_decoding_control_character_then_report_offset_for_decoded_control_character()
     {
        let query = "bad=%07";

        let (character, index, location, message) = expect_invalid_character(query);

        assert_eq!(character, '\u{0007}');
        assert_eq!(index, 4);
        assert_eq!(location, ParseLocation::Value);
        assert_eq!(
            message,
            "invalid character `\u{7}` in value at byte offset 4"
        );
    }

    #[test]
    fn should_report_unexpected_character_when_question_mark_in_key_then_report_unexpected_question_mark_position()
     {
        let query = "foo?bar=1";

        let (index, location, message) = expect_unexpected_question_mark(query);

        assert_eq!(index, 3);
        assert_eq!(location, ParseLocation::Query);
        assert_eq!(
            message,
            "unexpected '?' character in query at byte offset 3"
        );
    }

    #[test]
    fn should_return_invalid_character_error_when_raw_space_present_in_key_then_report_space_character_violation()
     {
        let query = "bad key=1";

        let (character, index, location, message) = expect_invalid_character(query);

        assert_eq!(character, ' ');
        assert_eq!(index, 3);
        assert_eq!(location, ParseLocation::Query);
        assert_eq!(message, "invalid character ` ` in query at byte offset 3");
    }

    #[test]
    fn should_report_errors_when_brackets_unmatched_and_depth_limit_exceeded_then_surface_unmatched_bracket_and_depth_errors()
     {
        let depth_limited = build_parse_options(|builder| builder.max_depth(1));

        let (key, bracket, message) = expect_unmatched_bracket("a[=1");
        let (depth_key, limit, depth, depth_message) =
            expect_depth_exceeded("a[b][c]=1", &depth_limited);

        assert_eq!(key, "a[");
        assert_eq!(bracket, '[');
        assert_eq!(message, "unmatched '[' bracket sequence in key 'a['");
        assert_eq!(depth_key, "a[b][c]");
        assert_eq!(limit, 1);
        assert_eq!(depth, 2);
        assert_eq!(
            depth_message,
            "maximum bracket depth exceeded in key 'a[b][c]' (depth 2, limit 1)"
        );
    }

    #[test]
    fn should_report_conflict_when_duplicate_keys_appear_then_fail_with_duplicate_key_error() {
        let query = "color=red&color=blue";

        let (key, message) = expect_duplicate_key(query);

        assert_eq!(key, "color");
        assert_eq!(message, "duplicate root key 'color' not allowed");
    }

    #[test]
    fn should_materialize_sparse_array_when_indices_skip_values_then_materialize_sparse_array_with_placeholders()
     {
        let query = "items[0]=apple&items[2]=cherry";

        let parsed = parse_value(query);
        let expected = json!({
            "items": ["apple", "", "cherry"],
        });

        assert_eq!(parsed, expected);
    }

    #[test]
    fn should_pad_missing_indices_when_first_element_is_non_zero_then_pad_missing_indices_with_empty_strings()
     {
        let query = "items[1]=late";

        let parsed = parse_value(query);
        let expected = json!({
            "items": ["", "late"],
        });

        assert_eq!(parsed, expected);
    }

    #[test]
    fn should_report_failure_when_percent_decoding_yields_invalid_utf8_then_fail_with_invalid_utf8_error()
     {
        let query = "bad=%FF";

        let (location, message) = expect_invalid_utf8(query);

        assert_eq!(location, ParseLocation::Value);
        assert_eq!(message, "decoded component in value is not valid UTF-8");
    }
}

mod serde_integration_tests {
    use super::*;

    #[test]
    fn should_report_human_readable_error_when_deserializing_into_struct_fails_then_surface_human_readable_deserialize_error()
     {
        #[derive(Debug, Deserialize, Default)]
        struct NumericTarget {
            #[serde(rename = "count")]
            _count: u32,
        }

        let (message, source) = expect_serde_error::<NumericTarget>("count=abc");

        assert_eq!(
            message,
            "failed to deserialize parsed query into target type: invalid number literal `abc` at count"
        );
        assert_eq!(source.to_string(), "invalid number literal `abc` at count");
    }
}
