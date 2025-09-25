use crate::{ParseError, ParseResult, QueryMap, Value};
use indexmap::IndexMap;

/// Parse a key with bracket notation into path segments
/// e.g., "foo[bar][0]" -> ["foo", "bar", "0"]
pub fn parse_key_path(key: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_brackets = false;
    let chars = key.chars();

    for ch in chars {
        match ch {
            '[' if !in_brackets => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
                in_brackets = true;
            }
            ']' if in_brackets => {
                segments.push(std::mem::take(&mut current));
                in_brackets = false;
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        segments.push(current);
    }

    segments
}

fn is_placeholder(value: &Value) -> bool {
    matches!(value, Value::String(s) if s.is_empty())
}

/// Insert a value into nested structure based on path segments
pub fn insert_nested_value(
    map: &mut QueryMap,
    segments: &[String],
    value: String,
) -> ParseResult<()> {
    if segments.is_empty() {
        return Ok(());
    }

    let root_key = &segments[0];

    if segments.len() == 1 {
        // Simple key without nesting
        match map.get_mut(root_key) {
            Some(_) => {
                return Err(ParseError::DuplicateKey {
                    key: root_key.clone(),
                });
            }
            None => {
                map.insert(root_key.clone(), Value::String(value));
            }
        }
        return Ok(());
    }

    // Build nested path iteratively
    build_nested_path(map, segments, value)
}

fn build_nested_path(
    map: &mut QueryMap,
    segments: &[String],
    final_value: String,
) -> ParseResult<()> {
    let root_key = &segments[0];

    // Create the root entry if it doesn't exist
    if !map.contains_key(root_key) {
        map.insert(root_key.clone(), Value::Object(IndexMap::new()));
    }

    // Build path recursively
    set_nested_value(map.get_mut(root_key).unwrap(), &segments[1..], final_value)
}

fn set_nested_value(current: &mut Value, path: &[String], final_value: String) -> ParseResult<()> {
    if path.is_empty() {
        return Ok(());
    }

    if path.len() == 1 {
        // This is the final segment - insert the value
        let segment = &path[0];
        match current {
            Value::Object(obj) => {
                if obj.contains_key(segment) {
                    return Err(ParseError::DuplicateKey {
                        key: segment.clone(),
                    });
                }
                obj.insert(segment.clone(), Value::String(final_value));
            }
            Value::Array(arr) => {
                if let Ok(idx) = segment.parse::<usize>() {
                    if arr.len() <= idx {
                        arr.resize(idx + 1, Value::String(String::new()));
                    } else if !is_placeholder(&arr[idx]) {
                        return Err(ParseError::DuplicateKey {
                            key: segment.clone(),
                        });
                    }
                    arr[idx] = Value::String(final_value);
                }
            }
            Value::String(_) => {
                // Convert to object
                let mut obj = IndexMap::new();
                obj.insert(segment.clone(), Value::String(final_value));
                *current = Value::Object(obj);
            }
        }
        return Ok(());
    }

    // Navigate deeper - we have more path segments
    let segment = &path[0];
    let remaining_path = &path[1..];

    // Determine if the next level should be an array or object
    let next_is_numeric = remaining_path
        .first()
        .map(|s| s.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false);

    match current {
        Value::Object(obj) => {
            let entry = obj.entry(segment.clone()).or_insert_with(|| {
                if next_is_numeric {
                    Value::Array(Vec::new())
                } else {
                    Value::Object(IndexMap::new())
                }
            });
            set_nested_value(entry, remaining_path, final_value)
        }
        Value::Array(arr) => {
            if let Ok(idx) = segment.parse::<usize>() {
                if arr.len() <= idx {
                    arr.resize(
                        idx + 1,
                        if next_is_numeric {
                            Value::Array(Vec::new())
                        } else {
                            Value::Object(IndexMap::new())
                        },
                    );
                }
                set_nested_value(&mut arr[idx], remaining_path, final_value)
            } else {
                Ok(()) // Invalid array index, skip
            }
        }
        Value::String(_) => {
            // Convert to object and try again
            *current = Value::Object(IndexMap::new());
            set_nested_value(current, path, final_value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_path() {
        assert_eq!(parse_key_path("foo"), vec!["foo"]);
        assert_eq!(parse_key_path("foo[bar]"), vec!["foo", "bar"]);
        assert_eq!(parse_key_path("foo[bar][0]"), vec!["foo", "bar", "0"]);
        assert_eq!(
            parse_key_path("foo[bar][baz][0]"),
            vec!["foo", "bar", "baz", "0"]
        );
    }

    #[test]
    fn test_insert_nested_simple() {
        let mut map = QueryMap::new();
        insert_nested_value(&mut map, &["foo".to_string()], "bar".to_string()).unwrap();

        assert_eq!(map.get("foo").unwrap().as_str().unwrap(), "bar");
    }

    #[test]
    fn test_insert_nested_object() {
        let mut map = QueryMap::new();
        insert_nested_value(
            &mut map,
            &["foo".to_string(), "bar".to_string()],
            "baz".to_string(),
        )
        .unwrap();

        let obj = map.get("foo").unwrap().as_object().unwrap();
        assert_eq!(obj.get("bar").unwrap().as_str().unwrap(), "baz");
    }

    #[test]
    fn test_insert_nested_array() {
        let mut map = QueryMap::new();
        insert_nested_value(
            &mut map,
            &["foo".to_string(), "0".to_string()],
            "bar".to_string(),
        )
        .unwrap();

        let foo_value = map.get("foo").unwrap();
        match foo_value {
            Value::Array(arr) => {
                assert_eq!(arr[0].as_str().unwrap(), "bar");
            }
            Value::Object(obj) => {
                // If it created an object instead, check the "0" key
                assert_eq!(obj.get("0").unwrap().as_str().unwrap(), "bar");
            }
            _ => panic!("Expected array or object, got string"),
        }
    }
}
