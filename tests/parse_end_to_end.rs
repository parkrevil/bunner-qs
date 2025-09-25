mod common;

use bunner_qs::{ParseError, ParseOptions, parse, parse_with};
use common::{assert_str_entry, assert_string_array, expect_object, map_from_pairs};

#[test]
fn parses_basic_key_value_pairs() {
    let parsed = parse("a=1&b=two").expect("basic pairs should parse");
    let expected = map_from_pairs(&[("a", "1"), ("b", "two")]);
    assert_eq!(parsed, expected);
}

#[test]
fn decodes_percent_encoded_ascii_and_unicode() {
    let parsed =
        parse("name=J%C3%BCrgen&emoji=%F0%9F%98%80").expect("percent encoding should decode");
    assert_str_entry(&parsed, "name", "Jürgen");
    assert_str_entry(&parsed, "emoji", "😀");
}

#[test]
fn allows_empty_input() {
    let parsed = parse("").expect("empty input should produce empty map");
    assert!(parsed.is_empty());
}

#[test]
fn space_as_plus_option_controls_plus_handling() {
    let relaxed = ParseOptions {
        space_as_plus: true,
        ..ParseOptions::default()
    };
    let relaxed = parse_with("note=one+two", &relaxed).expect("plus should become space");
    assert_str_entry(&relaxed, "note", "one two");

    let strict = parse("note=one+two").expect("default should keep plus literal");
    assert_str_entry(&strict, "note", "one+two");
}

#[test]
fn rejects_invalid_percent_encoding_sequences() {
    let error = parse("bad=%2").expect_err("truncated percent escape should fail");
    match error {
        ParseError::InvalidPercentEncoding { index } => assert_eq!(index, 4),
        other => panic!("unexpected error variant: {other:?}"),
    }

    let error = parse("bad=%ZZ").expect_err("non-hex percent escape should fail");
    match error {
        ParseError::InvalidPercentEncoding { index } => assert_eq!(index, 4),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_control_characters_and_unexpected_question_mark() {
    let input_with_control = format!("bad{}key=1", '\u{0007}');
    let error = parse(&input_with_control).expect_err("control characters should be rejected");
    match error {
        ParseError::InvalidCharacter { character, .. } => assert_eq!(character, '\u{0007}'),
        other => panic!("unexpected error variant: {other:?}"),
    }

    let error = parse("foo?bar=1").expect_err("embedded question mark should fail");
    match error {
        ParseError::UnexpectedQuestionMark { index } => assert_eq!(index, 3),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn detects_unmatched_brackets_and_depth_overflow() {
    let error = parse("a[=1").expect_err("unmatched bracket should fail");
    match error {
        ParseError::UnmatchedBracket { key } => assert_eq!(key, "a["),
        other => panic!("unexpected error variant: {other:?}"),
    }

    let options = ParseOptions {
        max_depth: Some(1),
        ..ParseOptions::default()
    };
    let error = parse_with("a[b][c]=1", &options).expect_err("depth limit should be enforced");
    match error {
        ParseError::DepthExceeded { key, limit } => {
            assert_eq!(key, "a[b][c]");
            assert_eq!(limit, 1);
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
    let error = parse_with("a=1&b=2", &param_limited)
        .expect_err("parameter limit should trigger on second entry");
    match error {
        ParseError::TooManyParameters { limit, actual } => {
            assert_eq!(limit, 1);
            assert_eq!(actual, 2);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    let length_limited = ParseOptions {
        max_length: Some(5),
        ..ParseOptions::default()
    };
    let error = parse_with("toolong=1", &length_limited)
        .expect_err("input exceeding max length should fail");
    match error {
        ParseError::InputTooLong { limit } => assert_eq!(limit, 5),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_duplicate_keys() {
    let error = parse("color=red&color=blue").expect_err("duplicate keys should be rejected");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "color"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_invalid_utf8_in_percent_sequences() {
    let error = parse("bad=%FF").expect_err("invalid UTF-8 should be surfaced");
    match error {
        ParseError::InvalidUtf8 => {}
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parses_nested_objects_and_arrays() {
    let parsed = parse(
        "user[name]=Alice&user[stats][age]=30&user[hobbies][]=reading&user[hobbies][]=coding",
    )
    .expect("nested structures should parse");

    let user = expect_object(parsed.get("user").expect("missing user object"));

    assert_str_entry(user, "name", "Alice");

    let stats = expect_object(user.get("stats").expect("missing stats"));
    assert_str_entry(stats, "age", "30");

    let hobbies = user.get("hobbies").expect("missing hobbies");
    assert_string_array(hobbies, &["reading", "coding"]);
}

#[test]
fn round_trips_complex_structure_with_stringify() {
    let input = "data[users][0][name]=Alice&data[users][1][name]=Bob&data[meta][version]=1";
    let parsed = parse(input).expect("parse should succeed");
    let stringified = parsed.to_string().expect("stringify should succeed");
    let reparsed = parse(&stringified).expect("reparse should succeed");
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
