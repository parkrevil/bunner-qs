use super::*;
use crate::model::Value;
use serde::ser::SerializeMap;

mod value_map_serializer {
    use super::*;

    #[test]
    fn stores_string_value_when_serializing_single_entry() {
        // Arrange
        let mut serializer = ValueMapSerializer::new();

        // Act
        SerializeMap::serialize_key(&mut serializer, &"city")
            .expect("serializing key should succeed");
        SerializeMap::serialize_value(&mut serializer, &"Seoul")
            .expect("serializing value should succeed");
        let result = SerializeMap::end(serializer).expect("ending serializer should succeed");

        // Assert
        let map = match result.expect("map serializer should produce a value") {
            Value::Object(map) => map,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("city"), Some(&Value::String("Seoul".into())));
    }

    #[test]
    fn skips_entry_when_value_serializes_to_none() {
        // Arrange
        let mut serializer = ValueMapSerializer::new();

        // Act
        SerializeMap::serialize_key(&mut serializer, &"optional")
            .expect("serializing key should succeed");
        SerializeMap::serialize_value(&mut serializer, &Option::<String>::None)
            .expect("serializing none should succeed");
        let result = SerializeMap::end(serializer).expect("ending serializer should succeed");

        // Assert
        let map = match result.expect("map serializer should produce a value") {
            Value::Object(map) => map,
            other => panic!("unexpected value: {other:?}"),
        };
        assert!(map.is_empty());
    }

    #[test]
    fn errors_when_serialize_value_called_without_key() {
        // Arrange
        let mut serializer = ValueMapSerializer::new();

        // Act
        let error = SerializeMap::serialize_value(&mut serializer, &"orphan")
            .expect_err("missing key should be rejected");

        // Assert
        match error {
            SerializeError::Message(message) => {
                assert_eq!(message, "serialize_value called before serialize_key")
            }
            other => panic!("unexpected error type: {other:?}"),
        }
    }

    #[test]
    fn stringifies_numeric_key_when_serializing() {
        // Arrange
        let mut serializer = ValueMapSerializer::new();

        // Act
        SerializeMap::serialize_key(&mut serializer, &42u8)
            .expect("serializing numeric key should succeed");
        SerializeMap::serialize_value(&mut serializer, &"answer")
            .expect("serializing value should succeed");
        let result = SerializeMap::end(serializer).expect("ending serializer should succeed");

        // Assert
        let map = match result.expect("map serializer should produce a value") {
            Value::Object(map) => map,
            other => panic!("unexpected value: {other:?}"),
        };
        assert_eq!(map.get("42"), Some(&Value::String("answer".into())));
    }
}
