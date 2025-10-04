use super::*;
use crate::model::Value;
use assert_matches::assert_matches;
use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, SerializeTuple, SerializeTupleStruct, Serializer};

mod serialize_to_query_map {
    use super::*;

    #[derive(Serialize)]
    struct Profile<'a> {
        name: &'a str,
        age: u32,
    }

    #[derive(Serialize)]
    enum Command<'a> {
        Invoke(&'a str, &'a str),
    }

    #[test]
    fn should_serialize_struct_into_ordered_map_when_struct_has_simple_fields_then_store_field_strings()
     {
        let profile = Profile {
            name: "Mina",
            age: 23,
        };

        let map = serialize_to_query_map(&profile).expect("struct should serialize into a map");

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("name"), Some(&Value::String("Mina".into())));
        assert_eq!(map.get("age"), Some(&Value::String("23".into())));
    }

    #[test]
    fn should_reject_string_top_level_when_serializing_query_map_then_return_top_level_error() {
        let value = "hello";

        let error = serialize_to_query_map(&value).expect_err("non-object top level should fail");

        assert_matches!(
            &error,
            SerializeError::TopLevel(found) if found == "string",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_report_unexpected_skip_when_top_level_option_is_none_then_return_unexpected_skip_error()
     {
        let value = Option::<String>::None;

        let error = serialize_to_query_map(&value).expect_err("none should yield unexpected skip");

        assert_matches!(
            error,
            SerializeError::UnexpectedSkip,
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_reject_array_top_level_when_serializing_query_map_then_return_top_level_error() {
        let values = vec!["hello", "world"];

        let error = serialize_to_query_map(&values).expect_err("array should fail");

        assert_matches!(
            &error,
            SerializeError::TopLevel(found) if found == "array",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_propagate_unsupported_error_when_enum_contains_tuple_variant_then_return_unsupported_message()
     {
        let command = Command::Invoke("ping", "now");

        let error =
            serialize_to_query_map(&command).expect_err("tuple variant should be unsupported");

        assert_matches!(
            &error,
            SerializeError::Unsupported(kind) if *kind == "tuple variant",
            "unexpected error: {error:?}"
        );
    }
}

mod value_serializer {
    use super::*;

    #[test]
    fn should_serialize_true_bool_when_serializing_root_value_then_return_string_true() {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_bool(true)
            .expect("bool serialization should succeed");

        assert_eq!(result, Some(Value::String("true".into())));
    }

    #[test]
    fn should_serialize_bytes_when_bytes_are_utf8_then_return_string_value() {
        let serializer = ValueSerializer::root();
        let bytes = b"cafe";

        let result = serializer
            .serialize_bytes(bytes)
            .expect("bytes should serialize to string");

        assert_eq!(result, Some(Value::String("cafe".into())));
    }

    #[test]
    fn should_return_none_when_serializing_root_none_then_skip_value() {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_none()
            .expect("serializing none should succeed");

        assert!(result.is_none());
    }

    #[test]
    fn should_error_for_newtype_variant_when_serializer_does_not_support_variant_then_return_unsupported_error()
     {
        let serializer = ValueSerializer::root();

        let error = serializer
            .serialize_newtype_variant("Mode", 0, "Special", &42u8)
            .expect_err("newtype variant should be unsupported");

        assert_matches!(
            &error,
            SerializeError::Unsupported(kind) if *kind == "newtype variant",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_preserve_none_in_sequence_when_option_contains_none_then_emit_empty_string() {
        let mut seq = ValueSeqSerializer::new(Some(2));

        SerializeSeq::serialize_element(&mut seq, &Some("alpha"))
            .expect("first element should serialize");
        SerializeSeq::serialize_element(&mut seq, &Option::<&str>::None)
            .expect("none element should serialize");
        let result = SerializeSeq::end(seq).expect("sequence should finish");

        let value = result.expect("sequence should produce value");
        let array = value.as_array().expect("value should be array");
        assert_eq!(array.len(), 2);
        assert_matches!(array.first(), Some(Value::String(text)) if text == "alpha");
        assert_matches!(array.get(1), Some(Value::String(text)) if text.is_empty());
    }

    #[test]
    fn should_serialize_signed_integer_when_value_is_negative_then_return_decimal_string() {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_i64(-87)
            .expect("signed integer should serialize");

        assert_eq!(result, Some(Value::String("-87".into())));
    }

    #[test]
    fn should_serialize_unsigned_integer_when_value_is_positive_then_return_decimal_string() {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_u64(144)
            .expect("unsigned integer should serialize");

        assert_eq!(result, Some(Value::String("144".into())));
    }

    #[test]
    fn should_serialize_float_when_value_is_negative_then_return_decimal_string() {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_f64(-3.25)
            .expect("float should serialize");

        assert_eq!(result, Some(Value::String("-3.25".into())));
    }

    #[test]
    fn should_serialize_char_when_value_is_unicode_then_return_string() {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_char('Ω')
            .expect("char should serialize");

        assert_eq!(result, Some(Value::String("Ω".into())));
    }

    #[test]
    fn should_delegate_some_to_inner_serializer_when_option_contains_value_then_return_serialized_value()
     {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_some("cloud")
            .expect("some should serialize inner");

        assert_eq!(result, Some(Value::String("cloud".into())));
    }

    #[test]
    fn should_serialize_unit_struct_when_value_has_no_fields_then_return_empty_string() {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_unit_struct("Heartbeat")
            .expect("unit struct should serialize");

        assert_eq!(result, Some(Value::String(String::new())));
    }

    #[test]
    fn should_serialize_unit_variant_when_variant_has_name_then_return_variant_name() {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_unit_variant("Mode", 2, "Burst")
            .expect("unit variant should serialize");

        assert_eq!(result, Some(Value::String("Burst".into())));
    }

    #[test]
    fn should_serialize_newtype_struct_when_wrapper_contains_value_then_return_inner_string() {
        let serializer = ValueSerializer::root();

        let result = serializer
            .serialize_newtype_struct("Wrapper", &123u16)
            .expect("newtype struct should serialize inner value");

        assert_eq!(result, Some(Value::String("123".into())));
    }

    #[test]
    fn should_preserve_none_as_empty_string_when_sequence_serializer_handles_none() {
        let serializer = ValueSerializer::sequence_element();

        let result = serializer
            .serialize_none()
            .expect("sequence serializer should allow none");

        assert_eq!(result, Some(Value::String(String::new())));
    }

    #[test]
    fn should_collect_tuple_elements_when_tuple_has_two_entries_then_return_array_value() {
        let mut tuple = Serializer::serialize_tuple(ValueSerializer::root(), 2)
            .expect("tuple serializer should be created");

        SerializeTuple::serialize_element(&mut tuple, &"left")
            .expect("first element should serialize");
        SerializeTuple::serialize_element(&mut tuple, &"right")
            .expect("second element should serialize");
        let result = SerializeTuple::end(tuple).expect("tuple should finish");

        let value = result.expect("tuple should produce value");
        let array = value.as_array().expect("value should be array");
        assert_eq!(array.len(), 2);
        assert_matches!(array.first(), Some(Value::String(text)) if text == "left");
        assert_matches!(array.get(1), Some(Value::String(text)) if text == "right");
    }

    #[test]
    fn should_collect_tuple_struct_fields_when_serializing_tuple_struct_then_return_array_value() {
        let mut serializer = Serializer::serialize_tuple_struct(ValueSerializer::root(), "Pair", 2)
            .expect("tuple struct serializer should be created");

        SerializeTupleStruct::serialize_field(&mut serializer, &"first")
            .expect("first field should serialize");
        SerializeTupleStruct::serialize_field(&mut serializer, &"second")
            .expect("second field should serialize");
        let result = SerializeTupleStruct::end(serializer).expect("tuple struct should finish");

        let value = result.expect("tuple struct should produce value");
        let array = value.as_array().expect("value should be array");
        assert_eq!(array.len(), 2);
        assert_matches!(array.first(), Some(Value::String(text)) if text == "first");
        assert_matches!(array.get(1), Some(Value::String(text)) if text == "second");
    }

    #[test]
    fn should_collect_map_entries_when_key_value_pairs_are_serialized_then_return_object_value() {
        let mut map = Serializer::serialize_map(ValueSerializer::root(), Some(1))
            .expect("map serializer should be created");

        SerializeMap::serialize_key(&mut map, "role").expect("key should serialize");
        SerializeMap::serialize_value(&mut map, &"admin").expect("value should serialize");
        let result = SerializeMap::end(map).expect("map should finish");

        let value = result.expect("map should produce value");
        let entries = value.as_object().expect("value should be object");
        assert_eq!(entries.len(), 1);
        assert_matches!(entries.get("role"), Some(Value::String(text)) if text == "admin");
    }

    #[test]
    fn should_error_for_tuple_variant_when_serializer_does_not_support_variant_then_return_unsupported_error()
     {
        let serializer = ValueSerializer::root();

        let error = serializer
            .serialize_tuple_variant("Mode", 1, "Burst", 2)
            .err()
            .expect("tuple variant should be unsupported");

        assert_matches!(
            &error,
            SerializeError::Unsupported(kind) if *kind == "tuple variant",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_error_for_struct_variant_when_serializer_does_not_support_variant_then_return_unsupported_error()
     {
        let serializer = ValueSerializer::root();

        let error = serializer
            .serialize_struct_variant("Mode", 3, "Drift", 1)
            .err()
            .expect("struct variant should be unsupported");

        assert_matches!(
            &error,
            SerializeError::Unsupported(kind) if *kind == "struct variant",
            "unexpected error: {error:?}"
        );
    }
}

mod describe_value_fn {
    use super::*;

    #[test]
    fn should_describe_object_value_when_value_is_object() {
        let mut map = OrderedMap::default();
        map.insert("id".into(), Value::String("42".into()));

        let description = describe_value(&Value::Object(map));

        assert_eq!(description, "object");
    }
}
