#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/options.rs"]
mod options;
#[path = "common/serde_helpers.rs"]
mod serde_helpers;

use asserts::{assert_str_path, assert_string_array_path};
use bunner_qs::{ParseError, parse, parse_with};
use options::try_build_parse_options;
use serde_helpers::{assert_parse_roundtrip, assert_stringify_roundtrip};
use serde_json::{Value, json};
fn parse_value(query: &str) -> Value {
    parse(query).expect("parse should succeed")
}

fn stringify_roundtrip(map: &Value) -> Value {
    assert_stringify_roundtrip(map)
}

fn duplicate_key_key(query: &str) -> String {
    match parse::<Value>(query).expect_err("duplicate key should fail") {
        ParseError::DuplicateKey { key } => key,
        other => panic!("expected duplicate key error, got {other:?}"),
    }
}

fn parse_with_depth(query: &str, depth: usize) -> Result<Value, ParseError> {
    let options = try_build_parse_options(|builder| builder.max_depth(depth))
        .expect("parse options builder should succeed");
    parse_with(query, &options)
}

fn depth_error(query: &str, depth: usize) -> ParseError {
    parse_with_depth(query, depth).expect_err("depth constraint should fail")
}

mod parse_roundtrip_tests {
    use super::*;

    #[test]
    fn when_structures_are_deep_it_should_roundtrip() {
        // Arrange
        let query = "profile[name]=Ada&profile[contacts][email]=ada@example.com&profile[contacts][phones][0]=+44%20123&profile[contacts][phones][1]=+44%20987&profile[meta][created]=2024";

        // Act
        let parsed = assert_parse_roundtrip(query);

        // Assert
        assert_str_path(&parsed, &["profile", "name"], "Ada");
        assert_str_path(
            &parsed,
            &["profile", "contacts", "email"],
            "ada@example.com",
        );
        assert_string_array_path(
            &parsed,
            &["profile", "contacts", "phones"],
            &["+44 123", "+44 987"],
        );
        assert_str_path(&parsed, &["profile", "meta", "created"], "2024");
    }

    #[test]
    fn when_arrays_have_gaps_it_should_preserve_objects() {
        // Arrange
        let query = "key[0][a]=1&key[1]=&key[2][b]=2";

        // Act
        let parsed = parse_value(query);
        let key_array = parsed
            .get("key")
            .and_then(Value::as_array)
            .expect("key should parse as array");

        // Assert
        assert_eq!(key_array.len(), 3);
        assert_eq!(key_array[0].get("a").and_then(Value::as_str), Some("1"));
        assert_eq!(key_array[1].as_str(), Some(""));
        assert_eq!(key_array[2].get("b").and_then(Value::as_str), Some("2"));
    }

    #[test]
    fn when_append_pattern_is_uniform_it_should_collect_values() {
        // Arrange
        let query = "tags[]=rust&tags[]=serde";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_string_array_path(&parsed, &["tags"], &["rust", "serde"]);
    }

    #[test]
    fn when_numeric_pattern_is_uniform_it_should_collect_values() {
        // Arrange
        let query = "items[0]=apple&items[1]=banana";

        // Act
        let parsed = parse_value(query);

        // Assert
        assert_string_array_path(&parsed, &["items"], &["apple", "banana"]);
    }

    #[test]
    fn when_stringifying_numeric_indices_it_should_preserve_order() {
        // Arrange
        let map = json!({ "items": ["alpha", "beta", "gamma"] });

        // Act
        let reparsed = stringify_roundtrip(&map);

        // Assert
        assert_string_array_path(&reparsed, &["items"], &["alpha", "beta", "gamma"]);
    }
}

mod parse_conflict_tests {
    use super::*;

    #[test]
    fn when_array_scalar_conflict_occurs_it_should_return_duplicate_key() {
        // Arrange
        let query = "items[0]=apple&items[0][kind]=fruit";

        // Act
        let key = duplicate_key_key(query);

        // Assert
        assert_eq!(key, "items");
    }

    #[test]
    fn when_array_object_conflict_occurs_it_should_return_duplicate_key() {
        // Arrange
        let query = "items[0][kind]=fruit&items[0]=apple";

        // Act
        let key = duplicate_key_key(query);

        // Assert
        assert!(matches!(key.as_str(), "items" | "0"));
    }

    #[test]
    fn when_append_and_numeric_patterns_mix_it_should_return_duplicate_key() {
        // Arrange
        let query = "key[]=1&key[0]=1";

        // Act
        let key = duplicate_key_key(query);

        // Assert
        assert_eq!(key, "key");
    }

    #[test]
    fn when_scalar_and_nested_mix_it_should_return_duplicate_key() {
        // Arrange
        let query = "foo=1&foo[bar]=2";

        // Act
        let key = duplicate_key_key(query);

        // Assert
        assert_eq!(key, "foo");
    }

    #[test]
    fn when_scalar_duplicates_exist_it_should_return_duplicate_key() {
        // Arrange
        let query = "foo=1&foo=2";

        // Act
        let key = duplicate_key_key(query);

        // Assert
        assert_eq!(key, "foo");
    }

    #[test]
    fn when_numeric_indexes_are_sparse_it_should_return_duplicate_key() {
        // Arrange
        let query = "items[0]=apple&items[2]=cherry";

        // Act
        let key = duplicate_key_key(query);

        // Assert
        assert_eq!(key, "items");
    }
}

mod parse_limits_tests {
    use super::*;

    #[test]
    fn when_depth_exceeds_limit_it_should_report_depth_error() {
        // Arrange
        let query = "profile[contacts][phones][0][number]=+44%20123";

        // Act
        parse_with_depth(query, 4).expect("depth of four should succeed");
        let error = depth_error(query, 2);

        // Assert
        match error {
            ParseError::DepthExceeded { key, limit } => {
                assert_eq!(key, "profile[contacts][phones][0][number]");
                assert_eq!(limit, 2);
            }
            other => panic!("expected depth exceeded error, got {other:?}"),
        }
    }
}
