mod common;

use bunner_qs::{ParseError, QueryMap, Value, parse, stringify};
use common::{assert_str_entry, assert_string_array, expect_object};

#[test]
fn parses_deeply_nested_structure_and_round_trips() {
    let query = "profile[name]=Ada&profile[contacts][email]=ada@example.com&profile[contacts][phones][0]=+44%20123&profile[contacts][phones][1]=+44%20987&profile[meta][created]=2024";
    let parsed = parse(query).expect("nested structure should parse");

    let profile = expect_object(parsed.get("profile").expect("missing profile"));
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

    let stringified = parsed.to_string().expect("stringify should succeed");
    let reparsed = parse(&stringified).expect("reparsed string should match");
    assert_eq!(parsed, reparsed);
}

#[test]
fn allows_uniform_append_pattern() {
    let parsed = parse("tags[]=rust&tags[]=serde").expect("append pattern should parse");
    let tags = parsed.get("tags").expect("missing tags");
    assert_string_array(tags, &["rust", "serde"]);
}

#[test]
fn allows_uniform_numeric_pattern() {
    let parsed = parse("items[0]=apple&items[1]=banana").expect("numeric pattern should parse");
    let items = parsed.get("items").expect("missing items");
    assert_string_array(items, &["apple", "banana"]);
}

#[test]
fn stringify_preserves_array_order_for_numeric_indices() {
    let mut map = QueryMap::new();
    map.insert(
        "items".into(),
        Value::Array(vec![
            Value::String("alpha".into()),
            Value::String("beta".into()),
            Value::String("gamma".into()),
        ]),
    );

    let encoded = stringify(&map).expect("stringify should succeed");
    let reparsed = parse(&encoded).expect("encoded string should parse");

    let items = reparsed.get("items").expect("missing items");
    assert_string_array(items, &["alpha", "beta", "gamma"]);
}

#[test]
fn rejects_mixed_append_and_numeric_patterns() {
    let error = parse("key[]=1&key[0]=1").expect_err("mixed append/numeric should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "key"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_scalar_and_nested_mix() {
    let error = parse("foo=1&foo[bar]=2").expect_err("mixing scalar and nested should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "foo"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_duplicate_scalar_values() {
    let error = parse("foo=1&foo=2").expect_err("duplicate scalar keys should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "foo"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_non_contiguous_numeric_indexes() {
    let error = parse("items[0]=apple&items[2]=cherry")
        .expect_err("non contiguous numeric indexes should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "items"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}
