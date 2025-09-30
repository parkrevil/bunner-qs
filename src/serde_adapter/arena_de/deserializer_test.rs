use super::super::value_ref::ArenaValueRef;
use super::ArenaValueDeserializer;
use super::deserialize_from_arena_map;
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use crate::serde_adapter::errors::DeserializeError;
use serde::Deserialize;

fn make_string<'arena>(arena: &'arena ParseArena, value: &str) -> ArenaValue<'arena> {
    ArenaValue::string(arena.alloc_str(value))
}

fn make_sequence<'arena>(arena: &'arena ParseArena, items: &[&'static str]) -> ArenaValue<'arena> {
    let mut values = arena.alloc_vec();
    for item in items {
        values.push(ArenaValue::string(arena.alloc_str(item)));
    }
    ArenaValue::Seq(values)
}

fn make_map<'arena>(arena: &'arena ParseArena) -> ArenaQueryMap<'arena> {
    ArenaQueryMap::with_capacity(arena, 4)
}

fn alloc_key<'arena>(arena: &'arena ParseArena, key: &str) -> &'arena str {
    arena.alloc_str(key)
}

mod deserialize_from_arena_map {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Profile {
        name: String,
        active: bool,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Flag {
        flag: bool,
    }

    #[test]
    fn when_deserializing_struct_it_should_return_expected_profile() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = make_map(&arena);
        map.try_insert_str(&arena, "name", make_string(&arena, "Yuna"))
            .expect("unique key should insert");
        map.try_insert_str(&arena, "active", make_string(&arena, "true"))
            .expect("unique key should insert");

        // Act
        let result =
            deserialize_from_arena_map::<Profile>(&map).expect("deserialization should succeed");

        // Assert
        assert_eq!(
            result,
            Profile {
                name: "Yuna".to_string(),
                active: true,
            }
        );
    }

    #[test]
    fn when_boolean_literal_is_invalid_it_should_return_invalid_bool_error() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = make_map(&arena);
        map.try_insert_str(&arena, "flag", make_string(&arena, "not-bool"))
            .expect("unique key should insert");

        // Act
        let error = deserialize_from_arena_map::<Flag>(&map)
            .expect_err("invalid boolean literal should fail");

        // Assert
        match error {
            DeserializeError::InvalidBool { value } => assert_eq!(value, "not-bool"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn when_struct_contains_unknown_field_it_should_report_error_with_expectations() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = make_map(&arena);
        map.try_insert_str(&arena, "name", make_string(&arena, "Dana"))
            .expect("unique key should insert");
        map.try_insert_str(&arena, "extra", make_string(&arena, "nope"))
            .expect("unique key should insert");

        // Act
        let error =
            deserialize_from_arena_map::<Profile>(&map).expect_err("unknown field should fail");

        // Assert
        match error {
            DeserializeError::UnknownField { field, expected } => {
                assert_eq!(field, "extra");
                assert_eq!(expected, "name, active");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

mod arena_value_deserializer {
    use super::*;

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Count {
        count: u8,
    }

    #[derive(Debug, Deserialize)]
    enum OperatingMode {
        Fast,
        Slow,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Wrapper(String);

    #[derive(Debug, Deserialize, PartialEq)]
    struct Pair(u8, u8);

    #[derive(Debug, Deserialize, PartialEq)]
    struct Marker;

    #[test]
    fn when_sequence_length_is_incorrect_it_should_return_descriptive_message() {
        // Arrange
        let arena = ParseArena::new();
        let sequence_value = make_sequence(&arena, &["1"]);
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&sequence_value),
        };

        // Act
        let error =
            <(u8, u8)>::deserialize(deserializer).expect_err("tuple length mismatch should fail");

        // Assert
        assert_eq!(error.to_string(), "expected tuple of length 2, found 1");
    }

    #[test]
    fn when_struct_field_repeats_it_should_report_duplicate_field_error() {
        // Arrange
        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((alloc_key(&arena, "count"), make_string(&arena, "1")));
        entries.push((alloc_key(&arena, "count"), make_string(&arena, "2")));
        let map_value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&map_value),
        };

        // Act
        let error =
            Count::deserialize(deserializer).expect_err("duplicate field should be rejected");

        // Assert
        match error {
            DeserializeError::DuplicateField { field } => assert_eq!(field, "count"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn when_unit_is_backed_by_non_empty_string_it_should_return_unexpected_type_error() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "not-empty");
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&value),
        };

        // Act
        let error = <()>::deserialize(deserializer).expect_err("non-empty unit should fail");

        // Assert
        match error {
            DeserializeError::UnexpectedType { expected, found } => {
                assert_eq!(expected, "empty string for unit");
                assert_eq!(found, "non-empty string");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn when_char_is_single_character_it_should_deserialize_successfully() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "ß");
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&value),
        };

        // Act
        let result = char::deserialize(deserializer).expect("single character should deserialize");

        // Assert
        assert_eq!(result, 'ß');
    }

    #[test]
    fn when_char_contains_multiple_characters_it_should_report_invalid_number() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "no");
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&value),
        };

        // Act
        let error = char::deserialize(deserializer).expect_err("multi-character should fail");

        // Assert
        match error {
            DeserializeError::InvalidNumber { value } => assert_eq!(value, "no"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn when_unit_is_backed_by_empty_string_it_should_deserialize_successfully() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "");
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&value),
        };

        // Act
        let result = <()>::deserialize(deserializer);

        // Assert
        assert!(result.is_ok(), "empty string should deserialize unit");
    }

    #[test]
    fn when_unit_struct_is_backed_by_empty_string_it_should_deserialize_successfully() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "");
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&value),
        };

        // Act
        let result = Marker::deserialize(deserializer).expect("unit struct should deserialize");

        // Assert
        assert_eq!(result, Marker);
    }

    #[test]
    fn when_newtype_struct_is_backed_by_string_it_should_deserialize_inner_value() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "neo");
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&value),
        };

        // Act
        let result = Wrapper::deserialize(deserializer).expect("newtype struct should deserialize");

        // Assert
        assert_eq!(result, Wrapper("neo".into()));
    }

    #[test]
    fn when_tuple_struct_has_matching_length_it_should_deserialize_elements() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_sequence(&arena, &["5", "7"]);
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&value),
        };

        // Act
        let result = Pair::deserialize(deserializer).expect("tuple struct should deserialize");

        // Assert
        assert_eq!(result, Pair(5, 7));
    }

    #[test]
    fn when_enumeration_is_requested_it_should_return_unsupported_message() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "Fast");
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&value),
        };

        // Act
        let error = OperatingMode::deserialize(deserializer)
            .expect_err("enum deserialization should be unsupported");

        // Assert
        assert_eq!(
            error.to_string(),
            "enum `OperatingMode` with variants [Fast, Slow] cannot be deserialized from query strings"
        );
    }

    #[test]
    fn when_map_is_requested_but_value_is_string_it_should_report_unexpected_type() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "plain");
        let deserializer = ArenaValueDeserializer {
            value: ArenaValueRef::from_value(&value),
        };

        // Act
        let error = <std::collections::HashMap<String, String>>::deserialize(deserializer)
            .expect_err("string cannot deserialize into map");

        // Assert
        match error {
            DeserializeError::UnexpectedType { expected, found } => {
                assert_eq!(expected, "object");
                assert_eq!(found, "string");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
