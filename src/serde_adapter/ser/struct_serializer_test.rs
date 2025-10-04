use super::*;
use crate::model::Value;
use assert_matches::assert_matches;
use serde::ser::SerializeStruct;

mod value_struct_serializer {
    use super::*;

    #[test]
    fn should_collect_multiple_fields_into_object_when_struct_has_multiple_fields_then_store_field_values()
     {
        let mut serializer = ValueStructSerializer::new();

        SerializeStruct::serialize_field(&mut serializer, "name", &"Aria")
            .expect("serializing string field should succeed");
        SerializeStruct::serialize_field(&mut serializer, "age", &27u8)
            .expect("serializing numeric field should succeed");
        let result = SerializeStruct::end(serializer).expect("ending serializer should succeed");

        let value = result.expect("struct serializer should produce a value");
        let map = value.as_object().expect("value should be object");
        assert_eq!(map.len(), 2);
        assert_matches!(map.get("name"), Some(Value::String(text)) if text == "Aria");
        assert_matches!(map.get("age"), Some(Value::String(text)) if text == "27");
    }

    #[test]
    fn should_skip_entry_when_field_serializes_to_none_then_produce_empty_map() {
        let mut serializer = ValueStructSerializer::new();

        SerializeStruct::serialize_field(&mut serializer, "nickname", &Option::<String>::None)
            .expect("serializing none should succeed");
        let result = SerializeStruct::end(serializer).expect("ending serializer should succeed");

        let value = result.expect("struct serializer should produce a value");
        let map = value.as_object().expect("value should be object");
        assert!(map.is_empty());
    }

    #[test]
    fn should_store_array_when_field_serializes_to_sequence_then_store_array_items() {
        let mut serializer = ValueStructSerializer::new();

        SerializeStruct::serialize_field(&mut serializer, "skills", &vec!["drums", "guitar"])
            .expect("serializing sequence should succeed");
        let result = SerializeStruct::end(serializer).expect("ending serializer should succeed");

        let value = result.expect("struct serializer should produce a value");
        let map = value.as_object().expect("value should be object");
        let skills = map.get("skills").expect("skills field should exist");
        let array = skills.as_array().expect("skills should be array");
        assert_eq!(array.len(), 2);
        assert_matches!(array.first(), Some(Value::String(text)) if text == "drums");
        assert_matches!(array.get(1), Some(Value::String(text)) if text == "guitar");
    }
}

mod map_key_serializer {
    use super::*;
    use serde::ser::Serializer;

    #[test]
    fn should_serialize_string_key_when_key_is_str_then_preserve_text() {
        let result = MapKeySerializer
            .serialize_str("alpha")
            .expect("string key should serialize");

        assert_eq!(result, "alpha");
    }

    #[test]
    fn should_serialize_boolean_key_when_key_is_bool_then_return_literal_string() {
        let result = MapKeySerializer
            .serialize_bool(true)
            .expect("bool key should serialize");

        assert_eq!(result, "true");
    }

    #[test]
    fn should_serialize_signed_numbers_when_key_is_integer_then_return_decimal_strings() {
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

        assert_eq!(i8_value, "-3");
        assert_eq!(i16_value, "-4");
        assert_eq!(i32_value, "-5");
        assert_eq!(i64_value, "-6");
        assert_eq!(i128_value, "-7");
    }

    #[test]
    fn should_serialize_unsigned_numbers_when_key_is_unsigned_integer_then_return_decimal_strings()
    {
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

        assert_eq!(u8_value, "3");
        assert_eq!(u16_value, "4");
        assert_eq!(u32_value, "5");
        assert_eq!(u64_value, "6");
        assert_eq!(u128_value, "7");
    }

    #[test]
    fn should_serialize_floats_when_key_is_float_then_preserve_precision() {
        let f32_value = MapKeySerializer
            .serialize_f32(1.5)
            .expect("f32 should serialize");
        let f64_value = MapKeySerializer
            .serialize_f64(-2.25)
            .expect("f64 should serialize");

        assert_eq!(f32_value, "1.5");
        assert_eq!(f64_value, "-2.25");
    }

    #[test]
    fn should_serialize_char_when_key_is_char_then_return_single_character_string() {
        let result = MapKeySerializer
            .serialize_char('ß')
            .expect("char should serialize");

        assert_eq!(result, "ß");
    }

    #[test]
    fn should_serialize_bytes_when_key_is_bytes_then_return_utf8_lossy_string() {
        let bytes = b"caf\xC3\xA9";

        let result = MapKeySerializer
            .serialize_bytes(bytes)
            .expect("bytes should serialize");

        assert_eq!(result, "café");
    }

    #[test]
    fn should_return_invalid_key_when_key_is_unit_then_return_invalid_key_error() {
        let error = MapKeySerializer
            .serialize_unit()
            .expect_err("unit should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message) if message.contains("unit"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_return_invalid_key_when_key_is_unit_struct_then_return_invalid_key_error() {
        let error = MapKeySerializer
            .serialize_unit_struct("Marker")
            .expect_err("unit struct should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message) if message.contains("unit struct `Marker`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_serialize_unit_variant_when_key_is_unit_variant_then_return_variant_name() {
        let result = MapKeySerializer
            .serialize_unit_variant("Flavor", 1, "Vanilla")
            .expect("unit variant should serialize");

        assert_eq!(result, "Vanilla");
    }

    #[test]
    fn should_serialize_newtype_struct_when_key_is_newtype_struct_then_return_inner_value() {
        let result =
            serde::ser::Serializer::serialize_newtype_struct(MapKeySerializer, "Wrapper", &42u8)
                .expect("newtype struct should serialize inner");

        assert_eq!(result, "42");
    }

    #[test]
    fn should_return_invalid_key_when_key_is_none_then_return_invalid_key_error() {
        let error = MapKeySerializer
            .serialize_none()
            .expect_err("none should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message) if message.contains("option"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_serialize_inner_value_when_key_is_some_then_return_inner_value_string() {
        let result = serde::ser::Serializer::serialize_some(MapKeySerializer, &123u16)
            .expect("some should serialize inner value");

        assert_eq!(result, "123");
    }

    #[test]
    fn should_error_with_variant_details_when_key_is_newtype_variant_then_include_enum_and_variant()
    {
        let error = MapKeySerializer
            .serialize_newtype_variant("Flavor", 2, "Mint", &"cool")
            .expect_err("newtype variant should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message)
                if message.contains("enum `Flavor`") && message.contains("variant `Mint`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_error_with_length_details_when_key_is_sequence_then_return_invalid_key_error() {
        let error = MapKeySerializer
            .serialize_seq(Some(3))
            .err()
            .expect("sequence should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message) if message.contains("sequence"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_error_with_length_details_when_key_is_tuple_then_return_invalid_key_error() {
        let error = MapKeySerializer
            .serialize_tuple(2)
            .err()
            .expect("tuple should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message) if message.contains("tuple"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_error_with_name_and_length_when_key_is_tuple_struct_then_return_invalid_key_error() {
        let error = MapKeySerializer
            .serialize_tuple_struct("Point", 2)
            .err()
            .expect("tuple struct should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message)
                if message.contains("tuple struct `Point`") && message.contains("length 2"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_error_with_variant_details_when_key_is_tuple_variant_then_return_invalid_key_error() {
        let error = MapKeySerializer
            .serialize_tuple_variant("Flavor", 3, "Cherry", 2)
            .err()
            .expect("tuple variant should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message)
                if message.contains("enum `Flavor`") && message.contains("variant `Cherry`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_error_with_length_hint_when_key_is_map_then_return_invalid_key_error() {
        let error = MapKeySerializer
            .serialize_map(Some(1))
            .err()
            .expect("map should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message) if message.contains("map"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_error_with_field_count_when_key_is_struct_then_return_invalid_key_error() {
        let error = MapKeySerializer
            .serialize_struct("Profile", 3)
            .err()
            .expect("struct should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message) if message.contains("struct `Profile`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn should_error_with_variant_details_when_key_is_struct_variant_then_return_invalid_key_error()
    {
        let error = MapKeySerializer
            .serialize_struct_variant("Flavor", 4, "Mocha", 2)
            .err()
            .expect("struct variant should be rejected");

        assert_matches!(
            &error,
            SerializeError::InvalidKey(message)
                if message.contains("enum `Flavor`") && message.contains("variant `Mocha`"),
            "unexpected error: {error}"
        );
    }
}
