use bunner_qs::{
    QueryMap, StringifyError, StringifyOptions, Value, stringify, stringify_with_options,
};

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
    let encoded = stringify(&map).expect("should stringify");
    assert_eq!(encoded, "a=1&b=two");
}

#[test]
fn uses_plus_for_spaces_when_requested() {
    let map = map_simple(&[("note", "hello world")]);
    let options = StringifyOptions {
        space_as_plus: true,
        ..StringifyOptions::default()
    };
    let encoded = stringify_with_options(&map, &options).expect("should encode with plus");
    assert_eq!(encoded, "note=hello+world");

    let default_encoded = stringify(&map).expect("default percent encodes");
    assert_eq!(default_encoded, "note=hello%20world");
}

#[test]
fn can_add_query_prefix() {
    let map = map_simple(&[("a", "1")]);
    let options = StringifyOptions {
        add_query_prefix: true,
        ..StringifyOptions::default()
    };
    let encoded = stringify_with_options(&map, &options).expect("should prefix with question mark");
    assert_eq!(encoded, "?a=1");

    let empty = QueryMap::new();
    let prefixed_empty =
        stringify_with_options(&empty, &options).expect("empty map still prefixes");
    assert_eq!(prefixed_empty, "?");
}

#[test]
fn rejects_control_characters_in_value() {
    let mut map = QueryMap::new();
    map.insert("a".to_string(), Value::String("line\nbreak".to_string()));
    let error = stringify(&map).expect_err("control characters should be rejected");
    assert!(matches!(error, StringifyError::InvalidValue { .. }));
}

#[test]
fn rejects_control_characters_in_key() {
    let mut map = QueryMap::new();
    map.insert("bad\x07key".to_string(), Value::String("value".to_string()));
    let error = stringify(&map).expect_err("control characters in key should be rejected");
    assert!(matches!(error, StringifyError::InvalidKey { .. }));
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
