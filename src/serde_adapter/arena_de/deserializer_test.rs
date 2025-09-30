use super::super::value_ref::ArenaValueRef;
use super::ArenaValueDeserializer;
use super::deserialize_from_arena_map;
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use crate::parsing_helpers::{make_sequence, make_string};
use crate::serde_adapter::errors::DeserializeError;
use serde::Deserialize;

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
    fn deserializes_struct_into_expected_profile() {
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
    fn rejects_invalid_boolean_literal_with_error() {
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
    fn reports_unknown_field_with_expected_list() {
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
    fn reports_sequence_length_mismatch_message() {
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
    fn reports_duplicate_field_error_when_struct_field_repeats() {
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
    fn rejects_non_empty_string_for_unit_with_unexpected_type() {
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
    fn deserializes_single_character_string_into_char() {
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
    fn rejects_multi_character_string_as_char() {
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
    fn deserializes_unit_from_empty_string() {
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
    fn deserializes_unit_struct_from_empty_string() {
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
    fn deserializes_newtype_struct_from_string() {
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
    fn deserializes_tuple_struct_with_matching_length() {
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
    fn rejects_enumeration_deserialization_as_unsupported() {
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
    fn rejects_string_value_when_map_requested() {
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
