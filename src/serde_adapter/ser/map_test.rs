use super::*;
use crate::model::Value;
use assert_matches::assert_matches;
use serde::ser::SerializeMap;

mod value_map_serializer {
    use super::*;
    use crate::model::OrderedMap;
    use serde::Serialize;
    use serde::ser::Error as _;
    use serde_json::json;

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

    fn finalize(serializer: ValueMapSerializer) -> Value {
        SerializeMap::end(serializer)
            .expect("ending serializer should succeed")
            .expect("map serializer should produce a value")
    }

    fn object(value: &Value) -> &OrderedMap<String, Value> {
        value.as_object().expect("value should be object")
    }

    #[test]
    fn should_store_string_value_when_serializing_single_entry_then_return_object_with_value() {
        let mut serializer = ValueMapSerializer::new();

        SerializeMap::serialize_key(&mut serializer, &"city")
            .expect("serializing key should succeed");
        SerializeMap::serialize_value(&mut serializer, &"Seoul")
            .expect("serializing value should succeed");
    let value = finalize(serializer);
    let map = object(&value);
        assert_eq!(map.len(), 1);
        assert_matches!(map.get("city"), Some(Value::String(text)) if text == "Seoul");
    }

    #[test]
    fn should_skip_entry_when_value_serializes_to_none_then_produce_empty_map() {
        let mut serializer = ValueMapSerializer::new();

        SerializeMap::serialize_key(&mut serializer, &"optional")
            .expect("serializing key should succeed");
        SerializeMap::serialize_value(&mut serializer, &Option::<String>::None)
            .expect("serializing none should succeed");
    let value = finalize(serializer);
    let map = object(&value);
        assert!(map.is_empty());
    }

    #[test]
    fn should_require_new_key_after_none_value_then_disallow_orphan_value() {
        let mut serializer = ValueMapSerializer::new();

        SerializeMap::serialize_key(&mut serializer, &"optional")
            .expect("serializing key should succeed");
        SerializeMap::serialize_value(&mut serializer, &Option::<String>::None)
            .expect("serializing none should succeed");

        let error = SerializeMap::serialize_value(&mut serializer, &"orphan")
            .expect_err("value without key should be rejected after none");

        assert_matches!(
            &error,
            SerializeError::Message(message)
                if message == "serialize_value called before serialize_key",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_error_when_serialize_value_called_without_key_then_return_missing_key_message() {
        let mut serializer = ValueMapSerializer::new();

        let error = SerializeMap::serialize_value(&mut serializer, &"orphan")
            .expect_err("missing key should be rejected");

        assert_matches!(
            &error,
            SerializeError::Message(message)
                if message == "serialize_value called before serialize_key",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_stringify_numeric_key_when_serializing_then_store_key_as_string() {
        let mut serializer = ValueMapSerializer::new();

        SerializeMap::serialize_key(&mut serializer, &42u8)
            .expect("serializing numeric key should succeed");
        SerializeMap::serialize_value(&mut serializer, &"answer")
            .expect("serializing value should succeed");
    let value = finalize(serializer);
    let map = object(&value);
        assert_matches!(map.get("42"), Some(Value::String(text)) if text == "answer");
    }

    #[test]
    fn should_propagate_error_when_key_serialization_fails_then_return_message() {
        let mut serializer = ValueMapSerializer::new();

        let error = SerializeMap::serialize_key(&mut serializer, &FailingKey)
            .expect_err("failing key should error");

        assert_matches!(
            &error,
            SerializeError::Message(message) if message == "key serialization failed",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_propagate_error_when_value_serialization_fails_then_return_message() {
        let mut serializer = ValueMapSerializer::new();
        SerializeMap::serialize_key(&mut serializer, &"problem")
            .expect("serializing key should succeed");

        let error = SerializeMap::serialize_value(&mut serializer, &FailingValue)
            .expect_err("failing value should error");

        assert_matches!(
            &error,
            SerializeError::Message(message) if message == "value serialization failed",
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn should_replace_pending_key_when_next_key_overwritten_then_use_latest_value_label() {
        let mut serializer = ValueMapSerializer::new();

        SerializeMap::serialize_key(&mut serializer, &"stale")
            .expect("serializing first key should succeed");
        SerializeMap::serialize_key(&mut serializer, &"fresh")
            .expect("serializing replacement key should succeed");
        SerializeMap::serialize_value(&mut serializer, &"value")
            .expect("serializing value should succeed");
    let value = finalize(serializer);
    let map = object(&value);
        assert_eq!(map.len(), 1);
        assert_matches!(map.get("fresh"), Some(Value::String(text)) if text == "value");
    }

    #[test]
    fn should_allow_recovery_after_key_serialization_error_then_accept_new_key() {
        let mut serializer = ValueMapSerializer::new();

        let error = SerializeMap::serialize_key(&mut serializer, &FailingKey)
            .expect_err("failing key should raise error");
        assert_matches!(
            &error,
            SerializeError::Message(message) if message == "key serialization failed",
            "unexpected error: {error:?}"
        );

        SerializeMap::serialize_key(&mut serializer, &"recovered")
            .expect("serializer should continue after key failure");
        SerializeMap::serialize_value(&mut serializer, &"value")
            .expect("value should serialize with recovered key");
    let value = finalize(serializer);
    let map = object(&value);
        assert_eq!(map.len(), 1);
        assert_matches!(map.get("recovered"), Some(Value::String(text)) if text == "value");
    }

    #[test]
    fn should_produce_empty_object_when_end_called_without_entries_then_return_empty_map() {
        let serializer = ValueMapSerializer::new();

    let value = finalize(serializer);
    let map = object(&value);
        assert!(map.is_empty());
    }

    #[test]
    fn should_recover_after_value_serialization_error_then_accept_new_entry() {
        let mut serializer = ValueMapSerializer::new();
        SerializeMap::serialize_key(&mut serializer, &"broken")
            .expect("serializing key should succeed");

        let error = SerializeMap::serialize_value(&mut serializer, &FailingValue)
            .expect_err("value serialization should error");
        assert_matches!(
            &error,
            SerializeError::Message(message) if message == "value serialization failed"
        );

        SerializeMap::serialize_key(&mut serializer, &"fixed")
            .expect("serializing replacement key should succeed");
        SerializeMap::serialize_value(&mut serializer, &"value")
            .expect("serializing replacement value should succeed");

    let value = finalize(serializer);
    let map = object(&value);
        assert_eq!(map.len(), 1);
        assert_matches!(map.get("fixed"), Some(Value::String(text)) if text == "value");
    }

    #[test]
    fn should_store_object_value_when_serializing_nested_structure_then_preserve_object() {
        let mut serializer = ValueMapSerializer::new();

        SerializeMap::serialize_key(&mut serializer, &"payload")
            .expect("serializing key should succeed");
        SerializeMap::serialize_value(&mut serializer, &json!({"inner": 1, "flag": true}))
            .expect("serializing object should succeed");

    let value = finalize(serializer);
    let map = object(&value);
        let payload = map.get("payload").expect("payload entry should exist");
        let payload_obj = payload.as_object().expect("payload should be object");
        assert_eq!(payload_obj.get("inner"), Some(&Value::String("1".into())));
        assert_eq!(payload_obj.get("flag"), Some(&Value::String("true".into())));
    }
}
