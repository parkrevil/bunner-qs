use bunner_qs::{QueryMap, Value};
use indexmap::IndexMap;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
/// Build a `QueryMap` from key/value string pairs.
pub fn map_from_pairs(pairs: &[(&str, &str)]) -> QueryMap {
    let mut map = QueryMap::new();
    for (key, value) in pairs {
        map.insert((*key).to_owned(), Value::String((*value).to_owned()));
    }
    map
}

pub fn assert_str_entry(map: &IndexMap<String, Value>, key: &str, expected: &str) {
    let value = map
        .get(key)
        .unwrap_or_else(|| panic!("missing key `{key}` in QueryMap"));
    match value.as_str() {
        Some(actual) => assert_eq!(actual, expected),
        None => panic!("value for `{key}` was not a string: {value:?}"),
    }
}

#[allow(dead_code)]
pub fn assert_string_array(value: &Value, expected: &[&str]) {
    match value {
        Value::Array(items) => {
            assert_eq!(items.len(), expected.len(), "array length mismatch");
            for (idx, expected_value) in expected.iter().enumerate() {
                let actual = items[idx]
                    .as_str()
                    .unwrap_or_else(|| panic!("array index {idx} not a string"));
                assert_eq!(
                    actual, *expected_value,
                    "array value mismatch at index {idx}"
                );
            }
        }
        other => panic!("expected array value, got {other:?}"),
    }
}

#[allow(dead_code)]
pub fn expect_object(value: &Value) -> &IndexMap<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("expected object value, got {value:?}"))
}

#[allow(dead_code)]
pub fn expect_array(value: &Value) -> &[Value] {
    value
        .as_array()
        .unwrap_or_else(|| panic!("expected array value, got {value:?}"))
}

#[allow(dead_code)]
pub fn load_fixture(path: &str) -> String {
    let full_path = fixture_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|err| panic!("failed to read fixture `{}`: {}", full_path.display(), err))
}

#[cfg(feature = "serde")]
#[allow(dead_code)]
pub fn load_json_fixture<T>(path: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    let contents = load_fixture(path);
    serde_json::from_str(&contents)
        .unwrap_or_else(|err| panic!("failed to parse JSON fixture `{path}`: {err}"))
}

#[allow(dead_code)]
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}
