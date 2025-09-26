#[path = "common/arrays.rs"]
mod arrays;
#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/json.rs"]
mod json;

use arrays::assert_string_array;
use asserts::{assert_str_entry, expect_object, expect_path};
use bunner_qs::{ParseError, ParseOptions, SerdeQueryError, parse, parse_with, stringify};
use json::json_from_pairs;
use serde::Deserialize;
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
    let object = expect_object(&parsed);
    assert_str_entry(object, "name", "Jürgen");
    assert_str_entry(object, "emoji", "😀");
    assert_str_entry(object, "cyrillic", "Привет");
    assert_str_entry(object, "arabic", "مرحبا");
    assert_str_entry(object, "combining", "Café");
    assert_str_entry(object, "thai", "สวัสดี");
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
    let object = expect_object(&parsed);
    assert_str_entry(object, "flag", "");
}

#[test]
fn space_as_plus_option_controls_plus_handling() {
    let relaxed = ParseOptions {
        space_as_plus: true,
        ..ParseOptions::default()
    };
    let relaxed: Value = parse_with("note=one+two", &relaxed).expect("plus should become space");
    let relaxed_obj = expect_object(&relaxed);
    assert_str_entry(relaxed_obj, "note", "one two");

    let strict: Value = parse("note=one+two").expect("default should keep plus literal");
    let strict_obj = expect_object(&strict);
    assert_str_entry(strict_obj, "note", "one+two");
}

#[test]
fn rejects_invalid_percent_encoding_sequences() {
    let error = parse::<Value>("bad=%2").expect_err("truncated percent escape should fail");
    let message = error.to_string();
    match error {
        ParseError::InvalidPercentEncoding { index } => {
            assert_eq!(index, 4);
            assert_eq!(message, "invalid percent-encoding at byte offset 4");
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    let error = parse::<Value>("bad=%ZZ").expect_err("non-hex percent escape should fail");
    match error {
        ParseError::InvalidPercentEncoding { index } => assert_eq!(index, 4),
        other => panic!("unexpected error variant: {other:?}"),
    }
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
    let error =
        parse::<Value>(&input_with_control).expect_err("control characters should be rejected");
    let message = error.to_string();
    match error {
        ParseError::InvalidCharacter { character, index } => {
            assert_eq!(character, '\u{0007}');
            let expected =
                format!("query contains invalid character `{character}` at byte offset {index}");
            assert_eq!(message, expected);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    let error = parse::<Value>("foo?bar=1").expect_err("embedded question mark should fail");
    let message = error.to_string();
    match error {
        ParseError::UnexpectedQuestionMark { index } => {
            assert_eq!(index, 3);
            let expected = format!("unexpected '?' character inside query at byte offset {index}");
            assert_eq!(message, expected);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn detects_unmatched_brackets_and_depth_overflow() {
    let error = parse::<Value>("a[=1").expect_err("unmatched bracket should fail");
    let message = error.to_string();
    match error {
        ParseError::UnmatchedBracket { key } => {
            assert_eq!(key, "a[");
            let expected = format!("unmatched bracket sequence in key '{key}'");
            assert_eq!(message, expected);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    let options = ParseOptions {
        max_depth: Some(1),
        ..ParseOptions::default()
    };
    let error =
        parse_with::<Value>("a[b][c]=1", &options).expect_err("depth limit should be enforced");
    let message = error.to_string();
    match error {
        ParseError::DepthExceeded { key, limit } => {
            assert_eq!(key, "a[b][c]");
            assert_eq!(limit, 1);
            let expected = format!("maximum bracket depth exceeded in key '{key}' (limit {limit})");
            assert_eq!(message, expected);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn enforces_parameter_and_length_limits() {
    let param_limited = ParseOptions {
        max_params: Some(1),
        ..ParseOptions::default()
    };
    let error = parse_with::<Value>("a=1&b=2", &param_limited)
        .expect_err("parameter limit should trigger on second entry");
    let message = error.to_string();
    match error {
        ParseError::TooManyParameters { limit, actual } => {
            assert_eq!(limit, 1);
            assert_eq!(actual, 2);
            assert_eq!(
                message,
                format!("too many parameters: received {actual}, limit {limit}")
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    let length_limited = ParseOptions {
        max_length: Some(5),
        ..ParseOptions::default()
    };
    let error = parse_with::<Value>("toolong=1", &length_limited)
        .expect_err("input exceeding max length should fail");
    let message = error.to_string();
    match error {
        ParseError::InputTooLong { limit } => {
            assert_eq!(limit, 5);
            assert_eq!(message, "input exceeds maximum length of 5 characters");
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_duplicate_keys() {
    let error =
        parse::<Value>("color=red&color=blue").expect_err("duplicate keys should be rejected");
    let message = error.to_string();
    match error {
        ParseError::DuplicateKey { key } => {
            assert_eq!(key, "color");
            let expected = format!("duplicate key '{key}' not allowed");
            assert_eq!(message, expected);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_sparse_array_indices() {
    let error = parse::<Value>("items[0]=apple&items[2]=cherry")
        .expect_err("non-contiguous array indices should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "items"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_invalid_utf8_in_percent_sequences() {
    let error = parse::<Value>("bad=%FF").expect_err("invalid UTF-8 should be surfaced");
    let message = error.to_string();
    match error {
        ParseError::InvalidUtf8 => {
            assert_eq!(message, "decoded component is not valid UTF-8");
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn serde_error_messages_are_human_readable() {
    #[derive(Debug, Deserialize, Default)]
    struct NumericTarget {
        #[serde(rename = "count")]
        _count: u32,
    }

    let error = parse::<NumericTarget>("count=abc")
        .expect_err("non-numeric value should fail to deserialize");
    let message = error.to_string();
    match error {
        ParseError::Serde(source) => {
            let expected = "failed to deserialize parsed query into target type: failed to deserialize query map: invalid number literal `abc`";
            assert_eq!(message, expected);
            match source {
                SerdeQueryError::Deserialize(inner) => {
                    assert_eq!(inner.to_string(), "invalid number literal `abc`");
                }
                other => panic!("unexpected inner serde error: {other:?}"),
            }
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parses_nested_objects_and_arrays() {
    let parsed: Value = parse(
        "user[name]=Alice&user[stats][age]=30&user[hobbies][]=reading&user[hobbies][]=coding",
    )
    .expect("nested structures should parse");

    let user = expect_object(expect_path(&parsed, &["user"]));
    assert_str_entry(user, "name", "Alice");

    let stats = expect_object(expect_path(&parsed, &["user", "stats"]));
    assert_str_entry(stats, "age", "30");

    let hobbies = expect_path(&parsed, &["user", "hobbies"]);
    assert_string_array(hobbies, &["reading", "coding"]);
}

#[test]
fn round_trips_complex_structure_with_stringify() {
    let input = "data[users][0][name]=Alice&data[users][1][name]=Bob&data[meta][version]=1";
    let parsed: Value = parse(input).expect("parse should succeed");
    let stringified = stringify(&parsed).expect("stringify should succeed");
    let reparsed: Value = parse(&stringified).expect("reparse should succeed");
    assert_eq!(parsed, reparsed);
}

#[test]
fn parse_options_builder_produces_expected_configuration() {
    let options = ParseOptions::builder()
        .space_as_plus(true)
        .max_params(3)
        .max_length(128)
        .max_depth(2)
        .build()
        .expect("builder should generate options");

    assert!(options.space_as_plus);
    assert_eq!(options.max_params, Some(3));
    assert_eq!(options.max_length, Some(128));
    assert_eq!(options.max_depth, Some(2));
}
