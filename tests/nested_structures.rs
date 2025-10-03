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
    fn should_roundtrip_deep_structures_when_nested_contacts_present() {
        let query = "profile[name]=Ada&profile[contacts][email]=ada@example.com&profile[contacts][phones][0]=+44%20123&profile[contacts][phones][1]=+44%20987&profile[meta][created]=2024";

        let parsed = assert_parse_roundtrip(query);

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
    fn should_preserve_objects_when_arrays_have_gaps() {
        let query = "key[0][a]=1&key[1]=&key[2][b]=2";

        let parsed = parse_value(query);
        let key_array = parsed
            .get("key")
            .and_then(Value::as_array)
            .expect("key should parse as array");

        assert_eq!(key_array.len(), 3);
        assert_eq!(key_array[0].get("a").and_then(Value::as_str), Some("1"));
        assert_eq!(key_array[1].as_str(), Some(""));
        assert_eq!(key_array[2].get("b").and_then(Value::as_str), Some("2"));
    }

    #[test]
    fn should_collect_values_when_uniform_append_pattern_used() {
        let query = "tags[]=rust&tags[]=serde";

        let parsed = parse_value(query);

        assert_string_array_path(&parsed, &["tags"], &["rust", "serde"]);
    }

    #[test]
    fn should_collect_values_when_uniform_numeric_pattern_used() {
        let query = "items[0]=apple&items[1]=banana";

        let parsed = parse_value(query);

        assert_string_array_path(&parsed, &["items"], &["apple", "banana"]);
    }

    #[test]
    fn should_preserve_order_when_stringifying_numeric_indices() {
        let map = json!({ "items": ["alpha", "beta", "gamma"] });

        let reparsed = stringify_roundtrip(&map);

        assert_string_array_path(&reparsed, &["items"], &["alpha", "beta", "gamma"]);
    }
}

mod parse_conflict_tests {
    use super::*;

    #[test]
    fn should_return_duplicate_key_when_array_and_scalar_conflict() {
        let query = "items[0]=apple&items[0][kind]=fruit";

        let key = duplicate_key_key(query);

        assert_eq!(key, "items");
    }

    #[test]
    fn should_return_duplicate_key_when_array_and_object_conflict() {
        let query = "items[0][kind]=fruit&items[0]=apple";

        let key = duplicate_key_key(query);

        assert!(matches!(key.as_str(), "items" | "0"));
    }

    #[test]
    fn should_return_duplicate_key_when_append_and_numeric_patterns_mix() {
        let query = "key[]=1&key[0]=1";

        let key = duplicate_key_key(query);

        assert_eq!(key, "key");
    }

    #[test]
    fn should_return_duplicate_key_when_scalar_and_nested_patterns_mix() {
        let query = "foo=1&foo[bar]=2";

        let key = duplicate_key_key(query);

        assert_eq!(key, "foo");
    }

    #[test]
    fn should_return_duplicate_key_when_scalar_duplicates_present() {
        let query = "foo=1&foo=2";

        let key = duplicate_key_key(query);

        assert_eq!(key, "foo");
    }

    #[test]
    fn should_return_duplicate_key_when_numeric_indexes_are_sparse() {
        let query = "items[0]=apple&items[2]=cherry";

        let key = duplicate_key_key(query);

        assert_eq!(key, "items");
    }
}

mod parse_limits_tests {
    use super::*;

    #[test]
    fn should_report_depth_error_when_limit_is_exceeded() {
        let query = "profile[contacts][phones][0][number]=+44%20123";

        parse_with_depth(query, 4).expect("depth of four should succeed");
        let error = depth_error(query, 2);

        match error {
            ParseError::DepthExceeded { key, limit } => {
                assert_eq!(key, "profile[contacts][phones][0][number]");
                assert_eq!(limit, 2);
            }
            other => panic!("expected depth exceeded error, got {other:?}"),
        }
    }
}
