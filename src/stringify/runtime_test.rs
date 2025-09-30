use super::{StringifyError, StringifyRuntime};
use crate::config::StringifyOptions;
use crate::model::{OrderedMap, QueryMap, Value};

fn stringify_map(map: QueryMap, options: StringifyOptions) -> Result<String, StringifyError> {
    super::stringify_query_map_with(&map, &options)
}

fn options(space_as_plus: bool) -> StringifyOptions {
    StringifyOptions { space_as_plus }
}

fn nested_profile_map() -> QueryMap {
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

    QueryMap::from_iter([("profile", Value::Object(profile))])
}

mod stringify_query_map_with {
    use super::*;

    #[test]
    fn returns_empty_string_for_empty_map() {
        // Arrange
        let map = QueryMap::new();
        let options = StringifyOptions::default();

        // Act
        let result = stringify_map(map, options).expect("empty map should stringify");

        // Assert
        assert!(result.is_empty());
    }

    #[test]
    fn percent_encodes_spaces_when_plus_disabled() {
        // Arrange
        let map = QueryMap::from_iter([("space key", Value::from("space value"))]);
        let options = options(false);

        // Act
        let result = stringify_map(map, options).expect("stringify should succeed");

        // Assert
        assert_eq!(result, "space%20key=space%20value");
    }

    #[test]
    fn converts_spaces_to_plus_when_enabled() {
        // Arrange
        let map = QueryMap::from_iter([("space key", Value::from("space value"))]);
        let options = options(true);

        // Act
        let result = stringify_map(map, options).expect("stringify should succeed");

        // Assert
        assert_eq!(result, "space+key=space+value");
    }

    #[test]
    fn errors_on_control_characters_in_value() {
        // Arrange
        let map = QueryMap::from_iter([("note", Value::from("line1\nline2"))]);
        let options = StringifyOptions::default();

        // Act
        let error = stringify_map(map, options).expect_err("control characters should fail");

        // Assert
        match error {
            StringifyError::InvalidValue { key } => assert_eq!(key, "note"),
            other => panic!("expected InvalidValue error, got {other:?}"),
        }
    }

    #[test]
    fn stringifies_nested_structure_in_order() {
        // Arrange
        let map = nested_profile_map();
        let options = StringifyOptions::default();

        // Act
        let result = stringify_map(map, options).expect("stringify should succeed");

        // Assert
        assert_eq!(
            result,
            "profile%5Bname%5D=Alice&profile%5Bemails%5D%5B0%5D=work%40example.com&profile%5Bemails%5D%5B1%5D=home%40example.com",
        );
    }

    #[test]
    fn errors_on_control_character_in_nested_key() {
        // Arrange
        let profile: OrderedMap<String, Value> = OrderedMap::from_iter([
            ("valid".into(), Value::from("ok")),
            ("bad\nkey".into(), Value::from("oops")),
        ]);
        let map = QueryMap::from_iter([("profile", Value::Object(profile))]);
        let options = StringifyOptions::default();

        // Act
        let error = stringify_map(map, options).expect_err("invalid subkey should fail");

        // Assert
        match error {
            StringifyError::InvalidKey { key } => assert_eq!(key, "profile[bad\nkey]"),
            other => panic!("expected InvalidKey error, got {other:?}"),
        }
    }
}

mod stringify_runtime {
    use super::*;

    #[test]
    fn respects_space_option_on_creation() {
        // Arrange
        let options = options(true);

        // Act
        let runtime = StringifyRuntime::new(&options);

        // Assert
        assert!(runtime.space_as_plus);
    }
}
