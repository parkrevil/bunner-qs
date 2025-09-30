use super::*;
use crate::model::Value;
use serde::ser::SerializeStruct;

mod value_struct_serializer {
    use super::*;

    #[test]
    fn collects_multiple_fields_into_object() {
        // Arrange
        let mut serializer = ValueStructSerializer::new();

        // Act
        SerializeStruct::serialize_field(&mut serializer, "name", &"Aria")
            .expect("serializing string field should succeed");
        SerializeStruct::serialize_field(&mut serializer, "age", &27u8)
            .expect("serializing numeric field should succeed");
        let result = SerializeStruct::end(serializer).expect("ending serializer should succeed");

        // Assert
        let map = match result.expect("struct serializer should produce a value") {
            Value::Object(map) => map,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("name"), Some(&Value::String("Aria".into())));
        assert_eq!(map.get("age"), Some(&Value::String("27".into())));
    }

    #[test]
    fn skips_entry_when_field_serializes_to_none() {
        // Arrange
        let mut serializer = ValueStructSerializer::new();

        // Act
        SerializeStruct::serialize_field(&mut serializer, "nickname", &Option::<String>::None)
            .expect("serializing none should succeed");
        let result = SerializeStruct::end(serializer).expect("ending serializer should succeed");

        // Assert
        let map = match result.expect("struct serializer should produce a value") {
            Value::Object(map) => map,
            other => panic!("unexpected value: {other:?}"),
        };
        assert!(map.is_empty());
    }

    #[test]
    fn stores_array_when_field_serializes_to_sequence() {
        // Arrange
        let mut serializer = ValueStructSerializer::new();

        // Act
        SerializeStruct::serialize_field(&mut serializer, "skills", &vec!["drums", "guitar"])
            .expect("serializing sequence should succeed");
        let result = SerializeStruct::end(serializer).expect("ending serializer should succeed");

        // Assert
        let map = match result.expect("struct serializer should produce a value") {
            Value::Object(map) => map,
            other => panic!("unexpected value: {other:?}"),
        };
        let array = match map.get("skills") {
            Some(Value::Array(items)) => items,
            other => panic!("expected array value, got {other:?}"),
        };
        assert_eq!(array.len(), 2);
        assert_eq!(array[0], Value::String("drums".into()));
        assert_eq!(array[1], Value::String("guitar".into()));
    }
}

mod map_key_serializer {
    use super::*;
    use serde::ser::Serializer;

    #[test]
    fn serializes_string_key_preserving_text() {
        // Arrange

        // Act
        let result = MapKeySerializer
            .serialize_str("alpha")
            .expect("string key should serialize");

        // Assert
        assert_eq!(result, "alpha");
    }

    #[test]
    fn serializes_boolean_key_as_literal() {
        // Arrange

        // Act
        let result = MapKeySerializer
            .serialize_bool(true)
            .expect("bool key should serialize");

        // Assert
        assert_eq!(result, "true");
    }

    #[test]
    fn serializes_signed_numbers_as_decimal_strings() {
        // Arrange

        // Act
        let i8_value = MapKeySerializer
            .serialize_i8(-3)
            .expect("i8 should serialize");
        let i16_value = MapKeySerializer
            .serialize_i16(-4)
            .expect("i16 should serialize");
        let i32_value = MapKeySerializer
            .serialize_i32(-5)
            .expect("i32 should serialize");
        let i64_value = MapKeySerializer
            .serialize_i64(-6)
            .expect("i64 should serialize");
        let i128_value = MapKeySerializer
            .serialize_i128(-7)
            .expect("i128 should serialize");

        // Assert
        assert_eq!(i8_value, "-3");
        assert_eq!(i16_value, "-4");
        assert_eq!(i32_value, "-5");
        assert_eq!(i64_value, "-6");
        assert_eq!(i128_value, "-7");
    }

    #[test]
    fn serializes_unsigned_numbers_as_decimal_strings() {
        // Arrange

        // Act
        let u8_value = MapKeySerializer
            .serialize_u8(3)
            .expect("u8 should serialize");
        let u16_value = MapKeySerializer
            .serialize_u16(4)
            .expect("u16 should serialize");
        let u32_value = MapKeySerializer
            .serialize_u32(5)
            .expect("u32 should serialize");
        let u64_value = MapKeySerializer
            .serialize_u64(6)
            .expect("u64 should serialize");
        let u128_value = MapKeySerializer
            .serialize_u128(7)
            .expect("u128 should serialize");

        // Assert
        assert_eq!(u8_value, "3");
        assert_eq!(u16_value, "4");
        assert_eq!(u32_value, "5");
        assert_eq!(u64_value, "6");
        assert_eq!(u128_value, "7");
    }

    #[test]
    fn serializes_floats_preserving_precision() {
        // Arrange

        // Act
        let f32_value = MapKeySerializer
            .serialize_f32(1.5)
            .expect("f32 should serialize");
        let f64_value = MapKeySerializer
            .serialize_f64(-2.25)
            .expect("f64 should serialize");

        // Assert
        assert_eq!(f32_value, "1.5");
        assert_eq!(f64_value, "-2.25");
    }

    #[test]
    fn serializes_char_as_single_character_string() {
        // Arrange

        // Act
        let result = MapKeySerializer
            .serialize_char('ß')
            .expect("char should serialize");

        // Assert
        assert_eq!(result, "ß");
    }

    #[test]
    fn serializes_bytes_using_utf8_lossy() {
        // Arrange
        let bytes = b"caf\xC3\xA9";

        // Act
        let result = MapKeySerializer
            .serialize_bytes(bytes)
            .expect("bytes should serialize");

        // Assert
        assert_eq!(result, "café");
    }

    #[test]
    fn returns_invalid_key_for_unit() {
        // Arrange

        // Act
        let error = MapKeySerializer
            .serialize_unit()
            .expect_err("unit should be rejected");

        // Assert
        match error {
            SerializeError::InvalidKey(message) => assert!(message.contains("unit")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn returns_invalid_key_for_unit_struct() {
        // Arrange

        // Act
        let error = MapKeySerializer
            .serialize_unit_struct("Marker")
            .expect_err("unit struct should be rejected");

        // Assert
        match error {
            SerializeError::InvalidKey(message) => {
                assert!(message.contains("unit struct `Marker`"))
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn serializes_unit_variant_as_name() {
        // Arrange

        // Act
        let result = MapKeySerializer
            .serialize_unit_variant("Flavor", 1, "Vanilla")
            .expect("unit variant should serialize");

        // Assert
        assert_eq!(result, "Vanilla");
    }

    #[test]
    fn serializes_newtype_struct_via_inner_value() {
        // Arrange

        // Act
        let result =
            serde::ser::Serializer::serialize_newtype_struct(MapKeySerializer, "Wrapper", &42u8)
                .expect("newtype struct should serialize inner");

        // Assert
        assert_eq!(result, "42");
    }

    #[test]
    fn returns_invalid_key_for_none() {
        // Arrange

        // Act
        let error = MapKeySerializer
            .serialize_none()
            .expect_err("none should be rejected");

        // Assert
        match error {
            SerializeError::InvalidKey(message) => assert!(message.contains("option")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn serializes_inner_value_for_some() {
        // Arrange

        // Act
        let result = serde::ser::Serializer::serialize_some(MapKeySerializer, &123u16)
            .expect("some should serialize inner value");

        // Assert
        assert_eq!(result, "123");
    }

    #[test]
    fn errors_with_variant_details_for_newtype_variant() {
        // Arrange

        // Act
        let error = MapKeySerializer
            .serialize_newtype_variant("Flavor", 2, "Mint", &"cool")
            .expect_err("newtype variant should be rejected");

        // Assert
        match error {
            SerializeError::InvalidKey(message) => {
                assert!(message.contains("enum `Flavor`"));
                assert!(message.contains("variant `Mint`"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn errors_with_length_details_for_sequence() {
        // Arrange

        // Act
        let error = match MapKeySerializer.serialize_seq(Some(3)) {
            Err(err) => err,
            Ok(_) => panic!("sequence should be rejected"),
        };

        // Assert
        match error {
            SerializeError::InvalidKey(message) => assert!(message.contains("sequence")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn errors_with_length_details_for_tuple() {
        // Arrange

        // Act
        let error = match MapKeySerializer.serialize_tuple(2) {
            Err(err) => err,
            Ok(_) => panic!("tuple should be rejected"),
        };

        // Assert
        match error {
            SerializeError::InvalidKey(message) => assert!(message.contains("tuple")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn errors_with_name_and_length_for_tuple_struct() {
        // Arrange

        // Act
        let error = match MapKeySerializer.serialize_tuple_struct("Point", 2) {
            Err(err) => err,
            Ok(_) => panic!("tuple struct should be rejected"),
        };

        // Assert
        match error {
            SerializeError::InvalidKey(message) => {
                assert!(message.contains("tuple struct `Point`"));
                assert!(message.contains("length 2"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn errors_with_variant_details_for_tuple_variant() {
        // Arrange

        // Act
        let error = match MapKeySerializer.serialize_tuple_variant("Flavor", 3, "Cherry", 2) {
            Err(err) => err,
            Ok(_) => panic!("tuple variant should be rejected"),
        };

        // Assert
        match error {
            SerializeError::InvalidKey(message) => {
                assert!(message.contains("enum `Flavor`"));
                assert!(message.contains("variant `Cherry`"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn errors_with_length_hint_for_map() {
        // Arrange

        // Act
        let error = match MapKeySerializer.serialize_map(Some(1)) {
            Err(err) => err,
            Ok(_) => panic!("map should be rejected"),
        };

        // Assert
        match error {
            SerializeError::InvalidKey(message) => assert!(message.contains("map")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn errors_with_field_count_for_struct() {
        // Arrange

        // Act
        let error = match MapKeySerializer.serialize_struct("Profile", 3) {
            Err(err) => err,
            Ok(_) => panic!("struct should be rejected"),
        };

        // Assert
        match error {
            SerializeError::InvalidKey(message) => assert!(message.contains("struct `Profile`")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn errors_with_variant_details_for_struct_variant() {
        // Arrange

        // Act
        let error = match MapKeySerializer.serialize_struct_variant("Flavor", 4, "Mocha", 2) {
            Err(err) => err,
            Ok(_) => panic!("struct variant should be rejected"),
        };

        // Assert
        match error {
            SerializeError::InvalidKey(message) => {
                assert!(message.contains("enum `Flavor`"));
                assert!(message.contains("variant `Mocha`"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
