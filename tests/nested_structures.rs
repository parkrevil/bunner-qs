mod common;

use bunner_qs::{ParseError, ParseOptions, parse, parse_with, stringify};
use common::{assert_str_entry, assert_string_array, expect_object};
use serde_json::{Value, json};

#[test]
fn parses_deeply_nested_structure_and_round_trips() {
    let query = "profile[name]=Ada&profile[contacts][email]=ada@example.com&profile[contacts][phones][0]=+44%20123&profile[contacts][phones][1]=+44%20987&profile[meta][created]=2024";
    let parsed: Value = parse(query).expect("nested structure should parse");
    let root = expect_object(&parsed);

    let profile = expect_object(root.get("profile").expect("missing profile"));
    assert_str_entry(profile, "name", "Ada");

    let contacts = expect_object(profile.get("contacts").expect("missing contacts"));
    assert_str_entry(contacts, "email", "ada@example.com");

    let phones = contacts
        .get("phones")
        .expect("missing phones")
        .as_array()
        .expect("phones should be array");
    assert_eq!(phones.len(), 2);
    assert_eq!(phones[0].as_str(), Some("+44 123"));
    assert_eq!(phones[1].as_str(), Some("+44 987"));

    let meta = expect_object(profile.get("meta").expect("missing meta"));
    assert_str_entry(meta, "created", "2024");

    let stringified = stringify(&parsed).expect("stringify should succeed");
    let reparsed: Value = parse(&stringified).expect("reparsed string should match");
    assert_eq!(parsed, reparsed);
}

#[test]
fn allows_uniform_append_pattern() {
    let parsed: Value = parse("tags[]=rust&tags[]=serde").expect("append pattern should parse");
    let root = expect_object(&parsed);
    let tags = root.get("tags").expect("missing tags");
    assert_string_array(tags, &["rust", "serde"]);
}

#[test]
fn allows_uniform_numeric_pattern() {
    let parsed: Value =
        parse("items[0]=apple&items[1]=banana").expect("numeric pattern should parse");
    let root = expect_object(&parsed);
    let items = root.get("items").expect("missing items");
    assert_string_array(items, &["apple", "banana"]);
}

#[test]
fn stringify_preserves_array_order_for_numeric_indices() {
    let map = json!({
        "items": ["alpha", "beta", "gamma"]
    });

    let encoded = stringify(&map).expect("stringify should succeed");
    let reparsed: Value = parse(&encoded).expect("encoded string should parse");
    let root = expect_object(&reparsed);
    let items = root.get("items").expect("missing items");
    assert_string_array(items, &["alpha", "beta", "gamma"]);
}

#[test]
fn rejects_array_scalar_then_object_conflict() {
    let error = parse::<Value>("items[0]=apple&items[0][kind]=fruit")
        .expect_err("array entries should not change from scalar to object");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "items"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_array_object_then_scalar_conflict() {
    let error = parse::<Value>("items[0][kind]=fruit&items[0]=apple")
        .expect_err("array entries should not change from object to scalar");
    match error {
        ParseError::DuplicateKey { key } => assert!(key == "items" || key == "0"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_mixed_append_and_numeric_patterns() {
    let error = parse::<Value>("key[]=1&key[0]=1").expect_err("mixed append/numeric should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "key"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_scalar_and_nested_mix() {
    let error =
        parse::<Value>("foo=1&foo[bar]=2").expect_err("mixing scalar and nested should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "foo"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_duplicate_scalar_values() {
    let error = parse::<Value>("foo=1&foo=2").expect_err("duplicate scalar keys should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "foo"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_non_contiguous_numeric_indexes() {
    let error = parse::<Value>("items[0]=apple&items[2]=cherry")
        .expect_err("non contiguous numeric indexes should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "items"),
        other => panic!("unexpected error variant: {other:?}"),
    }
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
    let error = parse_with::<Value>(query, &strict)
        .expect_err("depth limit should reject deeply nested structure");
    match error {
        ParseError::DepthExceeded { key, limit } => {
            assert_eq!(key, "profile[contacts][phones][0][number]");
            assert_eq!(limit, 2);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}
