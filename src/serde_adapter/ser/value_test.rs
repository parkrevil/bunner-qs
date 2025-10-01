use super::*;
use crate::model::Value;
use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, SerializeTuple, Serializer};

mod serialize_to_query_map {
    use super::*;

    #[derive(Serialize)]
    struct Profile<'a> {
        name: &'a str,
        age: u32,
    }

    #[test]
    fn should_serialize_struct_into_ordered_map_when_struct_has_simple_fields_then_store_field_strings() {
        // Arrange
        let profile = Profile {
            name: "Mina",
            age: 23,
        };

        // Act
        let map = serialize_to_query_map(&profile).expect("struct should serialize into a map");

        // Assert
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("name"), Some(&Value::String("Mina".into())));
        assert_eq!(map.get("age"), Some(&Value::String("23".into())));
    }

    #[test]
    fn should_reject_string_top_level_when_serializing_query_map_then_return_top_level_error() {
        // Arrange
        let value = "hello";

        // Act
        let error = serialize_to_query_map(&value).expect_err("non-object top level should fail");

        // Assert
        match error {
            SerializeError::TopLevel(found) => assert_eq!(found, "string"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn should_report_unexpected_skip_when_top_level_option_is_none_then_return_unexpected_skip_error() {
        // Arrange
        let value = Option::<String>::None;

        // Act
        let error = serialize_to_query_map(&value).expect_err("none should yield unexpected skip");

        // Assert
        match error {
            SerializeError::UnexpectedSkip => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn should_reject_array_top_level_when_serializing_query_map_then_return_top_level_error() {
        // Arrange
        let values = vec!["hello", "world"];

        // Act
        let error = serialize_to_query_map(&values).expect_err("array should fail");

        // Assert
        match error {
            SerializeError::TopLevel(found) => assert_eq!(found, "array"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

mod value_serializer {
    use super::*;

    #[test]
    fn should_serialize_true_bool_when_serializing_root_value_then_return_string_true() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_bool(true)
            .expect("bool serialization should succeed");

        // Assert
        assert_eq!(result, Some(Value::String("true".into())));
    }

    #[test]
    fn should_serialize_bytes_when_bytes_are_utf8_then_return_string_value() {
        // Arrange
        let serializer = ValueSerializer::root();
        let bytes = b"cafe";

        // Act
        let result = serializer
            .serialize_bytes(bytes)
            .expect("bytes should serialize to string");

        // Assert
        assert_eq!(result, Some(Value::String("cafe".into())));
    }

    #[test]
    fn should_return_none_when_serializing_root_none_then_skip_value() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_none()
            .expect("serializing none should succeed");

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn should_error_for_newtype_variant_when_serializer_does_not_support_variant_then_return_unsupported_error() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let error = serializer
            .serialize_newtype_variant("Mode", 0, "Special", &42u8)
            .expect_err("newtype variant should be unsupported");

        // Assert
        match error {
            SerializeError::Unsupported(kind) => assert_eq!(kind, "newtype variant"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn should_preserve_none_in_sequence_when_option_contains_none_then_emit_empty_string() {
        // Arrange
        let mut seq = ValueSeqSerializer::new(Some(2));

        // Act
        SerializeSeq::serialize_element(&mut seq, &Some("alpha"))
            .expect("first element should serialize");
        SerializeSeq::serialize_element(&mut seq, &Option::<&str>::None)
            .expect("none element should serialize");
        let result = SerializeSeq::end(seq).expect("sequence should finish");

        // Assert
        let array = match result.expect("sequence should produce value") {
            Value::Array(items) => items,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(array.len(), 2);
        assert_eq!(array[0], Value::String("alpha".into()));
        assert_eq!(array[1], Value::String(String::new()));
    }

    #[test]
    fn should_serialize_signed_integer_when_value_is_negative_then_return_decimal_string() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_i64(-87)
            .expect("signed integer should serialize");

        // Assert
        assert_eq!(result, Some(Value::String("-87".into())));
    }

    #[test]
    fn should_serialize_unsigned_integer_when_value_is_positive_then_return_decimal_string() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_u64(144)
            .expect("unsigned integer should serialize");

        // Assert
        assert_eq!(result, Some(Value::String("144".into())));
    }

    #[test]
    fn should_serialize_float_when_value_is_negative_then_return_decimal_string() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_f64(-3.25)
            .expect("float should serialize");

        // Assert
        assert_eq!(result, Some(Value::String("-3.25".into())));
    }

    #[test]
    fn should_serialize_char_when_value_is_unicode_then_return_string() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_char('Ω')
            .expect("char should serialize");

        // Assert
        assert_eq!(result, Some(Value::String("Ω".into())));
    }

    #[test]
    fn should_delegate_some_to_inner_serializer_when_option_contains_value_then_return_serialized_value() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_some("cloud")
            .expect("some should serialize inner");

        // Assert
        assert_eq!(result, Some(Value::String("cloud".into())));
    }

    #[test]
    fn should_serialize_unit_struct_when_value_has_no_fields_then_return_empty_string() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_unit_struct("Heartbeat")
            .expect("unit struct should serialize");

        // Assert
        assert_eq!(result, Some(Value::String(String::new())));
    }

    #[test]
    fn should_serialize_unit_variant_when_variant_has_name_then_return_variant_name() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_unit_variant("Mode", 2, "Burst")
            .expect("unit variant should serialize");

        // Assert
        assert_eq!(result, Some(Value::String("Burst".into())));
    }

    #[test]
    fn should_serialize_newtype_struct_when_wrapper_contains_value_then_return_inner_string() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let result = serializer
            .serialize_newtype_struct("Wrapper", &123u16)
            .expect("newtype struct should serialize inner value");

        // Assert
        assert_eq!(result, Some(Value::String("123".into())));
    }

    #[test]
    fn should_collect_tuple_elements_when_tuple_has_two_entries_then_return_array_value() {
        // Arrange
        let mut tuple = Serializer::serialize_tuple(ValueSerializer::root(), 2)
            .expect("tuple serializer should be created");

        // Act
        SerializeTuple::serialize_element(&mut tuple, &"left")
            .expect("first element should serialize");
        SerializeTuple::serialize_element(&mut tuple, &"right")
            .expect("second element should serialize");
        let result = SerializeTuple::end(tuple).expect("tuple should finish");

        // Assert
        let array = match result.expect("tuple should produce value") {
            Value::Array(items) => items,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(array.len(), 2);
        assert_eq!(array[0], Value::String("left".into()));
        assert_eq!(array[1], Value::String("right".into()));
    }

    #[test]
    fn should_collect_map_entries_when_key_value_pairs_are_serialized_then_return_object_value() {
        // Arrange
        let mut map = Serializer::serialize_map(ValueSerializer::root(), Some(1))
            .expect("map serializer should be created");

        // Act
        SerializeMap::serialize_key(&mut map, "role").expect("key should serialize");
        SerializeMap::serialize_value(&mut map, &"admin").expect("value should serialize");
        let result = SerializeMap::end(map).expect("map should finish");

        // Assert
        let entries = match result.expect("map should produce value") {
            Value::Object(entries) => entries,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries.get("role"), Some(&Value::String("admin".into())));
    }

    #[test]
    fn should_error_for_tuple_variant_when_serializer_does_not_support_variant_then_return_unsupported_error() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let error = match serializer.serialize_tuple_variant("Mode", 1, "Burst", 2) {
            Err(err) => err,
            Ok(_) => panic!("tuple variant should be unsupported"),
        };

        // Assert
        match error {
            SerializeError::Unsupported(kind) => assert_eq!(kind, "tuple variant"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn should_error_for_struct_variant_when_serializer_does_not_support_variant_then_return_unsupported_error() {
        // Arrange
        let serializer = ValueSerializer::root();

        // Act
        let error = match serializer.serialize_struct_variant("Mode", 3, "Drift", 1) {
            Err(err) => err,
            Ok(_) => panic!("struct variant should be unsupported"),
        };

        // Assert
        match error {
            SerializeError::Unsupported(kind) => assert_eq!(kind, "struct variant"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
