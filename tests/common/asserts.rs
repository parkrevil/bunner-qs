use serde_json::{Map as JsonMap, Value};

pub fn assert_str_entry(map: &JsonMap<String, Value>, key: &str, expected: &str) {
    let value = map
        .get(key)
        .unwrap_or_else(|| panic!("missing key `{key}` in object"));
    match value.as_str() {
        Some(actual) => assert_eq!(actual, expected),
        None => panic!("value for `{key}` was not a string: {value:?}"),
    }
}

pub fn expect_object(value: &Value) -> &JsonMap<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("expected object value, got {value:?}"))
}
