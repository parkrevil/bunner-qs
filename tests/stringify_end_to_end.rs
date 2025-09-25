mod common;

use bunner_qs::{
    ParseOptions, QueryMap, StringifyError, StringifyOptions, Value, parse, parse_with, stringify,
    stringify_with,
};
use common::{assert_str_entry, map_from_pairs};
use indexmap::IndexMap;

fn build_nested_user_map() -> QueryMap {
    let mut user = IndexMap::new();
    user.insert("name".to_string(), Value::String("Jane".to_string()));

    let mut address = IndexMap::new();
    address.insert("city".to_string(), Value::String("Seoul".to_string()));
    address.insert("postal".to_string(), Value::String("04524".to_string()));
    user.insert("address".to_string(), Value::Object(address));

    let hobbies = vec![
        Value::String("tea".to_string()),
        Value::String("hiking".to_string()),
    ];
    user.insert("hobbies".to_string(), Value::Array(hobbies));

    let mut root = QueryMap::new();
    root.insert("user".to_string(), Value::Object(user));
    root
}

#[test]
fn stringifies_basic_pairs() {
    let map = map_from_pairs(&[("a", "1"), ("b", "two")]);
    let encoded = stringify(&map).expect("should stringify basic pairs");
    assert_eq!(encoded, "a=1&b=two");
}

#[test]
fn empty_map_returns_empty_string() {
    let map = QueryMap::new();
    let encoded = stringify(&map).expect("empty map should stringify");
    assert_eq!(encoded, "");
}

#[test]
fn method_matches_function_output() {
    let map = map_from_pairs(&[("x", "1"), ("y", "two")]);
    let via_fn = stringify(&map).expect("function stringify should succeed");
    let via_method = map.to_string().expect("QueryMap::to_string should match");
    assert_eq!(via_fn, via_method);
}

#[test]
fn space_encoding_respects_option() {
    let map = map_from_pairs(&[("note", "hello world")]);
    let plus = StringifyOptions {
        space_as_plus: true,
    };
    let encoded_plus = stringify_with(&map, &plus).expect("should encode spaces as plus");
    assert_eq!(encoded_plus, "note=hello+world");

    let encoded_default = stringify(&map).expect("default should percent encode spaces");
    assert_eq!(encoded_default, "note=hello%20world");
}

#[test]
fn percent_encodes_reserved_and_unicode() {
    let mut map = QueryMap::new();
    map.insert(
        "title".to_string(),
        Value::String("rock & roll/舞".to_string()),
    );
    let encoded = stringify(&map).expect("should percent encode reserved characters");
    assert_eq!(encoded, "title=rock%20%26%20roll%2F%E8%88%9E");
}

#[test]
fn percent_encodes_long_nested_unicode_values() {
    let long_value = "🚀".repeat(64);

    let mut profile = IndexMap::new();
    profile.insert("bio".to_string(), Value::String(long_value.clone()));

    let mut root = QueryMap::new();
    root.insert("profile".to_string(), Value::Object(profile));

    let encoded = stringify(&root).expect("should stringify long unicode value");
    assert!(encoded.contains("%F0%9F%9A%80"));

    let parsed = parse(&encoded).expect("percent encoded payload should parse");
    let profile = parsed
        .get("profile")
        .and_then(Value::as_object)
        .expect("missing profile");
    assert_str_entry(profile, "bio", &long_value);
}

#[test]
fn nested_structures_use_bracket_notation() {
    let map = build_nested_user_map();
    let encoded = stringify(&map).expect("should stringify nested structures");
    assert_eq!(
        encoded,
        "user%5Bname%5D=Jane&user%5Baddress%5D%5Bcity%5D=Seoul&user%5Baddress%5D%5Bpostal%5D=04524&user%5Bhobbies%5D%5B0%5D=tea&user%5Bhobbies%5D%5B1%5D=hiking",
    );
}

#[test]
fn round_trip_through_parse_preserves_structure() {
    let map = build_nested_user_map();
    let encoded = stringify(&map).expect("should stringify nested map");
    let parsed = parse(&encoded).expect("stringified output should parse");
    assert_eq!(parsed, map);
}

#[test]
fn round_trip_with_space_plus_option() {
    let mut map = QueryMap::new();
    map.insert("msg".into(), Value::String("one two".into()));

    let options = StringifyOptions::builder()
        .space_as_plus(true)
        .build()
        .expect("builder should succeed");
    let encoded = stringify_with(&map, &options).expect("stringify with plus should work");

    let parse_options = ParseOptions {
        space_as_plus: true,
        ..ParseOptions::default()
    };
    let reparsed = parse_with(&encoded, &parse_options).expect("parse should honor plus");
    assert_str_entry(&reparsed, "msg", "one two");
}

#[test]
fn rejects_control_characters_in_key() {
    let mut map = QueryMap::new();
    map.insert(
        "bad\u{0007}key".to_string(),
        Value::String("value".to_string()),
    );
    let error = stringify(&map).expect_err("control characters in key should fail");
    match error {
        StringifyError::InvalidKey { key } => assert_eq!(key, "bad\u{0007}key"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_control_characters_in_value() {
    let mut map = QueryMap::new();
    map.insert(
        "normal".to_string(),
        Value::String("line\nbreak".to_string()),
    );
    let error = stringify(&map).expect_err("control characters in value should fail");
    match error {
        StringifyError::InvalidValue { key } => assert_eq!(key, "normal"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn rejects_control_characters_in_nested_value() {
    let mut address = IndexMap::new();
    address.insert(
        "line1".to_string(),
        Value::String("First\nLine".to_string()),
    );

    let mut profile = IndexMap::new();
    profile.insert("address".to_string(), Value::Object(address));

    let mut map = QueryMap::new();
    map.insert("profile".to_string(), Value::Object(profile));

    let error = stringify(&map).expect_err("control characters inside nested value should fail");
    match error {
        StringifyError::InvalidValue { key } => assert_eq!(key, "profile[address][line1]"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn array_of_objects_stringifies_cleanly() {
    let mut phone_one = IndexMap::new();
    phone_one.insert("kind".to_string(), Value::String("mobile".to_string()));
    phone_one.insert("number".to_string(), Value::String("+44 123".to_string()));

    let mut phone_two = IndexMap::new();
    phone_two.insert("kind".to_string(), Value::String("office".to_string()));
    phone_two.insert("number".to_string(), Value::String("+44 987".to_string()));

    let mut contact = IndexMap::new();
    contact.insert(
        "phones".to_string(),
        Value::Array(vec![Value::Object(phone_one), Value::Object(phone_two)]),
    );

    let mut map = QueryMap::new();
    map.insert("contact".to_string(), Value::Object(contact));

    let encoded = stringify(&map).expect("array of objects should stringify");
    let reparsed = parse(&encoded).expect("stringified payload should parse");
    assert_eq!(reparsed, map);
}

#[test]
fn stringify_options_builder_configures_flags() {
    let options = StringifyOptions::builder()
        .space_as_plus(true)
        .build()
        .expect("builder should construct options");
    assert!(options.space_as_plus);
}
