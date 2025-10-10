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
use bunner_qs_rs::{DuplicateKeyBehavior, ParseOptions, QsParseError};
use json::json_from_pairs;
use serde_helpers::assert_parse_roundtrip;
use serde_json::{Value, json};

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

mod parse_error_by_variant_tests {
    use super::*;
    use bunner_qs_rs::parsing::ParseError;
    use bunner_qs_rs::parsing::errors::ParseLocation;
    use serde::Deserialize;

    fn expect_error(query: &str) -> ParseError {
        match parse_default::<Value>(query) {
            Err(QsParseError::Parse(error)) => error,
            Err(QsParseError::MissingParseOptions) => {
                unreachable!("parse options must be configured before parsing")
            }
            Ok(value) => panic!("expected parse error but succeeded with {value:?}"),
        }
    }

    fn expect_error_with_options(query: &str, options: &ParseOptions) -> ParseError {
        match parse_query::<Value>(query, options) {
            Err(QsParseError::Parse(error)) => error,
            Err(QsParseError::MissingParseOptions) => {
                unreachable!("parse options must be configured before parsing")
            }
            Ok(value) => panic!("expected parse error but succeeded with {value:?}"),
        }
    }

    #[test]
    fn should_fail_with_input_too_long_when_query_exceeds_configured_length_limit_then_report_length_details()
     {
        let options = build_parse_options(|builder| builder.max_length(5));
        let error = expect_error_with_options("toolong=1", &options);

        match error {
            ParseError::InputTooLong { limit, actual } => {
                assert_eq!(limit, 5);
                assert_eq!(actual, "toolong=1".len());
            }
            other => panic!("expected InputTooLong error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_too_many_parameters_when_query_exceeds_parameter_limit_then_report_limit_and_actual()
     {
        let options = build_parse_options(|builder| builder.max_params(1));
        let error = expect_error_with_options("a=1&b=2", &options);

        match error {
            ParseError::TooManyParameters { limit, actual } => {
                assert_eq!(limit, 1);
                assert_eq!(actual, 2);
            }
            other => panic!("expected TooManyParameters error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_duplicate_root_key_when_scalar_key_is_repeated_then_report_offending_key() {
        let error = expect_error("color=red&color=blue");

        match error {
            ParseError::DuplicateRootKey { key } => assert_eq!(key, "color"),
            other => panic!("expected DuplicateRootKey error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_duplicate_map_entry_when_nested_key_is_repeated_then_report_parent_and_segment()
     {
        let error = expect_error("user[name]=Alice&user[name]=Bob");

        match error {
            ParseError::DuplicateMapEntry { parent, segment } => {
                assert_eq!(parent, "user[user]");
                assert_eq!(segment, "name");
            }
            other => panic!("expected DuplicateMapEntry error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_duplicate_sequence_index_when_same_index_repeats_then_report_index_and_parent()
     {
        let error = expect_error("items[0]=apple&items[0]=pear");

        match error {
            ParseError::DuplicateSequenceIndex { parent, index } => {
                assert_eq!(parent, "items[items]");
                assert_eq!(index, 0);
            }
            other => panic!("expected DuplicateSequenceIndex error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_invalid_sequence_index_when_numeric_segment_exceeds_usize_then_report_segment()
     {
        let invalid_segment = "18446744073709551616"; // u64::MAX + 1
        let query = format!("items[{invalid_segment}]=value");
        let error = expect_error(&query);

        match error {
            ParseError::InvalidSequenceIndex { parent, segment } => {
                assert_eq!(parent, "items[items]");
                assert_eq!(segment, invalid_segment);
            }
            other => panic!("expected InvalidSequenceIndex error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_nested_value_conflict_when_scalar_and_nested_patterns_mix_then_report_conflicting_parent()
     {
        let error = expect_error("foo=1&foo[bar]=2");

        match error {
            ParseError::NestedValueConflict { parent } => assert_eq!(parent, "foo"),
            other => panic!("expected NestedValueConflict error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_key_pattern_conflict_when_append_and_property_patterns_mix_then_report_conflicting_segment()
     {
        let error = expect_error("key[]=1&key[fixed]=2");

        match error {
            ParseError::KeyPatternConflict { parent, segment } => {
                assert_eq!(parent, "key[key]");
                assert_eq!(segment, "fixed");
            }
            other => panic!("expected KeyPatternConflict error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_invalid_percent_encoding_when_component_is_truncated_then_report_byte_offset()
     {
        let error = expect_error("bad=%2");

        match error {
            ParseError::InvalidPercentEncoding { index, location } => {
                assert_eq!(index, 4);
                assert_eq!(location, ParseLocation::Value);
            }
            other => panic!("expected InvalidPercentEncoding error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_invalid_character_when_query_contains_raw_space_then_report_character_location_and_offset()
     {
        let error = expect_error("bad key=1");

        match error {
            ParseError::InvalidCharacter {
                character,
                index,
                location,
            } => {
                assert_eq!(character, ' ');
                assert_eq!(index, 3);
                assert_eq!(location, ParseLocation::Query);
            }
            other => panic!("expected InvalidCharacter error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_unexpected_question_mark_when_question_mark_appears_in_key_then_report_byte_offset()
     {
        let error = expect_error("foo?bar=1");

        match error {
            ParseError::UnexpectedQuestionMark { index, location } => {
                assert_eq!(index, 3);
                assert_eq!(location, ParseLocation::Query);
            }
            other => panic!("expected UnexpectedQuestionMark error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_unmatched_bracket_when_closing_bracket_missing_then_report_bracket_and_key()
    {
        let error = expect_error("a[=1");

        match error {
            ParseError::UnmatchedBracket { key, bracket } => {
                assert_eq!(key, "a[");
                assert_eq!(bracket, '[');
            }
            other => panic!("expected UnmatchedBracket error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_depth_exceeded_when_nested_depth_surpasses_limit_then_report_depth_details()
    {
        let options = build_parse_options(|builder| builder.max_depth(1));
        let error = expect_error_with_options("a[b][c]=1", &options);

        match error {
            ParseError::DepthExceeded { key, limit, depth } => {
                assert_eq!(key, "a[b][c]");
                assert_eq!(limit, 1);
                assert_eq!(depth, 2);
            }
            other => panic!("expected DepthExceeded error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_invalid_utf8_when_percent_decoding_yields_invalid_bytes_then_report_component_location()
     {
        let error = expect_error("bad=%FF");

        match error {
            ParseError::InvalidUtf8 { location } => {
                assert_eq!(location, ParseLocation::Value);
            }
            other => panic!("expected InvalidUtf8 error, got {other:?}"),
        }
    }

    #[test]
    fn should_fail_with_serde_error_when_deserializing_into_struct_then_surface_underlying_message()
    {
        #[derive(Debug, Deserialize, Default)]
        struct NumericTarget {
            #[serde(rename = "count")]
            _count: u32,
        }

        let error = match parse_default::<NumericTarget>("count=abc") {
            Err(QsParseError::Parse(error)) => error,
            Err(QsParseError::MissingParseOptions) => {
                unreachable!("parse options must be configured before parsing")
            }
            Ok(_) => panic!("expected parse failure when deserializing into struct"),
        };

        match error {
            ParseError::Serde(source) => {
                assert_eq!(source.to_string(), "invalid number literal `abc` at count");
            }
            other => panic!("expected Serde error, got {other:?}"),
        }
    }
}
