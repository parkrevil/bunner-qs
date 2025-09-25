use bunner_qs::{QueryMap, StringifyError, StringifyOptions, Value, stringify};
#[cfg(feature = "serde")]
use serde::Deserialize;

fn map_simple(entries: &[(&str, &str)]) -> QueryMap {
    let mut result = QueryMap::new();
    for (key, value) in entries {
        result.insert((*key).to_string(), Value::String((*value).to_string()));
    }
    result
}

#[test]
fn stringifies_basic_pairs() {
    let map = map_simple(&[("a", "1"), ("b", "two")]);
    let encoded = stringify(&map, None).expect("should stringify");
    assert_eq!(encoded, "a=1&b=two");
}

#[test]
fn query_map_to_string_method_matches_function() {
    let map = map_simple(&[("x", "1"), ("y", "two")]);
    let via_fn = stringify(&map, None).expect("function stringify should succeed");
    let via_method = map.to_string().expect("method stringify should succeed");
    assert_eq!(via_fn, via_method);
}

#[test]
fn uses_plus_for_spaces_when_requested() {
    let map = map_simple(&[("note", "hello world")]);
    let options = StringifyOptions {
        space_as_plus: true,
        ..StringifyOptions::default()
    };
    let encoded = stringify(&map, Some(options.clone())).expect("should encode with plus");
    assert_eq!(encoded, "note=hello+world");

    let default_encoded = stringify(&map, None).expect("default percent encodes");
    assert_eq!(default_encoded, "note=hello%20world");
}

#[test]
fn can_add_query_prefix() {
    let map = map_simple(&[("a", "1")]);
    let options = StringifyOptions {
        add_query_prefix: true,
        ..StringifyOptions::default()
    };
    let encoded = stringify(&map, Some(options.clone())).expect("should prefix with question mark");
    assert_eq!(encoded, "?a=1");

    let empty = QueryMap::new();
    let prefixed_empty = stringify(&empty, Some(options)).expect("empty map still prefixes");
    assert_eq!(prefixed_empty, "?");
}

#[test]
fn rejects_control_characters_in_value() {
    let mut map = QueryMap::new();
    map.insert("a".to_string(), Value::String("line\nbreak".to_string()));
    let error = stringify(&map, None).expect_err("control characters should be rejected");
    assert!(matches!(error, StringifyError::InvalidValue { .. }));
}

#[test]
fn rejects_control_characters_in_key() {
    let mut map = QueryMap::new();
    map.insert("bad\x07key".to_string(), Value::String("value".to_string()));
    let error = stringify(&map, None).expect_err("control characters in key should be rejected");
    assert!(matches!(error, StringifyError::InvalidKey { .. }));
}

#[cfg(feature = "serde")]
#[test]
fn query_map_to_struct_method_converts_struct() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Info {
        name: String,
        age: u8,
    }

    let mut map = QueryMap::new();
    map.insert("name".into(), Value::String("Jane".into()));
    map.insert("age".into(), Value::String("30".into()));

    let info = map.to_struct::<Info>().expect("should decode to struct");
    assert_eq!(
        info,
        Info {
            name: "Jane".into(),
            age: 30
        }
    );
}

#[test]
fn builder_constructs_stringify_options() {
    let options = StringifyOptions::builder()
        .space_as_plus(true)
        .add_query_prefix(true)
        .build();

    assert!(options.space_as_plus);
    assert!(options.add_query_prefix);
}
