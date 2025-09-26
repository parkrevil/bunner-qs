#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/json.rs"]
mod json;

use asserts::{assert_str_entry, expect_object, expect_path};
use bunner_qs::{
    ParseOptions, SerdeStringifyError, StringifyError, StringifyOptions, parse, parse_with,
    stringify, stringify_with,
};
use json::json_from_pairs;
use serde_json::{Map, Value, json};

fn build_nested_user_value() -> Value {
    json!({
        "user": {
            "name": "Jane",
            "address": {
                "city": "Seoul",
                "postal": "04524"
            },
            "hobbies": ["tea", "hiking"]
        }
    })
}

#[test]
fn stringifies_basic_pairs() {
    let map = json_from_pairs(&[("a", "1"), ("b", "two")]);
    let encoded = stringify(&map).expect("should stringify basic pairs");
    assert_eq!(encoded, "a=1&b=two");
}

#[test]
fn empty_map_returns_empty_string() {
    let map = Value::Object(Map::new());
    let encoded = stringify(&map).expect("empty map should stringify");
    assert_eq!(encoded, "");
}

#[test]
fn method_matches_function_output() {
    let map = json_from_pairs(&[("x", "1"), ("y", "two")]);
    let via_fn = stringify(&map).expect("function stringify should succeed");
    let via_options = stringify_with(&map, &StringifyOptions::default())
        .expect("default stringify_with should match");
    assert_eq!(via_fn, via_options);
}

#[test]
fn space_encoding_respects_option() {
    let map = json_from_pairs(&[("note", "hello world")]);
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
    let map = json!({
        "title": "rock & roll/舞"
    });
    let encoded = stringify(&map).expect("should percent encode reserved characters");
    assert_eq!(encoded, "title=rock%20%26%20roll%2F%E8%88%9E");
}

#[test]
fn percent_encodes_fragments_and_equals() {
    let map = json!({
        "frag#ment": "a=b&c"
    });

    let encoded = stringify(&map).expect("reserved characters should be encoded");
    assert_eq!(encoded, "frag%23ment=a%3Db%26c");

    let reparsed: Value = parse(&encoded).expect("encoded string should be parseable");
    let object = expect_object(&reparsed);
    assert_str_entry(object, "frag#ment", "a=b&c");
}

#[test]
fn plus_sign_is_percent_encoded_by_default() {
    let map = json!({
        "symbol": "1+1"
    });

    let encoded = stringify(&map).expect("plus should be percent encoded");
    assert_eq!(encoded, "symbol=1%2B1");

    let parsed: Value = parse(&encoded).expect("encoded plus should decode");
    let object = expect_object(&parsed);
    assert_str_entry(object, "symbol", "1+1");
}

#[test]
fn percent_encodes_long_nested_unicode_values() {
    let long_value = "🚀".repeat(64);

    let root = json!({
        "profile": {
            "bio": long_value
        }
    });

    let encoded = stringify(&root).expect("should stringify long unicode value");
    assert!(encoded.contains("%F0%9F%9A%80"));

    let parsed: Value = parse(&encoded).expect("percent encoded payload should parse");
    let profile = expect_object(expect_path(&parsed, &["profile"]));
    assert_str_entry(profile, "bio", &long_value);
}

#[test]
fn percent_encodes_multilingual_values() {
    let map = json!({
        "name": "Jürgen",
        "emoji": "😀",
        "cyrillic": "Привет",
        "arabic": "مرحبا",
        "combining": "Café",
        "thai": "สวัสดี"
    });

    let encoded = stringify(&map).expect("should percent encode multilingual values");
    for expected in [
        "name=J%C3%BCrgen",
        "emoji=%F0%9F%98%80",
        "cyrillic=%D0%9F%D1%80%D0%B8%D0%B2%D0%B5%D1%82",
        "arabic=%D9%85%D8%B1%D8%AD%D8%A8%D8%A7",
        "combining=Cafe%CC%81",
        "thai=%E0%B8%AA%E0%B8%A7%E0%B8%B1%E0%B8%AA%E0%B8%94%E0%B8%B5",
    ] {
        assert!(
            encoded.contains(expected),
            "encoded string `{encoded}` should contain `{expected}`"
        );
    }
}

#[test]
fn nested_structures_use_bracket_notation() {
    let map = build_nested_user_value();
    let encoded = stringify(&map).expect("should stringify nested structures");
    for expected in [
        "user%5Bname%5D=Jane",
        "user%5Baddress%5D%5Bcity%5D=Seoul",
        "user%5Baddress%5D%5Bpostal%5D=04524",
        "user%5Bhobbies%5D%5B0%5D=tea",
        "user%5Bhobbies%5D%5B1%5D=hiking",
    ] {
        assert!(
            encoded.contains(expected),
            "encoded string `{encoded}` should contain `{expected}`"
        );
    }
}

#[test]
fn round_trip_through_parse_preserves_structure() {
    let map = build_nested_user_value();
    let encoded = stringify(&map).expect("should stringify nested map");
    let parsed: Value = parse(&encoded).expect("stringified output should parse");
    assert_eq!(parsed, map);
}

#[test]
fn round_trip_with_space_plus_option() {
    let map = json!({
        "msg": "one two"
    });

    let options = StringifyOptions::builder()
        .space_as_plus(true)
        .build()
        .expect("builder should succeed");
    let encoded = stringify_with(&map, &options).expect("stringify with plus should work");

    let parse_options = ParseOptions {
        space_as_plus: true,
        ..ParseOptions::default()
    };
    let reparsed: Value = parse_with(&encoded, &parse_options).expect("parse should honor plus");
    let object = expect_object(&reparsed);
    assert_str_entry(object, "msg", "one two");
}

#[test]
fn rejects_control_characters_in_key() {
    let map = json!({
        "bad\u{0007}key": "value"
    });
    asserts::assert_err_matches!(
        stringify(&map),
        SerdeStringifyError::Stringify(StringifyError::InvalidKey { key }) => |_message| {
            assert_eq!(key, "bad\u{0007}key");
        }
    );
}

#[test]
fn rejects_control_characters_in_value() {
    let map = json!({
        "normal": "line\nbreak"
    });
    asserts::assert_err_matches!(
        stringify(&map),
        SerdeStringifyError::Stringify(StringifyError::InvalidValue { key }) => |_message| {
            assert_eq!(key, "normal");
        }
    );
}

#[test]
fn rejects_control_characters_in_nested_value() {
    let map = json!({
        "profile": {
            "address": {
                "line1": "First\nLine"
            }
        }
    });

    asserts::assert_err_matches!(
        stringify(&map),
        SerdeStringifyError::Stringify(StringifyError::InvalidValue { key }) => |_message| {
            assert_eq!(key, "profile[address][line1]");
        }
    );
}

#[test]
fn array_of_objects_stringifies_cleanly() {
    let map = json!({
        "contact": {
            "phones": [
                {"kind": "mobile", "number": "+44 123"},
                {"kind": "office", "number": "+44 987"}
            ]
        }
    });

    let encoded = stringify(&map).expect("array of objects should stringify");
    let reparsed: Value = parse(&encoded).expect("stringified payload should parse");
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
