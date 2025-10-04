use crate::model::{OrderedMap, Value};
use crate::serde_adapter::errors::SerializeError;
use ahash::RandomState;
use serde::Serialize;
use serde::ser::{self, Impossible, SerializeStruct};
use std::fmt::Display;

pub(crate) struct ValueStructSerializer {
    entries: OrderedMap<String, Value>,
}

impl ValueStructSerializer {
    pub(crate) fn new() -> Self {
        ValueStructSerializer {
            entries: OrderedMap::with_hasher(RandomState::default()),
        }
    }
}

impl SerializeStruct for ValueStructSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        if let Some(serialized) = value.serialize(super::value::ValueSerializer::root())? {
            self.entries.insert(key.to_string(), serialized);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(Value::Object(self.entries)))
    }
}

#[cfg(test)]
#[path = "struct_serializer_test.rs"]
mod struct_serializer_test;

pub(crate) struct MapKeySerializer;

impl MapKeySerializer {
    fn to_string<T: Display>(value: T) -> Result<String, SerializeError> {
        Ok(value.to_string())
    }
}

impl ser::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = SerializeError;
    type SerializeSeq = Impossible<String, SerializeError>;
    type SerializeTuple = Impossible<String, SerializeError>;
    type SerializeTupleStruct = Impossible<String, SerializeError>;
    type SerializeTupleVariant = Impossible<String, SerializeError>;
    type SerializeMap = Impossible<String, SerializeError>;
    type SerializeStruct = Impossible<String, SerializeError>;
    type SerializeStructVariant = Impossible<String, SerializeError>;

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Self::to_string(value)
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Self::to_string(v)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(String::from_utf8_lossy(value).into_owned())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::InvalidKey("unit".into()))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::InvalidKey(format!(
            "unit struct `{name}` cannot be used as a map key"
        )))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.to_string())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::InvalidKey("option".into()))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let _ = value;
        Err(SerializeError::InvalidKey(format!(
            "enum `{name}` variant `{variant}` (index {variant_index}) cannot be used as a map key"
        )))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(SerializeError::InvalidKey(format!(
            "sequence (length {:?}) cannot be used as a map key",
            len
        )))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(SerializeError::InvalidKey(format!(
            "tuple of length {len} cannot be used as a map key"
        )))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(SerializeError::InvalidKey(format!(
            "tuple struct `{name}` of length {len} cannot be used as a map key"
        )))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerializeError::InvalidKey(format!(
            "enum `{name}` tuple variant `{variant}` (index {variant_index}) with length {len} cannot be used as a map key"
        )))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerializeError::InvalidKey(format!(
            "map (length {:?}) cannot be used as a map key",
            len
        )))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(SerializeError::InvalidKey(format!(
            "struct `{name}` with {len} fields cannot be used as a map key"
        )))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerializeError::InvalidKey(format!(
            "enum `{name}` struct variant `{variant}` (index {variant_index}) with {len} fields cannot be used as a map key"
        )))
    }
}
