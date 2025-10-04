#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/json.rs"]
mod json;
#[path = "common/options.rs"]
mod options;
#[path = "common/serde_helpers.rs"]
mod serde_helpers;
#[path = "common/stringify_options.rs"]
mod stringify_options;

use asserts::assert_str_path;
use bunner_qs_rs::{StringifyError, StringifyOptions, parse, stringify, stringify_with};
use json::json_from_pairs;
use options::try_build_parse_options;
use serde::Serialize;
use serde_helpers::{
    assert_encoded_contains, assert_parse_roundtrip, assert_stringify_roundtrip,
    assert_stringify_roundtrip_with_options,
};
use serde_json::{Map, Value, json};
use stringify_options::try_build_stringify_options;

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

const STRINGIFY_BUILD_OK: &str = "stringify options builder should succeed";

#[test]
fn should_stringify_basic_pairs_when_map_contains_simple_entries() {
    let map = json_from_pairs(&[("a", "1"), ("b", "two")]);
    let encoded = stringify(&map).expect("should stringify basic pairs");
    assert_eq!(encoded, "a=1&b=two");
    assert_parse_roundtrip(&encoded);
}

#[test]
fn should_return_empty_string_when_map_has_no_entries() {
    let map = Value::Object(Map::new());
    let encoded = stringify(&map).expect("empty map should stringify");
    assert_eq!(encoded, "");
}

#[test]
fn should_match_function_output_when_using_default_options() {
    let map = json_from_pairs(&[("x", "1"), ("y", "two")]);
    let via_fn = stringify(&map).expect("function stringify should succeed");
    let via_options = stringify_with(&map, &StringifyOptions::default())
        .expect("default stringify_with should match");
    assert_eq!(via_fn, via_options);
}

#[test]
fn should_encode_spaces_as_plus_when_option_is_enabled() {
    let map = json_from_pairs(&[("note", "hello world")]);
    let plus = try_build_stringify_options(|builder| builder.space_as_plus(true))
        .expect(STRINGIFY_BUILD_OK);
    let encoded_plus = stringify_with(&map, &plus).expect("should encode spaces as plus");
    assert_eq!(encoded_plus, "note=hello+world");

    let encoded_default = stringify(&map).expect("default should percent encode spaces");
    assert_eq!(encoded_default, "note=hello%20world");
}

#[test]
fn should_percent_encode_reserved_and_unicode_characters_when_stringifying() {
    let map = json!({
        "title": "rock & roll/舞"
    });
    let encoded = stringify(&map).expect("should percent encode reserved characters");
    assert_eq!(encoded, "title=rock%20%26%20roll%2F%E8%88%9E");
}

#[test]
fn should_percent_encode_fragments_and_equals_when_reserved_characters_present() {
    let map = json!({
        "frag#ment": "a=b&c"
    });

    let encoded = stringify(&map).expect("reserved characters should be encoded");
    assert_eq!(encoded, "frag%23ment=a%3Db%26c");

    let reparsed: Value = parse(&encoded).expect("encoded string should be parseable");
    assert_str_path(&reparsed, &["frag#ment"], "a=b&c");
}

#[test]
fn should_percent_encode_plus_sign_when_using_default_behavior() {
    let map = json!({
        "symbol": "1+1"
    });

    let encoded = stringify(&map).expect("plus should be percent encoded");
    assert_eq!(encoded, "symbol=1%2B1");

    let parsed: Value = parse(&encoded).expect("encoded plus should decode");
    assert_str_path(&parsed, &["symbol"], "1+1");
}

#[test]
fn should_percent_encode_long_unicode_values_when_stringifying_nested_data() {
    let long_value = "🚀".repeat(64);

    let root = json!({
        "profile": {
            "bio": long_value
        }
    });

    let encoded = stringify(&root).expect("should stringify long unicode value");
    assert_encoded_contains(&encoded, &["%F0%9F%9A%80"]);

    let parsed: Value = parse(&encoded).expect("percent encoded payload should parse");
    assert_str_path(&parsed, &["profile", "bio"], &long_value);
}

#[test]
fn should_percent_encode_multilingual_values_when_stringifying_map() {
    let map = json!({
        "name": "Jürgen",
        "emoji": "😀",
        "cyrillic": "Привет",
        "arabic": "مرحبا",
        "combining": "Café",
        "thai": "สวัสดี"
    });

    let encoded = stringify(&map).expect("should percent encode multilingual values");
    assert_encoded_contains(
        &encoded,
        &[
            "name=J%C3%BCrgen",
            "emoji=%F0%9F%98%80",
            "cyrillic=%D0%9F%D1%80%D0%B8%D0%B2%D0%B5%D1%82",
            "arabic=%D9%85%D8%B1%D8%AD%D8%A8%D8%A7",
            "combining=Cafe%CC%81",
            "thai=%E0%B8%AA%E0%B8%A7%E0%B8%B1%E0%B8%AA%E0%B8%94%E0%B8%B5",
        ],
    );
}

#[test]
fn should_encode_extended_unicode_keys_and_values_when_serializing() {
    let map = json!({
        "鍵🔑": "值🌈",
        "emoji_key🙂": "مرحبا",
        "combinação": "linhão"
    });

    let encoded = stringify(&map).expect("should encode extended unicode keys and values");
    assert_encoded_contains(
        &encoded,
        &[
            "%E9%8D%B5%F0%9F%94%91=%E5%80%BC%F0%9F%8C%88",
            "emoji_key%F0%9F%99%82=%D9%85%D8%B1%D8%AD%D8%A8%D8%A7",
            "combina%C3%A7%C3%A3o=linh%C3%A3o",
        ],
    );

    let reparsed: Value = parse(&encoded).expect("encoded query should round-trip");
    assert_str_path(&reparsed, &["鍵🔑"], "值🌈");
    assert_str_path(&reparsed, &["emoji_key🙂"], "مرحبا");
    assert_str_path(&reparsed, &["combinação"], "linhão");
}

#[test]
fn should_use_bracket_notation_when_stringifying_nested_structures() {
    let map = build_nested_user_value();
    let encoded = stringify(&map).expect("should stringify nested structures");
    assert_encoded_contains(
        &encoded,
        &[
            "user%5Bname%5D=Jane",
            "user%5Baddress%5D%5Bcity%5D=Seoul",
            "user%5Baddress%5D%5Bpostal%5D=04524",
            "user%5Bhobbies%5D%5B0%5D=tea",
            "user%5Bhobbies%5D%5B1%5D=hiking",
        ],
    );
}

#[test]
fn should_roundtrip_structure_when_parsing_stringified_payload() {
    let map = build_nested_user_value();
    assert_stringify_roundtrip(&map);
}

#[test]
fn should_roundtrip_with_spaces_when_plus_option_enabled() {
    let map = json!({
        "msg": "one two"
    });

    let options = try_build_stringify_options(|builder| builder.space_as_plus(true))
        .expect(STRINGIFY_BUILD_OK);
    let parse_options = try_build_parse_options(|builder| builder.space_as_plus(true))
        .expect("parse options builder should succeed");
    let reparsed = assert_stringify_roundtrip_with_options(&map, &options, &parse_options);
    assert_str_path(&reparsed, &["msg"], "one two");
}

#[test]
fn should_reject_control_characters_when_key_contains_them() {
    let map = json!({
        "bad\u{0007}key": "value"
    });
    asserts::assert_err_matches!(
        stringify(&map),
        StringifyError::InvalidKey { key } => |_message| {
            assert_eq!(key, "bad\u{0007}key");
        }
    );
}

#[test]
fn should_reject_control_characters_when_value_contains_line_break() {
    let map = json!({
        "normal": "line\nbreak"
    });
    asserts::assert_err_matches!(
        stringify(&map),
        StringifyError::InvalidValue { key } => |_message| {
            assert_eq!(key, "normal");
        }
    );
}

#[test]
fn should_reject_delete_character_when_value_contains_delete_control() {
    let map = json!({
        "note": format!("alert{}signal", '\u{007F}')
    });

    asserts::assert_err_matches!(
        stringify(&map),
        StringifyError::InvalidValue { key } => |_message| {
            assert_eq!(key, "note");
        }
    );
}

#[test]
fn should_reject_control_characters_when_nested_value_contains_them() {
    let map = json!({
        "profile": {
            "address": {
                "line1": "First\nLine"
            }
        }
    });

    asserts::assert_err_matches!(
        stringify(&map),
        StringifyError::InvalidValue { key } => |_message| {
            assert_eq!(key, "profile[address][line1]");
        }
    );
}

#[test]
fn should_stringify_array_of_objects_when_structure_is_nested() {
    let map = json!({
        "contact": {
            "phones": [
                {"kind": "mobile", "number": "+44 123"},
                {"kind": "office", "number": "+44 987"}
            ]
        }
    });

    let reparsed = assert_stringify_roundtrip(&map);
    assert_eq!(reparsed, map);
}

#[test]
fn should_configure_flags_when_building_stringify_options() {
    let options = try_build_stringify_options(|builder| builder.space_as_plus(true))
        .expect(STRINGIFY_BUILD_OK);
    assert!(options.space_as_plus);
}

#[test]
fn should_skip_none_fields_when_option_values_are_missing() {
    #[derive(Serialize)]
    struct OptionalFields<'a> {
        keep: Option<&'a str>,
        drop: Option<&'a str>,
    }

    let payload = OptionalFields {
        keep: Some("alpha"),
        drop: None,
    };

    let encoded = stringify(&payload).expect("option fields set to None should be skipped");
    assert_eq!(encoded, "keep=alpha");
}

#[test]
fn should_preserve_none_placeholders_when_sequence_contains_gaps() {
    #[derive(Serialize)]
    struct SequenceWithGaps<'a> {
        tags: Vec<Option<&'a str>>,
    }

    let payload = SequenceWithGaps {
        tags: vec![Some("zero"), None, Some("two")],
    };

    let encoded = stringify(&payload).expect("sequence placeholders should be preserved");
    assert_eq!(encoded, "tags%5B0%5D=zero&tags%5B1%5D=&tags%5B2%5D=two");
}
