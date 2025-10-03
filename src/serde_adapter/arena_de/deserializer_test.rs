use super::super::value_ref::ArenaValueRef;
use super::{
    ArenaMapDeserializer, ArenaStructDeserializer, ArenaValueDeserializer,
    deserialize_from_arena_map,
};
use crate::arena_helpers::{alloc_key, map_with_capacity};
use crate::parsing::arena::{ArenaValue, ParseArena};
use crate::parsing_helpers::{make_sequence, make_string};
use crate::serde_adapter::errors::{DeserializeErrorKind, PathSegment};
use serde::Deserialize;

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
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 4);
        map.try_insert_str(&arena, "name", make_string(&arena, "Yuna"))
            .expect("unique key should insert");
        map.try_insert_str(&arena, "active", make_string(&arena, "true"))
            .expect("unique key should insert");

        let result =
            deserialize_from_arena_map::<Profile>(&map).expect("deserialization should succeed");

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
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 4);
        map.try_insert_str(&arena, "flag", make_string(&arena, "not-bool"))
            .expect("unique key should insert");

        let error = deserialize_from_arena_map::<Flag>(&map)
            .expect_err("invalid boolean literal should fail");

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
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 4);
        map.try_insert_str(&arena, "name", make_string(&arena, "Dana"))
            .expect("unique key should insert");
        map.try_insert_str(&arena, "extra", make_string(&arena, "nope"))
            .expect("unique key should insert");

        let error =
            deserialize_from_arena_map::<Profile>(&map).expect_err("unknown field should fail");

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
    use serde::de::{Deserializer, SeqAccess, Visitor};
    use std::fmt;

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

    #[derive(Debug, Deserialize, PartialEq)]
    struct Trio(u8, u8, u8);

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Login {
        username: String,
    }

    #[allow(dead_code)]
    struct UnitSeed;

    impl<'de> serde::de::DeserializeSeed<'de> for UnitSeed {
        type Value = ();

        fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            Ok(())
        }
    }

    struct InvalidBoolSeed;

    impl<'de> serde::de::DeserializeSeed<'de> for InvalidBoolSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            bool::deserialize(deserializer).map(|_| ())
        }
    }

    #[test]
    fn should_report_sequence_length_mismatch_message_when_tuple_length_differs_then_return_length_error()
     {
        let arena = ParseArena::new();
        let sequence_value = make_sequence(&arena, &["1"]);
        let deserializer = deserializer_for(&sequence_value);

        let error =
            <(u8, u8)>::deserialize(deserializer).expect_err("tuple length mismatch should fail");

        assert_eq!(error.to_string(), "expected tuple of length 2, found 1");
    }

    #[test]
    fn should_report_tuple_struct_length_mismatch_then_return_length_error() {
        let arena = ParseArena::new();
        let sequence_value = make_sequence(&arena, &["1"]);
        let deserializer = deserializer_for(&sequence_value);

        let error =
            Pair::deserialize(deserializer).expect_err("tuple struct length mismatch should fail");

        assert_eq!(
            error.to_string(),
            "expected tuple struct `Pair` with 2 elements, found 1"
        );
    }

    #[test]
    fn should_dispatch_deserialize_any_to_sequence_branch_when_value_is_sequence() {
        struct CountingVisitor;

        impl<'de> Visitor<'de> for CountingVisitor {
            type Value = usize;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence of values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut count = 0;
                while seq.next_element::<String>()?.is_some() {
                    count += 1;
                }
                Ok(count)
            }
        }

        let arena = ParseArena::new();
        let sequence_value = make_sequence(&arena, &["one", "two"]);
        let deserializer = deserializer_for(&sequence_value);

        let visited = deserializer
            .deserialize_any(CountingVisitor)
            .expect("sequence branch should deserialize");

        assert_eq!(visited, 2);
    }

    #[test]
    fn should_report_tuple_struct_length_mismatch_with_dynamic_length_then_format_message() {
        let arena = ParseArena::new();
        let mut sequence_value = ArenaValue::seq_with_capacity(&arena, 0);
        if let ArenaValue::Seq(items) = &mut sequence_value {
            for label in ["1", "2"] {
                items.push(ArenaValue::string(arena.alloc_str(label)));
            }
        }
        let deserializer = deserializer_for(&sequence_value);

        let error =
            Trio::deserialize(deserializer).expect_err("tuple struct length mismatch should fail");

        assert_eq!(
            error.to_string(),
            "expected tuple struct `Trio` with 3 elements, found 2"
        );
    }

    #[test]
    fn should_report_duplicate_field_error_when_struct_field_repeats_then_return_duplicate_field_error()
     {
        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((alloc_key(&arena, "count"), make_string(&arena, "1")));
        entries.push((alloc_key(&arena, "count"), make_string(&arena, "2")));
        let map_value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };
        let deserializer = deserializer_for(&map_value);

        let error =
            Count::deserialize(deserializer).expect_err("duplicate field should be rejected");

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
        let arena = ParseArena::new();
        let value = make_string(&arena, "not-empty");
        let deserializer = deserializer_for(&value);

        let error = <()>::deserialize(deserializer).expect_err("non-empty unit should fail");

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
        let arena = ParseArena::new();
        let value = make_string(&arena, "ß");
        let deserializer = deserializer_for(&value);

        let result = char::deserialize(deserializer).expect("single character should deserialize");

        assert_eq!(result, 'ß');
    }

    #[test]
    fn should_reject_multi_character_string_as_char_when_string_has_multiple_chars_then_return_invalid_number_error()
     {
        let arena = ParseArena::new();
        let value = make_string(&arena, "no");
        let deserializer = deserializer_for(&value);

        let error = char::deserialize(deserializer).expect_err("multi-character should fail");

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
        let arena = ParseArena::new();
        let value = make_string(&arena, "");
        let deserializer = deserializer_for(&value);

        let result = <()>::deserialize(deserializer);

        assert!(result.is_ok(), "empty string should deserialize unit");
    }

    #[test]
    fn should_deserialize_unit_struct_from_empty_string_when_string_is_empty_then_return_struct_instance()
     {
        let arena = ParseArena::new();
        let value = make_string(&arena, "");
        let deserializer = deserializer_for(&value);

        let result = Marker::deserialize(deserializer).expect("unit struct should deserialize");

        assert_eq!(result, Marker);
    }

    #[test]
    fn should_deserialize_newtype_struct_from_string_when_string_matches_inner_then_return_wrapper()
    {
        let arena = ParseArena::new();
        let value = make_string(&arena, "neo");
        let deserializer = deserializer_for(&value);

        let result = Wrapper::deserialize(deserializer).expect("newtype struct should deserialize");

        assert_eq!(result, Wrapper("neo".into()));
    }

    #[test]
    fn should_deserialize_tuple_struct_with_matching_length_when_sequence_length_matches_then_return_tuple_struct()
     {
        let arena = ParseArena::new();
        let value = make_sequence(&arena, &["5", "7"]);
        let deserializer = deserializer_for(&value);

        let result = Pair::deserialize(deserializer).expect("tuple struct should deserialize");

        assert_eq!(result, Pair(5, 7));
    }

    #[test]
    fn should_dispatch_deserialize_any_to_sequence_branch() {
        use serde::de::{Deserializer, SeqAccess, Visitor};
        use std::fmt;

        struct SeqLenVisitor;

        impl<'de> Visitor<'de> for SeqLenVisitor {
            type Value = usize;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence value")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut count = 0;
                while let Some::<String>(_item) = seq.next_element()? {
                    count += 1;
                }
                Ok(count)
            }
        }

        let arena = ParseArena::new();
        let sequence_value = make_sequence(&arena, &["alpha", "beta", "gamma"]);
        let deserializer = deserializer_for(&sequence_value);

        let count = deserializer
            .deserialize_any(SeqLenVisitor)
            .expect("deserialize_any should visit sequence");

        assert_eq!(count, 3);
    }

    #[test]
    fn should_dispatch_deserialize_any_to_string_branch() {
        use serde::de::{Deserializer, Visitor};
        use std::fmt;

        struct BorrowVisitor;

        impl<'de> Visitor<'de> for BorrowVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a borrowed string")
            }

            fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(value.to_string())
            }
        }

        let arena = ParseArena::new();
        let value = make_string(&arena, "orion");
        let deserializer = deserializer_for(&value);

        let text = deserializer
            .deserialize_any(BorrowVisitor)
            .expect("deserialize_any should visit borrowed string");

        assert_eq!(text, "orion");
    }

    #[test]
    fn should_dispatch_deserialize_any_to_map_branch() {
        use serde::de::{Deserializer, MapAccess, Visitor};
        use std::fmt;

        struct MapCollector;

        impl<'de> Visitor<'de> for MapCollector {
            type Value = Vec<(String, String)>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map representing object values")
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some((key, value)) = access.next_entry::<String, String>()? {
                    entries.push((key, value));
                }
                Ok(entries)
            }
        }

        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((alloc_key(&arena, "alpha"), make_string(&arena, "1")));
        entries.push((alloc_key(&arena, "beta"), make_string(&arena, "2")));
        let map_value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };
        let deserializer = deserializer_for(&map_value);

        let collected = deserializer
            .deserialize_any(MapCollector)
            .expect("deserialize_any should visit map");

        assert_eq!(
            collected,
            vec![("alpha".into(), "1".into()), ("beta".into(), "2".into())]
        );
    }

    #[test]
    fn should_report_unexpected_type_when_tuple_expected_but_value_is_string() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "not-a-tuple");
        let deserializer = deserializer_for(&value);

        let error = <(u8, u8)>::deserialize(deserializer)
            .expect_err("string should not deserialize as tuple");

        match error.kind() {
            DeserializeErrorKind::UnexpectedType { expected, found } => {
                assert_eq!(*expected, "tuple");
                assert_eq!(*found, "string");
            }
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert_eq!(error.path(), &[]);
    }

    #[test]
    fn should_deserialize_all_integer_types_successfully() {
        let arena = ParseArena::new();
        let positive = make_string(&arena, "42");
        let negative = make_string(&arena, "-17");

        macro_rules! assert_signed {
            ($ty:ty, $value:expr, $expected:expr) => {{
                let deserializer = deserializer_for(&$value);
                let parsed = <$ty>::deserialize(deserializer).expect("signed integer should parse");
                assert_eq!(parsed, $expected);
            }};
        }

        macro_rules! assert_unsigned {
            ($ty:ty, $value:expr, $expected:expr) => {{
                let deserializer = deserializer_for(&$value);
                let parsed =
                    <$ty>::deserialize(deserializer).expect("unsigned integer should parse");
                assert_eq!(parsed, $expected);
            }};
        }

        assert_signed!(i8, positive, 42);
        assert_signed!(i16, positive, 42);
        assert_signed!(i32, positive, 42);
        assert_signed!(i64, positive, 42);
        assert_signed!(i128, positive, 42);
        assert_signed!(i32, negative, -17);
        assert_signed!(i64, negative, -17);
        assert_signed!(i128, negative, -17);

        assert_unsigned!(u8, positive, 42);
        assert_unsigned!(u16, positive, 42);
        assert_unsigned!(u32, positive, 42);
        assert_unsigned!(u64, positive, 42);
        assert_unsigned!(u128, positive, 42);
    }

    #[test]
    fn should_report_invalid_number_when_unsigned_parse_fails_then_return_invalid_number_error() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "12nope");
        let deserializer = deserializer_for(&value);

        let error = u16::deserialize(deserializer).expect_err("invalid digits should fail");

        match error.kind() {
            DeserializeErrorKind::InvalidNumber { value } => assert_eq!(value, "12nope"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_deserialize_float_variants_successfully() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "3.50");

        let f32_value = f32::deserialize(deserializer_for(&value)).expect("f32 should parse");
        let f64_value = f64::deserialize(deserializer_for(&value)).expect("f64 should parse");

        assert!((f32_value - 3.5).abs() < f32::EPSILON);
        assert!((f64_value - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn should_deserialize_borrowed_and_owned_strings_successfully() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "text");

        let borrowed: &str =
            <&str>::deserialize(deserializer_for(&value)).expect("borrowed str should deserialize");
        let owned: String =
            String::deserialize(deserializer_for(&value)).expect("owned string should deserialize");

        assert_eq!(borrowed, "text");
        assert_eq!(owned, "text");
    }

    #[test]
    fn should_report_expected_string_when_requesting_string_from_array() {
        let arena = ParseArena::new();
        let value = make_sequence(&arena, &["one"]);
        let deserializer = deserializer_for(&value);

        let error =
            String::deserialize(deserializer).expect_err("array should not deserialize to string");

        match error.kind() {
            DeserializeErrorKind::ExpectedString { found } => assert_eq!(*found, "array"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_expected_string_when_requesting_string_from_object() {
        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((alloc_key(&arena, "value"), make_string(&arena, "text")));
        let map_value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };
        let deserializer = deserializer_for(&map_value);

        let error =
            String::deserialize(deserializer).expect_err("map should not deserialize to string");

        match error.kind() {
            DeserializeErrorKind::ExpectedString { found } => assert_eq!(*found, "object"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_expected_string_when_number_requested_from_array_then_return_expected_string_error()
     {
        let arena = ParseArena::new();
        let sequence = make_sequence(&arena, &["5"]);
        let deserializer = deserializer_for(&sequence);

        let error =
            u8::deserialize(deserializer).expect_err("array should not deserialize to number");

        match error.kind() {
            DeserializeErrorKind::ExpectedString { found } => assert_eq!(*found, "array"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_expected_string_when_bool_requested_from_object_then_return_expected_string_error()
     {
        let arena = ParseArena::new();
        let map_value = ArenaValue::Map {
            entries: arena.alloc_vec(),
            index: Default::default(),
        };
        let deserializer = deserializer_for(&map_value);

        let error =
            bool::deserialize(deserializer).expect_err("map should not deserialize to bool");

        match error.kind() {
            DeserializeErrorKind::ExpectedString { found } => assert_eq!(*found, "object"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_invalid_float_literal_when_parsing_float_fails() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "not-a-number");
        let deserializer = deserializer_for(&value);

        let error = f64::deserialize(deserializer).expect_err("invalid float should fail");

        match error.kind() {
            DeserializeErrorKind::InvalidNumber { value } => assert_eq!(value, "not-a-number"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_index_in_path_when_sequence_element_fails_then_return_indexed_error() {
        let arena = ParseArena::new();
        let value = make_sequence(&arena, &["true", "no"]);
        let deserializer = deserializer_for(&value);

        let error = <Vec<bool>>::deserialize(deserializer)
            .expect_err("invalid boolean element should fail");

        match error.kind() {
            DeserializeErrorKind::InvalidBool { value } => assert_eq!(value, "no"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert_eq!(error.path(), &[PathSegment::Index(1)]);
    }

    #[test]
    fn should_report_unexpected_type_when_sequence_expected_but_scalar_provided() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "scalar");
        let deserializer = deserializer_for(&value);

        let error = <Vec<String>>::deserialize(deserializer)
            .expect_err("scalar cannot deserialize into sequence");

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
    fn should_report_unexpected_type_for_tuple_when_scalar_provided() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "scalar");

        let error = <(u8, u8)>::deserialize(deserializer_for(&value))
            .expect_err("scalar cannot deserialize into tuple");

        match error.kind() {
            DeserializeErrorKind::UnexpectedType { expected, found } => {
                assert_eq!(*expected, "tuple");
                assert_eq!(*found, "string");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_unexpected_type_for_tuple_struct_when_scalar_provided() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "scalar");

        let error = Pair::deserialize(deserializer_for(&value))
            .expect_err("scalar cannot deserialize into tuple struct");

        match error.kind() {
            DeserializeErrorKind::UnexpectedType { expected, found } => {
                assert_eq!(*expected, "Pair");
                assert_eq!(*found, "string");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_unexpected_type_when_sequence_requested_from_map() {
        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((alloc_key(&arena, "key"), make_string(&arena, "value")));
        let map_value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };
        let deserializer = deserializer_for(&map_value);

        let error = <Vec<String>>::deserialize(deserializer)
            .expect_err("map cannot deserialize into sequence");

        match error.kind() {
            DeserializeErrorKind::UnexpectedType { expected, found } => {
                assert_eq!(*expected, "array");
                assert_eq!(*found, "object");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_report_unexpected_type_when_map_requested_from_sequence() {
        let arena = ParseArena::new();
        let sequence = make_sequence(&arena, &["value"]);
        let deserializer = deserializer_for(&sequence);

        let error = <std::collections::HashMap<String, String>>::deserialize(deserializer)
            .expect_err("sequence cannot deserialize into map");

        match error.kind() {
            DeserializeErrorKind::UnexpectedType { expected, found } => {
                assert_eq!(*expected, "object");
                assert_eq!(*found, "array");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_reject_enumeration_deserialization_when_enum_is_not_supported_then_return_unsupported_error()
     {
        let arena = ParseArena::new();
        let value = make_string(&arena, "Fast");
        let deserializer = deserializer_for(&value);

        let error = OperatingMode::deserialize(deserializer)
            .expect_err("enum deserialization should be unsupported");

        assert_eq!(
            error.to_string(),
            "enum `OperatingMode` with variants [Fast, Slow] cannot be deserialized from query strings"
        );
    }

    #[test]
    fn should_return_borrowed_bytes_when_deserializing_bytes_then_expose_underlying_slice() {
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

        let bytes = serde::de::Deserializer::deserialize_bytes(deserializer, BytesVisitor)
            .expect("bytes should deserialize");

        assert_eq!(bytes, b"data!");
    }

    #[test]
    fn should_collect_owned_bytes_when_invoking_byte_buf_deserializer_then_return_owned_buffer() {
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

        let bytes = serde::de::Deserializer::deserialize_byte_buf(deserializer, ByteBufVisitor)
            .expect("byte buffer should deserialize");

        assert_eq!(bytes, b"bin");
    }

    #[test]
    fn should_deserialize_option_to_some_when_value_present_then_wrap_inner_string() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "maybe");
        let deserializer = deserializer_for(&value);

        let result =
            Option::<String>::deserialize(deserializer).expect("option should deserialize as some");

        assert_eq!(result, Some("maybe".to_string()));
    }

    #[test]
    fn should_reject_unit_struct_when_string_not_empty_then_return_unexpected_type_error() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "nope");
        let deserializer = deserializer_for(&value);

        let error = Marker::deserialize(deserializer)
            .expect_err("non-empty string cannot deserialize unit struct");

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
        let arena = ParseArena::new();
        let value = make_string(&arena, "plain");
        let deserializer = deserializer_for(&value);

        let error = <std::collections::HashMap<String, String>>::deserialize(deserializer)
            .expect_err("string cannot deserialize into map");

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
        let arena = ParseArena::new();
        let value = make_string(&arena, "solo");
        let deserializer = deserializer_for(&value);

        let error = Login::deserialize(deserializer)
            .expect_err("scalar cannot satisfy struct requirements");

        match error.kind() {
            DeserializeErrorKind::ExpectedObject { struct_name, found } => {
                assert_eq!(*struct_name, "Login");
                assert_eq!(*found, "string");
            }
            other => panic!("unexpected kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_deserialize_identifier_from_string_when_identifier_requested() {
        let arena = ParseArena::new();
        let value = make_string(&arena, "identifier");
        let deserializer = deserializer_for(&value);

        struct IdentifierVisitor;

        impl<'de> serde::de::Visitor<'de> for IdentifierVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("identifier string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.to_string())
            }
        }

        let identifier =
            serde::de::Deserializer::deserialize_identifier(deserializer, IdentifierVisitor)
                .expect("identifier should deserialize");

        assert_eq!(identifier, "identifier");
    }

    #[test]
    fn should_ignore_any_value_when_deserializing_ignored_any_then_return_unit() {
        let arena = ParseArena::new();
        let sequence = make_sequence(&arena, &["1", "2"]);
        let deserializer = deserializer_for(&sequence);

        serde::de::Deserializer::deserialize_ignored_any(deserializer, serde::de::IgnoredAny)
            .expect("ignored any should deserialize to unit");
    }

    #[test]
    fn should_deserialize_hashmap_from_map_value_then_collect_entries() {
        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((alloc_key(&arena, "name"), make_string(&arena, "neo")));
        entries.push((alloc_key(&arena, "role"), make_string(&arena, "chosen")));
        let map_value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };
        let deserializer = deserializer_for(&map_value);

        let result = <std::collections::HashMap<String, String>>::deserialize(deserializer)
            .expect("map should deserialize");

        assert_eq!(result.get("name"), Some(&"neo".to_string()));
        assert_eq!(result.get("role"), Some(&"chosen".to_string()));
    }

    #[test]
    fn should_error_when_map_value_requested_without_pending_key_then_return_missing_value_error() {
        let arena = ParseArena::new();
        let entries = arena.alloc_vec();
        let mut map_deserializer = ArenaMapDeserializer {
            iter: entries.iter(),
            value: None,
            path: Vec::new(),
            pending_key: None,
        };

        let error = serde::de::MapAccess::next_value_seed(&mut map_deserializer, UnitSeed)
            .expect_err("missing map value should error");

        match error.kind() {
            DeserializeErrorKind::Message(message) => {
                assert_eq!(message, "value missing for map entry");
            }
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_error_when_struct_value_requested_without_pending_key_then_return_missing_value_error()
     {
        let arena = ParseArena::new();
        let entries = arena.alloc_vec();
        let mut struct_deserializer = ArenaStructDeserializer {
            iter: entries.iter(),
            value: None,
            allowed: &["field"],
            seen: std::collections::HashSet::new(),
            path: Vec::new(),
            pending_key: None,
        };

        let error = serde::de::MapAccess::next_value_seed(&mut struct_deserializer, UnitSeed)
            .expect_err("missing struct field should error");

        match error.kind() {
            DeserializeErrorKind::Message(message) => {
                assert_eq!(message, "value missing for struct field");
            }
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert!(error.path().is_empty());
    }

    #[test]
    fn should_label_unknown_key_when_map_value_seed_runs_without_pending_key() {
        let arena = ParseArena::new();
        let missing_value = make_string(&arena, "maybe");
        let empty_entries: [(&str, ArenaValue); 0] = [];
        let mut map_deserializer = ArenaMapDeserializer {
            iter: empty_entries.iter(),
            value: Some(&missing_value),
            path: vec![PathSegment::Key("root".to_string())],
            pending_key: None,
        };

        let error = serde::de::MapAccess::next_value_seed(&mut map_deserializer, InvalidBoolSeed)
            .expect_err("invalid bool literal should error");

        match error.kind() {
            DeserializeErrorKind::InvalidBool { value } => assert_eq!(value, "maybe"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert_eq!(
            error.path(),
            &[
                PathSegment::Key("root".to_string()),
                PathSegment::Key("<unknown>".to_string()),
            ]
        );
    }

    #[test]
    fn should_label_unknown_key_when_struct_value_seed_runs_without_pending_key() {
        use std::collections::HashSet;

        let arena = ParseArena::new();
        let missing_value = make_string(&arena, "maybe");
        let empty_entries: [(&str, ArenaValue); 0] = [];
        let mut struct_deserializer = ArenaStructDeserializer {
            iter: empty_entries.iter(),
            value: Some(&missing_value),
            allowed: &["field"],
            seen: HashSet::new(),
            path: vec![PathSegment::Key("root".to_string())],
            pending_key: None,
        };

        let error =
            serde::de::MapAccess::next_value_seed(&mut struct_deserializer, InvalidBoolSeed)
                .expect_err("invalid bool literal should error");

        match error.kind() {
            DeserializeErrorKind::InvalidBool { value } => assert_eq!(value, "maybe"),
            other => panic!("unexpected error kind: {other:?}"),
        }
        assert_eq!(
            error.path(),
            &[
                PathSegment::Key("root".to_string()),
                PathSegment::Key("<unknown>".to_string()),
            ]
        );
    }
}
