use super::super::value_ref::ArenaValueRef;
use super::ArenaValueDeserializer;
use super::deserialize_from_arena_map;
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use crate::parsing_helpers::{make_sequence, make_string};
use crate::serde_adapter::errors::{DeserializeErrorKind, PathSegment};
use serde::Deserialize;

fn make_map<'arena>(arena: &'arena ParseArena) -> ArenaQueryMap<'arena> {
    ArenaQueryMap::with_capacity(arena, 4)
}

fn alloc_key<'arena>(arena: &'arena ParseArena, key: &str) -> &'arena str {
    arena.alloc_str(key)
}

fn deserializer_for<'arena>(value: &'arena ArenaValue<'arena>) -> ArenaValueDeserializer<'arena> {
    ArenaValueDeserializer::new(ArenaValueRef::from_value(value), Vec::new())
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
    fn should_deserialize_struct_into_expected_profile_when_keys_match_struct_fields_then_return_populated_struct()
     {
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
    fn should_reject_invalid_boolean_literal_when_field_value_is_not_bool_then_return_invalid_bool_error()
     {
        // Arrange
        let arena = ParseArena::new();
        let mut map = make_map(&arena);
        map.try_insert_str(&arena, "flag", make_string(&arena, "not-bool"))
            .expect("unique key should insert");

        // Act
        let error = deserialize_from_arena_map::<Flag>(&map)
            .expect_err("invalid boolean literal should fail");

        // Assert
        match error.kind() {
            DeserializeErrorKind::InvalidBool { value } => {
                assert_eq!(value, "not-bool");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert_eq!(error.path(), &[PathSegment::Key("flag".to_string())]);
    }

    #[test]
    fn should_report_unknown_field_with_expected_list_when_map_contains_unexpected_field_then_return_unknown_field_error()
     {
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
        match error.kind() {
            DeserializeErrorKind::UnknownField { field, expected } => {
                assert_eq!(field, "extra");
                assert_eq!(expected, "name, active");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert_eq!(error.path(), &[PathSegment::Key("extra".to_string())]);
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

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Login {
        username: String,
    }

    #[test]
    fn should_report_sequence_length_mismatch_message_when_tuple_length_differs_then_return_length_error()
     {
        // Arrange
        let arena = ParseArena::new();
        let sequence_value = make_sequence(&arena, &["1"]);
        let deserializer = deserializer_for(&sequence_value);

        // Act
        let error =
            <(u8, u8)>::deserialize(deserializer).expect_err("tuple length mismatch should fail");

        // Assert
        assert_eq!(error.to_string(), "expected tuple of length 2, found 1");
    }

    #[test]
    fn should_report_duplicate_field_error_when_struct_field_repeats_then_return_duplicate_field_error()
     {
        // Arrange
        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((alloc_key(&arena, "count"), make_string(&arena, "1")));
        entries.push((alloc_key(&arena, "count"), make_string(&arena, "2")));
        let map_value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };
        let deserializer = deserializer_for(&map_value);

        // Act
        let error =
            Count::deserialize(deserializer).expect_err("duplicate field should be rejected");

        // Assert
        match error.kind() {
            DeserializeErrorKind::DuplicateField { field } => {
                assert_eq!(field, "count");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert_eq!(error.path(), &[PathSegment::Key("count".to_string())]);
    }

    #[test]
    fn should_reject_non_empty_string_for_unit_when_unit_requires_empty_string_then_return_unexpected_type_error()
     {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "not-empty");
        let deserializer = deserializer_for(&value);

        // Act
        let error = <()>::deserialize(deserializer).expect_err("non-empty unit should fail");

        // Assert
        match error.kind() {
            DeserializeErrorKind::UnexpectedType { expected, found } => {
                assert_eq!(*expected, "empty string for unit");
                assert_eq!(*found, "non-empty string");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert_eq!(error.path(), &[]);
    }

    #[test]
    fn should_deserialize_single_character_string_into_char_when_string_has_one_char_then_return_char()
     {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "ß");
        let deserializer = deserializer_for(&value);

        // Act
        let result = char::deserialize(deserializer).expect("single character should deserialize");

        // Assert
        assert_eq!(result, 'ß');
    }

    #[test]
    fn should_reject_multi_character_string_as_char_when_string_has_multiple_chars_then_return_invalid_number_error()
     {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "no");
        let deserializer = deserializer_for(&value);

        // Act
        let error = char::deserialize(deserializer).expect_err("multi-character should fail");

        // Assert
        match error.kind() {
            DeserializeErrorKind::InvalidNumber { value } => {
                assert_eq!(value, "no");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert_eq!(error.path(), &[]);
    }

    #[test]
    fn should_deserialize_unit_from_empty_string_when_string_is_empty_then_return_unit() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "");
        let deserializer = deserializer_for(&value);

        // Act
        let result = <()>::deserialize(deserializer);

        // Assert
        assert!(result.is_ok(), "empty string should deserialize unit");
    }

    #[test]
    fn should_deserialize_unit_struct_from_empty_string_when_string_is_empty_then_return_struct_instance()
     {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "");
        let deserializer = deserializer_for(&value);

        // Act
        let result = Marker::deserialize(deserializer).expect("unit struct should deserialize");

        // Assert
        assert_eq!(result, Marker);
    }

    #[test]
    fn should_deserialize_newtype_struct_from_string_when_string_matches_inner_then_return_wrapper()
    {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "neo");
        let deserializer = deserializer_for(&value);

        // Act
        let result = Wrapper::deserialize(deserializer).expect("newtype struct should deserialize");

        // Assert
        assert_eq!(result, Wrapper("neo".into()));
    }

    #[test]
    fn should_deserialize_tuple_struct_with_matching_length_when_sequence_length_matches_then_return_tuple_struct()
     {
        // Arrange
        let arena = ParseArena::new();
        let value = make_sequence(&arena, &["5", "7"]);
        let deserializer = deserializer_for(&value);

        // Act
        let result = Pair::deserialize(deserializer).expect("tuple struct should deserialize");

        // Assert
        assert_eq!(result, Pair(5, 7));
    }

    #[test]
    fn should_report_expected_string_when_requesting_string_from_array() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_sequence(&arena, &["one"]);
        let deserializer = deserializer_for(&value);

        // Act
        let error =
            String::deserialize(deserializer).expect_err("array should not deserialize to string");

        // Assert
        match error.kind() {
            DeserializeErrorKind::ExpectedString { found } => assert_eq!(*found, "array"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_expected_string_when_requesting_string_from_object() {
        // Arrange
        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((alloc_key(&arena, "value"), make_string(&arena, "text")));
        let map_value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };
        let deserializer = deserializer_for(&map_value);

        // Act
        let error =
            String::deserialize(deserializer).expect_err("map should not deserialize to string");

        // Assert
        match error.kind() {
            DeserializeErrorKind::ExpectedString { found } => assert_eq!(*found, "object"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_invalid_float_literal_when_parsing_float_fails() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "not-a-number");
        let deserializer = deserializer_for(&value);

        // Act
        let error = f64::deserialize(deserializer).expect_err("invalid float should fail");

        // Assert
        match error.kind() {
            DeserializeErrorKind::InvalidNumber { value } => assert_eq!(value, "not-a-number"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_index_in_path_when_sequence_element_fails_then_return_indexed_error() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_sequence(&arena, &["true", "no"]);
        let deserializer = deserializer_for(&value);

        // Act
        let error = <Vec<bool>>::deserialize(deserializer)
            .expect_err("invalid boolean element should fail");

        // Assert
        match error.kind() {
            DeserializeErrorKind::InvalidBool { value } => assert_eq!(value, "no"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert_eq!(error.path(), &[PathSegment::Index(1)]);
    }

    #[test]
    fn should_report_unexpected_type_when_sequence_expected_but_scalar_provided() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "scalar");
        let deserializer = deserializer_for(&value);

        // Act
        let error = <Vec<String>>::deserialize(deserializer)
            .expect_err("scalar cannot deserialize into sequence");

        // Assert
        match error.kind() {
            DeserializeErrorKind::UnexpectedType { expected, found } => {
                assert_eq!(*expected, "array");
                assert_eq!(*found, "string");
            }
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_reject_enumeration_deserialization_when_enum_is_not_supported_then_return_unsupported_error()
     {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "Fast");
        let deserializer = deserializer_for(&value);

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
    fn should_return_borrowed_bytes_when_deserializing_bytes_then_expose_underlying_slice() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "data!");
        let deserializer = deserializer_for(&value);

        struct BytesVisitor;

        impl<'de> serde::de::Visitor<'de> for BytesVisitor {
            type Value = &'de [u8];

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("borrowed byte slice")
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v)
            }
        }

        // Act
        let bytes = serde::de::Deserializer::deserialize_bytes(deserializer, BytesVisitor)
            .expect("bytes should deserialize");

        // Assert
        assert_eq!(bytes, b"data!");
    }

    #[test]
    fn should_collect_owned_bytes_when_invoking_byte_buf_deserializer_then_return_owned_buffer() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "bin");
        let deserializer = deserializer_for(&value);

        struct ByteBufVisitor;

        impl<'de> serde::de::Visitor<'de> for ByteBufVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("owned byte buffer")
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v)
            }
        }

        // Act
        let bytes = serde::de::Deserializer::deserialize_byte_buf(deserializer, ByteBufVisitor)
            .expect("byte buffer should deserialize");

        // Assert
        assert_eq!(bytes, b"bin");
    }

    #[test]
    fn should_deserialize_option_to_some_when_value_present_then_wrap_inner_string() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "maybe");
        let deserializer = deserializer_for(&value);

        // Act
        let result =
            Option::<String>::deserialize(deserializer).expect("option should deserialize as some");

        // Assert
        assert_eq!(result, Some("maybe".to_string()));
    }

    #[test]
    fn should_reject_unit_struct_when_string_not_empty_then_return_unexpected_type_error() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "nope");
        let deserializer = deserializer_for(&value);

        // Act
        let error = Marker::deserialize(deserializer)
            .expect_err("non-empty string cannot deserialize unit struct");

        // Assert
        match error.kind() {
            DeserializeErrorKind::UnexpectedType { expected, found } => {
                assert_eq!(*expected, "Marker");
                assert_eq!(*found, "string");
            }
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_reject_string_value_when_map_requested_then_return_unexpected_type_error() {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "plain");
        let deserializer = deserializer_for(&value);

        // Act
        let error = <std::collections::HashMap<String, String>>::deserialize(deserializer)
            .expect_err("string cannot deserialize into map");

        // Assert
        match error.kind() {
            DeserializeErrorKind::UnexpectedType { expected, found } => {
                assert_eq!(*expected, "object");
                assert_eq!(*found, "string");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert_eq!(error.path(), &[]);
    }

    #[test]
    fn should_report_expected_object_when_struct_requested_from_scalar_then_return_expected_object_error()
     {
        // Arrange
        let arena = ParseArena::new();
        let value = make_string(&arena, "solo");
        let deserializer = deserializer_for(&value);

        // Act
        let error = Login::deserialize(deserializer)
            .expect_err("scalar cannot satisfy struct requirements");

        // Assert
        match error.kind() {
            DeserializeErrorKind::ExpectedObject { struct_name, found } => {
                assert_eq!(*struct_name, "Login");
                assert_eq!(*found, "string");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }
}
