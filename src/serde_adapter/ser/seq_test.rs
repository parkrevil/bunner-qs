use super::*;
use crate::model::Value;
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

mod value_seq_serializer {
    use super::*;

    #[test]
    fn when_serializing_mixed_option_sequence_it_should_convert_none_to_empty_string() {
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
    fn when_serializing_nested_arrays_it_should_preserve_structure() {
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
    fn when_tuple_serializer_collects_elements_it_should_return_array_value() {
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
        assert_eq!(items, vec![Value::String("left".into()), Value::String("right".into())]);
    }

    #[test]
    fn when_tuple_struct_serializer_collects_fields_it_should_return_array_value() {
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
    fn when_serializing_tuple_variant_it_should_return_unsupported_message() {
        // Arrange
        let mut serializer = ValueSeqSerializer::new(None);

        // Act
        let error = SerializeTupleVariant::serialize_field(&mut serializer, &123i32)
            .expect_err("tuple variant field should be unsupported");

        // Assert
        assert!(error.to_string().contains("tuple variants are unsupported"));
    }

    #[test]
    fn when_ending_tuple_variant_it_should_return_unsupported_message() {
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
}
