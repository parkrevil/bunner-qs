#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/json.rs"]
mod json;
#[path = "common/options.rs"]
mod options;
#[path = "common/serde_helpers.rs"]
mod serde_helpers;

use asserts::{assert_str_path, assert_string_array_path};
use bunner_qs::{ParseError, SerdeQueryError, parse, parse_with};
use json::json_from_pairs;
use options::build_parse_options;
use serde::Deserialize;
use serde_helpers::assert_parse_roundtrip;
use serde_json::{Value, json};

#[test]
fn parses_basic_key_value_pairs() {
    let parsed: Value = parse("a=1&b=two").expect("basic pairs should parse");
    let expected = json!({ "a": "1", "b": "two" });
    assert_eq!(parsed, expected);
}

#[test]
fn decodes_percent_encoded_ascii_and_unicode() {
    let parsed: Value = parse(concat!(
        "name=J%C3%BCrgen",
        "&emoji=%F0%9F%98%80",
        "&cyrillic=%D0%9F%D1%80%D0%B8%D0%B2%D0%B5%D1%82",
        "&arabic=%D9%85%D8%B1%D8%AD%D8%A8%D8%A7",
        "&combining=Cafe%CC%81",
        "&thai=%E0%B8%AA%E0%B8%A7%E0%B8%B1%E0%B8%AA%E0%B8%94%E0%B8%B5",
    ))
    .expect("percent encoding should decode");
    assert_str_path(&parsed, &["name"], "Jürgen");
    assert_str_path(&parsed, &["emoji"], "😀");
    assert_str_path(&parsed, &["cyrillic"], "Привет");
    assert_str_path(&parsed, &["arabic"], "مرحبا");
    assert_str_path(&parsed, &["combining"], "Café");
    assert_str_path(&parsed, &["thai"], "สวัสดี");
}

#[test]
fn allows_empty_input() {
    let parsed: Value = parse("").expect("empty input should produce empty result");
    assert_eq!(parsed, Value::Null, "empty input should yield null");
}

#[test]
fn parses_lone_question_mark_as_empty() {
    let parsed: Value = parse("?").expect("lone question mark should be treated as empty");
    assert_eq!(parsed, Value::Null, "leading '?' should not create entries");
}

#[test]
fn strips_leading_question_mark_before_pairs() {
    let parsed: Value = parse("?foo=bar&baz=qux").expect("leading question mark should be ignored");
    let expected = json_from_pairs(&[("foo", "bar"), ("baz", "qux")]);
    assert_eq!(parsed, expected);
}

#[test]
fn treats_flag_without_value_as_empty_string() {
    let parsed: Value = parse("flag").expect("keys without '=' should map to empty strings");
    assert_str_path(&parsed, &["flag"], "");
}

#[test]
fn space_as_plus_option_controls_plus_handling() {
    let relaxed = build_parse_options(|builder| builder.space_as_plus(true));
    let relaxed: Value = parse_with("note=one+two", &relaxed).expect("plus should become space");
    assert_str_path(&relaxed, &["note"], "one two");

    let strict: Value = parse("note=one+two").expect("default should keep plus literal");
    assert_str_path(&strict, &["note"], "one+two");
}

#[test]
fn rejects_invalid_percent_encoding_sequences() {
    asserts::assert_err_matches!(
        parse::<Value>("bad=%2"),
        ParseError::InvalidPercentEncoding { index } => |error_message| {
            assert_eq!(index, 4);
            assert_eq!(error_message, "invalid percent-encoding at byte offset 4");
        }
    );

    asserts::assert_err_matches!(
        parse::<Value>("bad=%ZZ"),
        ParseError::InvalidPercentEncoding { index } => |_error_message| {
            assert_eq!(index, 4);
        }
    );
}

#[test]
fn ignores_trailing_ampersands_without_pairs() {
    let parsed: Value = parse("alpha=beta&&").expect("trailing '&' should be ignored");
    let expected = json_from_pairs(&[("alpha", "beta")]);
    assert_eq!(parsed, expected);
}

#[test]
fn rejects_control_characters_and_unexpected_question_mark() {
    let input_with_control = format!("bad{}key=1", '\u{0007}');
    asserts::assert_err_matches!(
        parse::<Value>(&input_with_control),
        ParseError::InvalidCharacter { character, index } => |error_message| {
            assert_eq!(character, '\u{0007}');
            let expected =
                format!("query contains invalid character `{character}` at byte offset {index}");
            assert_eq!(error_message, expected);
        }
    );

    asserts::assert_err_matches!(
        parse::<Value>("foo?bar=1"),
        ParseError::UnexpectedQuestionMark { index } => |error_message| {
            assert_eq!(index, 3);
            let expected = format!(
                "unexpected '?' character inside query at byte offset {index}"
            );
            assert_eq!(error_message, expected);
        }
    );
}

#[test]
fn rejects_raw_space_in_keys() {
    asserts::assert_err_matches!(
        parse::<Value>("bad key=1"),
        ParseError::InvalidCharacter { character, index } => |error_message| {
            assert_eq!(character, ' ');
            assert_eq!(index, 3);
            let expected =
                format!("query contains invalid character `{character}` at byte offset {index}");
            assert_eq!(error_message, expected);
        }
    );
}

#[test]
fn detects_unmatched_brackets_and_depth_overflow() {
    asserts::assert_err_matches!(
        parse::<Value>("a[=1"),
        ParseError::UnmatchedBracket { key } => |error_message| {
            assert_eq!(key, "a[");
            let expected = format!("unmatched bracket sequence in key '{key}'");
            assert_eq!(error_message, expected);
        }
    );

    let options = build_parse_options(|builder| builder.max_depth(1));
    asserts::assert_err_matches!(
        parse_with::<Value>("a[b][c]=1", &options),
        ParseError::DepthExceeded { key, limit } => |error_message| {
            assert_eq!(key, "a[b][c]");
            assert_eq!(limit, 1);
            let expected = format!("maximum bracket depth exceeded in key '{key}' (limit {limit})");
            assert_eq!(error_message, expected);
        }
    );
}

#[test]
fn enforces_parameter_and_length_limits() {
    let param_limited = build_parse_options(|builder| builder.max_params(1));
    asserts::assert_err_matches!(
        parse_with::<Value>("a=1&b=2", &param_limited),
        ParseError::TooManyParameters { limit, actual } => |error_message| {
            assert_eq!(limit, 1);
            assert_eq!(actual, 2);
            assert_eq!(
                error_message,
                format!("too many parameters: received {actual}, limit {limit}")
            );
        }
    );

    let length_limited = build_parse_options(|builder| builder.max_length(5));
    asserts::assert_err_matches!(
        parse_with::<Value>("toolong=1", &length_limited),
        ParseError::InputTooLong { limit } => |error_message| {
            assert_eq!(limit, 5);
            assert_eq!(error_message, "input exceeds maximum length of 5 characters");
        }
    );
}

#[test]
fn rejects_duplicate_keys() {
    asserts::assert_err_matches!(
        parse::<Value>("color=red&color=blue"),
        ParseError::DuplicateKey { key } => |error_message| {
            assert_eq!(key, "color");
            let expected = format!("duplicate key '{key}' not allowed");
            assert_eq!(error_message, expected);
        }
    );
}

#[test]
fn rejects_sparse_array_indices() {
    asserts::assert_err_matches!(
        parse::<Value>("items[0]=apple&items[2]=cherry"),
        ParseError::DuplicateKey { key } => |_error_message| {
            assert_eq!(key, "items");
        }
    );
}

#[test]
fn rejects_invalid_utf8_in_percent_sequences() {
    asserts::assert_err_matches!(
        parse::<Value>("bad=%FF"),
        ParseError::InvalidUtf8 => |error_message| {
            assert_eq!(error_message, "decoded component is not valid UTF-8");
        }
    );
}

#[test]
fn serde_error_messages_are_human_readable() {
    #[derive(Debug, Deserialize, Default)]
    struct NumericTarget {
        #[serde(rename = "count")]
        _count: u32,
    }

    asserts::assert_err_matches!(
        parse::<NumericTarget>("count=abc"),
        ParseError::Serde(source) => |error_message| {
            let expected = "failed to deserialize parsed query into target type: failed to deserialize query map: invalid number literal `abc`";
            assert_eq!(error_message, expected);
            match source {
                SerdeQueryError::Deserialize(inner) => {
                    assert_eq!(inner.to_string(), "invalid number literal `abc`");
                }
                other => panic!("unexpected inner serde error: {other:?}"),
            }
        }
    );
}

#[test]
fn parses_nested_objects_and_arrays() {
    let parsed: Value = parse(
        "user[name]=Alice&user[stats][age]=30&user[hobbies][]=reading&user[hobbies][]=coding",
    )
    .expect("nested structures should parse");

    assert_str_path(&parsed, &["user", "name"], "Alice");
    assert_str_path(&parsed, &["user", "stats", "age"], "30");
    assert_string_array_path(&parsed, &["user", "hobbies"], &["reading", "coding"]);
}

#[test]
fn round_trips_complex_structure_with_stringify() {
    let input = "data[users][0][name]=Alice&data[users][1][name]=Bob&data[meta][version]=1";
    assert_parse_roundtrip(input);
}

#[test]
fn parse_options_builder_produces_expected_configuration() {
    let options = build_parse_options(|builder| {
        builder
            .space_as_plus(true)
            .max_params(3)
            .max_length(128)
            .max_depth(2)
    });

    assert!(options.space_as_plus);
    assert_eq!(options.max_params, Some(3));
    assert_eq!(options.max_length, Some(128));
    assert_eq!(options.max_depth, Some(2));
}
