use crate::model::{OrderedMap, Value};
use crate::serde_adapter::errors::SerializeError;
use ahash::RandomState;
use serde::ser::{self, Impossible, Serialize, SerializeMap, SerializeSeq, SerializeStruct};
use std::fmt::Display;

pub(crate) fn serialize_to_query_map<T: Serialize>(
    data: &T,
) -> Result<OrderedMap<String, Value>, SerializeError> {
    match data.serialize(ValueSerializer)? {
        Some(Value::Object(map)) => Ok(map),
        Some(other) => Err(SerializeError::TopLevel(describe_value(&other))),
        None => Err(SerializeError::UnexpectedSkip),
    }
}

fn describe_value(value: &Value) -> String {
    match value {
        Value::String(_) => "string".into(),
        Value::Array(_) => "array".into(),
        Value::Object(_) => "object".into(),
    }
}

struct ValueSerializer;

fn string_value<T: Display>(value: T) -> Value {
    Value::String(value.to_string())
}

impl ser::Serializer for ValueSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;
    type SerializeSeq = ValueSeqSerializer;
    type SerializeTuple = ValueSeqSerializer;
    type SerializeTupleStruct = ValueSeqSerializer;
    type SerializeTupleVariant = ValueSeqSerializer;
    type SerializeMap = ValueMapSerializer;
    type SerializeStruct = ValueStructSerializer;
    type SerializeStructVariant = Impossible<Option<Value>, SerializeError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value(v)))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Some(Value::String(String::from_utf8_lossy(v).into_owned())))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(ValueSerializer)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(string_value("")))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        debug_assert!(!name.is_empty(), "unit struct should have a name");
        Ok(Some(string_value("")))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        debug_assert!(!name.is_empty(), "enum should have a name");
        debug_assert!(variant_index < u32::MAX, "variant index should be finite");
        Ok(Some(string_value(variant)))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        debug_assert!(!name.is_empty(), "newtype struct should have a name");
        value.serialize(ValueSerializer)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let _ = value;
        Err(SerializeError::Message(format!(
            "enum `{name}` newtype variant `{variant}` (index {variant_index}) cannot be serialized into query strings"
        )))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ValueSeqSerializer {
            items: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        debug_assert!(!name.is_empty(), "tuple struct should have a name");
        debug_assert!(len < usize::MAX, "tuple struct length must be finite");
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerializeError::Message(format!(
            "enum `{name}` tuple variant `{variant}` (index {variant_index}) with length {len} cannot be serialized into query strings"
        )))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let capacity = len.unwrap_or(0);
        let entries = if capacity == 0 {
            OrderedMap::with_hasher(RandomState::default())
        } else {
            OrderedMap::with_capacity_and_hasher(capacity, RandomState::default())
        };
        Ok(ValueMapSerializer {
            entries,
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        debug_assert!(!name.is_empty(), "struct should have a name");
        debug_assert!(len < usize::MAX, "struct field length must be finite");
        let entries = if len == 0 {
            OrderedMap::with_hasher(RandomState::default())
        } else {
            OrderedMap::with_capacity_and_hasher(len, RandomState::default())
        };
        Ok(ValueStructSerializer { entries })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerializeError::Message(format!(
            "enum `{name}` struct variant `{variant}` (index {variant_index}) with {len} fields cannot be serialized into query strings"
        )))
    }
}

struct ValueSeqSerializer {
    items: Vec<Value>,
}

impl SerializeSeq for ValueSeqSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        if let Some(serialized) = value.serialize(ValueSerializer)? {
            self.items.push(serialized);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(Value::Array(self.items)))
    }
}

impl serde::ser::SerializeTuple for ValueSeqSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for ValueSeqSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for ValueSeqSerializer {
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

struct ValueMapSerializer {
    entries: OrderedMap<String, Value>,
    next_key: Option<String>,
}

impl SerializeMap for ValueMapSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let serialized = key.serialize(MapKeySerializer)?;
        self.next_key = Some(serialized);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self.next_key.take().ok_or_else(|| {
            SerializeError::Message("serialize_value called before serialize_key".into())
        })?;
        if let Some(serialized) = value.serialize(ValueSerializer)? {
            self.entries.insert(key, serialized);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(Value::Object(self.entries)))
    }
}

struct ValueStructSerializer {
    entries: OrderedMap<String, Value>,
}

impl SerializeStruct for ValueStructSerializer {
    type Ok = Option<Value>;
    type Error = SerializeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        if let Some(serialized) = value.serialize(ValueSerializer)? {
            self.entries.insert(key.to_string(), serialized);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(Value::Object(self.entries)))
    }
}

struct MapKeySerializer;

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
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        debug_assert!(!name.is_empty(), "enum should have a name");
        debug_assert!(variant_index < u32::MAX, "variant index must be finite");
        Ok(variant.to_string())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        debug_assert!(!name.is_empty(), "newtype struct should have a name");
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
