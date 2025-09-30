use super::{stringify_query_map_with, StringifyError, StringifyRuntime};
use crate::config::StringifyOptions;
use crate::model::{OrderedMap, QueryMap, Value};

mod stringify_runtime_tests {
    use super::*;

    #[test]
    fn when_query_map_is_empty_it_should_return_empty_string() {
        let map = QueryMap::new();
        let options = StringifyOptions::default();

        let result = stringify_query_map_with(&map, &options).expect("empty map should stringify");

        assert!(result.is_empty());
    }

    #[test]
    fn when_creating_runtime_it_should_respect_space_option() {
    let options = StringifyOptions { space_as_plus: true };

    let runtime = StringifyRuntime::new(&options);

        assert!(runtime.space_as_plus);
    }

    #[test]
    fn when_space_as_plus_is_disabled_it_should_percent_encode_spaces() {
        let map = QueryMap::from_iter([("space key", Value::from("space value"))]);
        let options = StringifyOptions { space_as_plus: false };

        let result = stringify_query_map_with(&map, &options).expect("stringify should succeed");

        assert_eq!(result, "space%20key=space%20value");
    }

    #[test]
    fn when_space_as_plus_is_enabled_it_should_convert_spaces_to_plus() {
        let map = QueryMap::from_iter([("space key", Value::from("space value"))]);
        let options = StringifyOptions { space_as_plus: true };

        let result = stringify_query_map_with(&map, &options).expect("stringify should succeed");

        assert_eq!(result, "space+key=space+value");
    }

    #[test]
    fn when_value_contains_control_characters_it_should_return_invalid_value() {
        let map = QueryMap::from_iter([("note", Value::from("line1\nline2"))]);
        let options = StringifyOptions::default();

        let error = stringify_query_map_with(&map, &options).expect_err("control characters should fail");

        match error {
            StringifyError::InvalidValue { key } => assert_eq!(key, "note"),
            other => panic!("expected InvalidValue error, got {other:?}"),
        }
    }

    #[test]
    fn when_nested_structure_is_stringified_it_should_append_segments_in_order() {
        let profile: OrderedMap<String, Value> = OrderedMap::from_iter([
            ("name".into(), Value::from("Alice")),
            (
                "emails".into(),
                Value::Array(vec![
                    Value::from("work@example.com"),
                    Value::from("home@example.com"),
                ]),
            ),
        ]);
        let map = QueryMap::from_iter([("profile", Value::Object(profile))]);
        let options = StringifyOptions::default();

        let result = stringify_query_map_with(&map, &options).expect("stringify should succeed");

        assert_eq!(
            result,
            "profile%5Bname%5D=Alice&profile%5Bemails%5D%5B0%5D=work%40example.com&profile%5Bemails%5D%5B1%5D=home%40example.com",
        );
    }

    #[test]
    fn when_nested_key_contains_control_character_it_should_return_invalid_key() {
        let profile: OrderedMap<String, Value> = OrderedMap::from_iter([
            ("valid".into(), Value::from("ok")),
            ("bad\nkey".into(), Value::from("oops")),
        ]);
        let map = QueryMap::from_iter([("profile", Value::Object(profile))]);
        let options = StringifyOptions::default();

        let error = stringify_query_map_with(&map, &options).expect_err("invalid subkey should fail");

        match error {
            StringifyError::InvalidKey { key } => {
                assert_eq!(key, "profile[bad\nkey]")
            }
            other => panic!("expected InvalidKey error, got {other:?}"),
        }
    }
}
