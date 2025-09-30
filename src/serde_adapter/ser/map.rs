use crate::model::{OrderedMap, Value};
use crate::serde_adapter::errors::SerializeError;
use ahash::RandomState;
use serde::Serialize;
use serde::ser::SerializeMap;

pub(crate) struct ValueMapSerializer {
    entries: OrderedMap<String, Value>,
    next_key: Option<String>,
}

impl ValueMapSerializer {
    pub(crate) fn new() -> Self {
        ValueMapSerializer {
            entries: OrderedMap::with_hasher(RandomState::default()),
            next_key: None,
        }
    }
}

impl SerializeMap for ValueMapSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let serialized = key.serialize(super::struct_serializer::MapKeySerializer)?;
        self.next_key = Some(serialized);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self.next_key.take().ok_or_else(|| {
            SerializeError::Message("serialize_value called before serialize_key".into())
        })?;
        if let Some(serialized) = value.serialize(super::value::ValueSerializer::root())? {
            self.entries.insert(key, serialized);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(Value::Object(self.entries)))
    }
}

#[cfg(test)]
#[path = "map_test.rs"]
mod map_test;
