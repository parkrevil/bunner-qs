use super::*;
use crate::model::Value;
use serde::ser::SerializeMap;

mod value_map_serializer {
    use super::*;
    use serde::Serialize;
    use serde::ser::Error as _;

    #[derive(Debug)]
    struct FailingKey;

    impl Serialize for FailingKey {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(S::Error::custom("key serialization failed"))
        }
    }

    #[derive(Debug)]
    struct FailingValue;

    impl Serialize for FailingValue {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(S::Error::custom("value serialization failed"))
        }
    }

    #[test]
    fn should_store_string_value_when_serializing_single_entry_then_return_object_with_value() {
        let mut serializer = ValueMapSerializer::new();

        SerializeMap::serialize_key(&mut serializer, &"city")
            .expect("serializing key should succeed");
        SerializeMap::serialize_value(&mut serializer, &"Seoul")
            .expect("serializing value should succeed");
        let result = SerializeMap::end(serializer).expect("ending serializer should succeed");

        let map = match result.expect("map serializer should produce a value") {
            Value::Object(map) => map,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("city"), Some(&Value::String("Seoul".into())));
    }

    #[test]
    fn should_skip_entry_when_value_serializes_to_none_then_produce_empty_map() {
        let mut serializer = ValueMapSerializer::new();

        SerializeMap::serialize_key(&mut serializer, &"optional")
            .expect("serializing key should succeed");
        SerializeMap::serialize_value(&mut serializer, &Option::<String>::None)
            .expect("serializing none should succeed");
        let result = SerializeMap::end(serializer).expect("ending serializer should succeed");

        let map = match result.expect("map serializer should produce a value") {
            Value::Object(map) => map,
            other => panic!("unexpected value: {other:?}"),
        };
        assert!(map.is_empty());
    }

    #[test]
    fn should_error_when_serialize_value_called_without_key_then_return_missing_key_message() {
        let mut serializer = ValueMapSerializer::new();

        let error = SerializeMap::serialize_value(&mut serializer, &"orphan")
            .expect_err("missing key should be rejected");

        match error {
            SerializeError::Message(message) => {
                assert_eq!(message, "serialize_value called before serialize_key")
            }
            other => panic!("unexpected error type: {other:?}"),
        }
    }

    #[test]
    fn should_stringify_numeric_key_when_serializing_then_store_key_as_string() {
        let mut serializer = ValueMapSerializer::new();

        SerializeMap::serialize_key(&mut serializer, &42u8)
            .expect("serializing numeric key should succeed");
        SerializeMap::serialize_value(&mut serializer, &"answer")
            .expect("serializing value should succeed");
        let result = SerializeMap::end(serializer).expect("ending serializer should succeed");

        let map = match result.expect("map serializer should produce a value") {
            Value::Object(map) => map,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(map.get("42"), Some(&Value::String("answer".into())));
    }

    #[test]
    fn should_propagate_error_when_key_serialization_fails_then_return_message() {
        let mut serializer = ValueMapSerializer::new();

        let error = SerializeMap::serialize_key(&mut serializer, &FailingKey)
            .expect_err("failing key should error");

        match error {
            SerializeError::Message(message) => {
                assert_eq!(message, "key serialization failed")
            }
            other => panic!("unexpected error type: {other:?}"),
        }
    }

    #[test]
    fn should_propagate_error_when_value_serialization_fails_then_return_message() {
        let mut serializer = ValueMapSerializer::new();
        SerializeMap::serialize_key(&mut serializer, &"problem")
            .expect("serializing key should succeed");

        let error = SerializeMap::serialize_value(&mut serializer, &FailingValue)
            .expect_err("failing value should error");

        match error {
            SerializeError::Message(message) => {
                assert_eq!(message, "value serialization failed")
            }
            other => panic!("unexpected error type: {other:?}"),
        }
    }
}
