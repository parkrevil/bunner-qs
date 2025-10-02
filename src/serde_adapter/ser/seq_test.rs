use super::*;
use crate::model::Value;
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
        // Arrange
        let mut serializer = ValueSeqSerializer::new(Some(3));

        // Act
        SerializeSeq::serialize_element(&mut serializer, &"alpha")
            .expect("first element should serialize");
        SerializeSeq::serialize_element(&mut serializer, &Option::<&str>::None)
            .expect("none element should serialize");
        SerializeSeq::serialize_element(&mut serializer, &"gamma")
            .expect("third element should serialize");
        let result = SerializeSeq::end(serializer).expect("sequence should finish");

        // Assert
        let array = match result.expect("sequence serializer should produce a value") {
            Value::Array(items) => items,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(array.len(), 3);
        assert_eq!(array[0], Value::String("alpha".into()));
        assert_eq!(array[1], Value::String(String::new()));
        assert_eq!(array[2], Value::String("gamma".into()));
    }

    #[test]
    fn should_preserve_structure_for_nested_arrays_when_nested_sequence_is_serialized_then_return_nested_arrays()
     {
        // Arrange
        let mut serializer = ValueSeqSerializer::new(None);
        let nested = vec!["one", "two"];

        // Act
        SerializeSeq::serialize_element(&mut serializer, &nested)
            .expect("nested sequence should serialize");
        let result = SerializeSeq::end(serializer).expect("sequence should finish");

        // Assert
        let array = match result.expect("sequence serializer should produce a value") {
            Value::Array(items) => items,
            other => panic!("unexpected value: {other:?}"),
        };
        let inner = match &array[0] {
            Value::Array(items) => items,
            other => panic!("expected nested array, got {other:?}"),
        };
        assert_eq!(inner.len(), 2);
        assert_eq!(inner[0], Value::String("one".into()));
        assert_eq!(inner[1], Value::String("two".into()));
    }

    #[test]
    fn should_return_array_value_from_tuple_serializer_when_tuple_elements_are_serialized_then_collect_into_array()
     {
        // Arrange
        let mut serializer = ValueSeqSerializer::new(Some(2));

        // Act
        SerializeTuple::serialize_element(&mut serializer, &"left")
            .expect("first tuple element should serialize");
        SerializeTuple::serialize_element(&mut serializer, &"right")
            .expect("second tuple element should serialize");
        let result = SerializeTuple::end(serializer).expect("tuple should finish");

        // Assert
        let items = match result.expect("tuple serializer should produce a value") {
            Value::Array(items) => items,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(
            items,
            vec![Value::String("left".into()), Value::String("right".into())]
        );
    }

    #[test]
    fn should_return_array_value_from_tuple_struct_serializer_when_tuple_struct_is_serialized_then_collect_fields()
     {
        // Arrange
        let mut serializer = ValueSeqSerializer::new(Some(1));

        // Act
        SerializeTupleStruct::serialize_field(&mut serializer, &123_i32)
            .expect("tuple struct field should serialize");
        let result = SerializeTupleStruct::end(serializer).expect("tuple struct should finish");

        // Assert
        let items = match result.expect("tuple struct serializer should produce a value") {
            Value::Array(items) => items,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(items, vec![Value::String("123".into())]);
    }

    #[test]
    fn should_report_unsupported_message_when_tuple_variant_field_is_serialized_then_return_error()
    {
        // Arrange
        let mut serializer = ValueSeqSerializer::new(None);

        // Act
        let error = SerializeTupleVariant::serialize_field(&mut serializer, &123i32)
            .expect_err("tuple variant field should be unsupported");

        // Assert
        assert!(error.to_string().contains("tuple variants are unsupported"));
    }

    #[test]
    fn should_report_unsupported_message_when_ending_tuple_variant_then_return_unsupported_error() {
        // Arrange
        let serializer = ValueSeqSerializer::new(None);

        // Act
        let error =
            SerializeTupleVariant::end(serializer).expect_err("tuple variant end should fail");

        // Assert
        assert_eq!(
            error.to_string(),
            "tuple variants are unsupported in query string serialization"
        );
    }

    #[test]
    fn should_convert_none_sequence_element_into_empty_string_when_pushed_directly() {
        // Arrange
        let mut serializer = ValueSeqSerializer::new(Some(1));

        // Act
        serializer.push_value(None);
        let result = SerializeSeq::end(serializer).expect("sequence end should succeed");

        // Assert
        let array = match result.expect("sequence serializer should produce value") {
            Value::Array(items) => items,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(array, vec![Value::String(String::new())]);
    }

    #[test]
    fn should_propagate_error_when_sequence_element_serialization_fails_then_return_message() {
        // Arrange
        let mut serializer = ValueSeqSerializer::new(None);

        // Act
        let error = SerializeSeq::serialize_element(&mut serializer, &FailingElement)
            .expect_err("failing element should error");

        // Assert
        match error {
            SerializeError::Message(message) => {
                assert_eq!(message, "element serialization failed")
            }
            other => panic!("unexpected error type: {other:?}"),
        }
    }

    #[test]
    fn should_push_empty_string_when_value_is_none_then_append_placeholder_element() {
        // Arrange
        let mut serializer = ValueSeqSerializer::new(Some(1));

        // Act
        serializer.push_value(None);
        let result =
            SerializeSeq::end(serializer).expect("sequence should finish after manual push");

        // Assert
        let array = match result.expect("sequence serializer should produce a value") {
            Value::Array(items) => items,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(array, vec![Value::String(String::new())]);
    }
}
