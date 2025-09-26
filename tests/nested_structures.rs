#[path = "common/asserts.rs"]
mod asserts;

use asserts::{assert_str_path, assert_string_array_path};
use bunner_qs::{ParseError, ParseOptions, parse, parse_with, stringify};
use serde_json::{Value, json};
#[test]
fn parses_deeply_nested_structure_and_round_trips() {
    let query = "profile[name]=Ada&profile[contacts][email]=ada@example.com&profile[contacts][phones][0]=+44%20123&profile[contacts][phones][1]=+44%20987&profile[meta][created]=2024";
    let parsed: Value = parse(query).expect("nested structure should parse");

    assert_str_path(&parsed, &["profile", "name"], "Ada");
    assert_str_path(
        &parsed,
        &["profile", "contacts", "email"],
        "ada@example.com",
    );
    assert_string_array_path(
        &parsed,
        &["profile", "contacts", "phones"],
        &["+44 123", "+44 987"],
    );
    assert_str_path(&parsed, &["profile", "meta", "created"], "2024");

    let stringified = stringify(&parsed).expect("stringify should succeed");
    let reparsed: Value = parse(&stringified).expect("reparsed string should match");
    assert_eq!(parsed, reparsed);
}

#[test]
fn allows_uniform_append_pattern() {
    let parsed: Value = parse("tags[]=rust&tags[]=serde").expect("append pattern should parse");
    assert_string_array_path(&parsed, &["tags"], &["rust", "serde"]);
}

#[test]
fn allows_uniform_numeric_pattern() {
    let parsed: Value =
        parse("items[0]=apple&items[1]=banana").expect("numeric pattern should parse");
    assert_string_array_path(&parsed, &["items"], &["apple", "banana"]);
}

#[test]
fn stringify_preserves_array_order_for_numeric_indices() {
    let map = json!({
        "items": ["alpha", "beta", "gamma"]
    });

    let encoded = stringify(&map).expect("stringify should succeed");
    let reparsed: Value = parse(&encoded).expect("encoded string should parse");
    assert_string_array_path(&reparsed, &["items"], &["alpha", "beta", "gamma"]);
}

#[test]
fn rejects_array_scalar_then_object_conflict() {
    asserts::assert_err_matches!(
        parse::<Value>("items[0]=apple&items[0][kind]=fruit"),
        ParseError::DuplicateKey { key } => |_message| {
            assert_eq!(key, "items");
        }
    );
}

#[test]
fn rejects_array_object_then_scalar_conflict() {
    asserts::assert_err_matches!(
        parse::<Value>("items[0][kind]=fruit&items[0]=apple"),
        ParseError::DuplicateKey { key } => |_message| {
            assert!(key == "items" || key == "0");
        }
    );
}

#[test]
fn rejects_mixed_append_and_numeric_patterns() {
    asserts::assert_err_matches!(
        parse::<Value>("key[]=1&key[0]=1"),
        ParseError::DuplicateKey { key } => |_message| {
            assert_eq!(key, "key");
        }
    );
}

#[test]
fn rejects_scalar_and_nested_mix() {
    asserts::assert_err_matches!(
        parse::<Value>("foo=1&foo[bar]=2"),
        ParseError::DuplicateKey { key } => |_message| {
            assert_eq!(key, "foo");
        }
    );
}

#[test]
fn rejects_duplicate_scalar_values() {
    asserts::assert_err_matches!(
        parse::<Value>("foo=1&foo=2"),
        ParseError::DuplicateKey { key } => |_message| {
            assert_eq!(key, "foo");
        }
    );
}

#[test]
fn rejects_non_contiguous_numeric_indexes() {
    asserts::assert_err_matches!(
        parse::<Value>("items[0]=apple&items[2]=cherry"),
        ParseError::DuplicateKey { key } => |_message| {
            assert_eq!(key, "items");
        }
    );
}

#[test]
fn respects_depth_limit_for_mixed_nested_structure() {
    let query = "profile[contacts][phones][0][number]=+44%20123";

    let permissive = ParseOptions {
        max_depth: Some(4),
        ..ParseOptions::default()
    };
    parse_with::<Value>(query, &permissive).expect("depth of four should succeed");

    let strict = ParseOptions {
        max_depth: Some(2),
        ..ParseOptions::default()
    };
    asserts::assert_err_matches!(
        parse_with::<Value>(query, &strict),
        ParseError::DepthExceeded { key, limit } => |_message| {
            assert_eq!(key, "profile[contacts][phones][0][number]");
            assert_eq!(limit, 2);
        }
    );
}
