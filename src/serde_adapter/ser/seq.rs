use crate::model::Value;
use crate::serde_adapter::errors::SerializeError;
use serde::Serialize;
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

pub(crate) struct ValueSeqSerializer {
    items: Vec<Value>,
}

impl ValueSeqSerializer {
    pub(crate) fn new(len: Option<usize>) -> Self {
        ValueSeqSerializer {
            items: Vec::with_capacity(len.unwrap_or(0)),
        }
    }

    fn push_value(&mut self, value: Option<Value>) {
        match value {
            Some(serialized) => self.items.push(serialized),
            None => self.items.push(Value::String(String::new())),
        }
    }
}

impl SerializeSeq for ValueSeqSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let serializer = super::value::ValueSerializer::sequence_element();
        let serialized = value.serialize(serializer)?;
        self.push_value(serialized);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(Value::Array(self.items)))
    }
}

impl SerializeTuple for ValueSeqSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for ValueSeqSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleVariant for ValueSeqSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let type_name = std::any::type_name::<T>();
        let _ = value;
        Err(SerializeError::Message(format!(
            "tuple variants are unsupported; attempted to serialize element of type `{type_name}`"
        )))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::Message(
            "tuple variants are unsupported in query string serialization".into(),
        ))
    }
}
