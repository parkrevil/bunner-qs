use super::*;
use crate::model::Value;
use assert_matches::assert_matches;
use serde::ser::{SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

mod value_seq_serializer {
    use super::*;
    use serde::Serialize;
    use serde::ser::{Error as _, SerializeSeq};

    #[derive(Debug)]
    struct FailingElement;

    impl Serialize for FailingElement {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(S::Error::custom("element serialization failed"))
        }
    }

    #[test]
    fn should_convert_none_to_empty_string_when_option_sequence_contains_none_then_emit_empty_string()
     {
        let mut serializer = ValueSeqSerializer::new(Some(3));

        SerializeSeq::serialize_element(&mut serializer, &"alpha")
            .expect("first element should serialize");
        SerializeSeq::serialize_element(&mut serializer, &Option::<&str>::None)
            .expect("none element should serialize");
        SerializeSeq::serialize_element(&mut serializer, &"gamma")
            .expect("third element should serialize");
        let result = SerializeSeq::end(serializer).expect("sequence should finish");

        let value = result.expect("sequence serializer should produce a value");
        let array = value
            .as_array()
            .expect("sequence serializer should produce an array");
        assert_eq!(array.len(), 3);
        assert_matches!(array.first(), Some(Value::String(text)) if text == "alpha");
        assert_matches!(array.get(1), Some(Value::String(text)) if text.is_empty());
        assert_matches!(array.get(2), Some(Value::String(text)) if text == "gamma");
    }

    #[test]
    fn should_preserve_structure_for_nested_arrays_when_nested_sequence_is_serialized_then_return_nested_arrays()
     {
        let mut serializer = ValueSeqSerializer::new(None);
        let nested = vec!["one", "two"];

        SerializeSeq::serialize_element(&mut serializer, &nested)
            .expect("nested sequence should serialize");
        let result = SerializeSeq::end(serializer).expect("sequence should finish");

        let value = result.expect("sequence serializer should produce a value");
        let array = value
            .as_array()
            .expect("sequence serializer should produce an array");
        let inner = array
            .first()
            .expect("sequence should contain nested array")
            .as_array()
            .expect("nested value should be array");
        assert_eq!(inner.len(), 2);
        assert_matches!(inner.first(), Some(Value::String(text)) if text == "one");
        assert_matches!(inner.get(1), Some(Value::String(text)) if text == "two");
    }

    #[test]
    fn should_return_array_value_from_tuple_serializer_when_tuple_elements_are_serialized_then_collect_into_array()
     {
        let mut serializer = ValueSeqSerializer::new(Some(2));

        SerializeTuple::serialize_element(&mut serializer, &"left")
            .expect("first tuple element should serialize");
        SerializeTuple::serialize_element(&mut serializer, &"right")
            .expect("second tuple element should serialize");
        let result = SerializeTuple::end(serializer).expect("tuple should finish");

        let value = result.expect("tuple serializer should produce a value");
        let array = value
            .as_array()
            .expect("tuple serializer should produce an array");
        assert_eq!(array.len(), 2);
        assert_matches!(array.first(), Some(Value::String(text)) if text == "left");
        assert_matches!(array.get(1), Some(Value::String(text)) if text == "right");
    }

    #[test]
    fn should_return_array_value_from_tuple_struct_serializer_when_tuple_struct_is_serialized_then_collect_fields()
     {
        let mut serializer = ValueSeqSerializer::new(Some(1));

        SerializeTupleStruct::serialize_field(&mut serializer, &123_i32)
            .expect("tuple struct field should serialize");
        let result = SerializeTupleStruct::end(serializer).expect("tuple struct should finish");

        let value = result.expect("tuple struct serializer should produce a value");
        let array = value
            .as_array()
            .expect("tuple struct serializer should produce an array");
        assert_eq!(array.len(), 1);
        assert_matches!(array.first(), Some(Value::String(text)) if text == "123");
    }

    #[test]
    fn should_report_unsupported_message_when_tuple_variant_field_is_serialized_then_return_error()
    {
        let mut serializer = ValueSeqSerializer::new(None);

        let error = SerializeTupleVariant::serialize_field(&mut serializer, &123i32)
            .expect_err("tuple variant field should be unsupported");

        assert!(error.to_string().contains("tuple variants are unsupported"));
    }

    #[test]
    fn should_report_unsupported_message_when_ending_tuple_variant_then_return_unsupported_error() {
        let serializer = ValueSeqSerializer::new(None);

        let error =
            SerializeTupleVariant::end(serializer).expect_err("tuple variant end should fail");

        assert_eq!(
            error.to_string(),
            "tuple variants are unsupported in query string serialization"
        );
    }

    #[test]
    fn should_convert_none_sequence_element_into_empty_string_when_pushed_directly_then_emit_placeholder_string()
     {
        let mut serializer = ValueSeqSerializer::new(Some(1));

        serializer.push_value(None);
        let result = SerializeSeq::end(serializer).expect("sequence end should succeed");

        let value = result.expect("sequence serializer should produce value");
        let array = value
            .as_array()
            .expect("sequence serializer should produce an array");
        assert_eq!(array.len(), 1);
        assert_matches!(array.first(), Some(Value::String(text)) if text.is_empty());
    }

    #[test]
    fn should_propagate_error_when_sequence_element_serialization_fails_then_return_message() {
        let mut serializer = ValueSeqSerializer::new(None);

        let error = SerializeSeq::serialize_element(&mut serializer, &FailingElement)
            .expect_err("failing element should error");

        assert_matches!(
            error,
            SerializeError::Message(ref message) if message == "element serialization failed",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_push_empty_string_when_value_is_none_then_append_placeholder_element() {
        let mut serializer = ValueSeqSerializer::new(Some(1));

        serializer.push_value(None);
        let result =
            SerializeSeq::end(serializer).expect("sequence should finish after manual push");

        let value = result.expect("sequence serializer should produce a value");
        let array = value
            .as_array()
            .expect("sequence serializer should produce an array");
        assert_eq!(array.len(), 1);
        assert_matches!(array.first(), Some(Value::String(text)) if text.is_empty());
    }
}
