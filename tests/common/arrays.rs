use serde_json::Value;

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
