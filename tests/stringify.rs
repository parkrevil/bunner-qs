use bunner_qs::{
    QueryMap, StringifyError, StringifyOptions, stringify, stringify_with_options,
    stringify_with_sorter,
};
fn map(entries: &[(&str, &[&str])]) -> QueryMap {
    let mut result = QueryMap::new();
    for (key, values) in entries {
        result.insert(
            (*key).to_string(),
            values.iter().map(|v| (*v).to_string()).collect(),
        );
    }
    result
}

#[test]
fn stringifies_basic_pairs() {
    let map = map(&[("a", &["1"]), ("b", &["two"])]);
    let encoded = stringify(&map).expect("should stringify");
    assert_eq!(encoded, "a=1&b=two");
}

#[test]
fn uses_plus_for_spaces_when_requested() {
    let map = map(&[("note", &["hello world"])]);
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
    let map = map(&[("a", &["1"])]);
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
    map.insert("a".to_string(), vec!["line\nbreak".to_string()]);
    let error = stringify(&map).expect_err("control characters should be rejected");
    assert!(matches!(error, StringifyError::InvalidValue { .. }));
}

#[test]
fn rejects_control_characters_in_key() {
    let mut map = QueryMap::new();
    map.insert("bad\x07key".to_string(), vec!["value".to_string()]);
    let error = stringify(&map).expect_err("control characters in key should be rejected");
    assert!(matches!(error, StringifyError::InvalidKey { .. }));
}

#[test]
fn respects_custom_sorter() {
    let map = map(&[("b", &["2"]), ("a", &["1"]), ("c", &["3"])]);
    let options = StringifyOptions::default();
    let mut sorter = |left: &str, right: &str| right.cmp(left);
    let encoded = stringify_with_sorter(&map, &options, Some(&mut sorter))
        .expect("custom sorter should succeed");
    assert_eq!(encoded, "c=3&b=2&a=1");
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
