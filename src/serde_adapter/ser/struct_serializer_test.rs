use super::*;
use crate::model::Value;
use serde::ser::SerializeStruct;

mod value_struct_serializer {
    use super::*;

    #[test]
    fn when_serializing_multiple_fields_it_should_collect_into_object() {
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
    fn when_field_serializes_to_none_it_should_skip_entry() {
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
    fn when_field_serializes_to_sequence_it_should_store_array() {
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
