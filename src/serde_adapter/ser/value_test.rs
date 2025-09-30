use super::*;
use crate::model::Value;
use serde::Serialize;
use serde::ser::{SerializeSeq, Serializer};

mod serialize_to_query_map {
    use super::*;

    #[derive(Serialize)]
    struct Profile<'a> {
        name: &'a str,
        age: u32,
    }

    #[test]
    fn when_serializing_struct_it_should_return_ordered_map() {
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
    fn when_top_level_is_string_it_should_report_top_level_error() {
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
    fn when_top_level_option_is_none_it_should_report_unexpected_skip() {
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
}

mod value_serializer {
    use super::*;

    #[test]
    fn when_serializing_boolean_true_it_should_return_string_true() {
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
    fn when_serializing_bytes_it_should_convert_to_utf8_string() {
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
    fn when_serializing_none_at_root_it_should_return_none() {
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
    fn when_serializing_newtype_variant_it_should_return_unsupported_error() {
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
    fn when_preserving_none_in_sequence_it_should_emit_empty_string() {
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
}
