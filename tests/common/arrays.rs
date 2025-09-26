use crate::asserts::expect_path;
use serde_json::Value;

#[track_caller]
pub fn assert_string_array(value: &Value, expected: &[&str]) {
    match value.as_array() {
        Some(items) => {
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
        None => panic!("expected array value, got {value:?}"),
    }
}

#[track_caller]
pub fn assert_string_array_path(value: &Value, path: &[&str], expected: &[&str]) {
    let node = expect_path(value, path);
    assert_string_array(node, expected);
}
