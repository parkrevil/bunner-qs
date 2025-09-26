use crate::value::Value;
use indexmap::IndexMap;
use serde::ser::{self, Impossible, Serialize, SerializeMap, SerializeSeq, SerializeStruct};
use std::fmt::{self, Display};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("{0}")]
    Message(String),
    #[error("top-level must serialize to a map, found {0}")]
    TopLevel(String),
    #[error("map key must be a string, found {0}")]
    InvalidKey(String),
    #[error("unexpected placeholder value encountered during serialization")]
    UnexpectedSkip,
    #[error("unsupported serialization form: {0}")]
    Unsupported(&'static str),
}

impl ser::Error for SerializeError {
    fn custom<T: Display>(msg: T) -> Self {
        SerializeError::Message(msg.to_string())
    }
}

#[derive(Debug)]
enum FormValue {
    String(String),
    Array(Vec<FormValue>),
    Object(IndexMap<String, FormValue>),
    Skip,
}

pub(crate) fn serialize_to_query_map<T: Serialize>(data: &T) -> Result<IndexMap<String, Value>, SerializeError> {
    match data.serialize(FormSerializer)? {
        FormValue::Object(map) => map_into_value(map),
        other => Err(SerializeError::TopLevel(describe_form_value(&other))),
    }
}

fn map_into_value(map: IndexMap<String, FormValue>) -> Result<IndexMap<String, Value>, SerializeError> {
    let mut output = IndexMap::with_capacity(map.len());
    for (key, value) in map {
        let converted = form_value_to_value(value)?;
        output.insert(key, converted);
    }
    Ok(output)
}

fn form_value_to_value(value: FormValue) -> Result<Value, SerializeError> {
    match value {
        FormValue::String(s) => Ok(Value::String(s)),
        FormValue::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match form_value_to_value(item)? {
                    Value::String(s) => out.push(Value::String(s)),
                    Value::Array(arr) => out.push(Value::Array(arr)),
                    Value::Object(obj) => out.push(Value::Object(obj)),
                }
            }
            Ok(Value::Array(out))
        }
        FormValue::Object(map) => map_into_value(map).map(Value::Object),
        FormValue::Skip => Err(SerializeError::UnexpectedSkip),
    }
}

fn describe_form_value(value: &FormValue) -> String {
    match value {
        FormValue::String(_) => "string".into(),
        FormValue::Array(_) => "array".into(),
        FormValue::Object(_) => "object".into(),
        FormValue::Skip => "skip".into(),
    }
}

struct FormSerializer;

impl ser::Serializer for FormSerializer {
    type Ok = FormValue;
    type Error = SerializeError;
    type SerializeSeq = FormSeqSerializer;
    type SerializeTuple = FormSeqSerializer;
    type SerializeTupleStruct = FormSeqSerializer;
    type SerializeTupleVariant = FormSeqSerializer;
    type SerializeMap = FormMapSerializer;
    type SerializeStruct = FormStructSerializer;
    type SerializeStructVariant = Impossible<FormValue, SerializeError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(String::from_utf8_lossy(v).into_owned()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::Skip)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(FormSerializer)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(String::new()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(String::new()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(FormSerializer)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::Unsupported("enum newtype variant"))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(FormSeqSerializer {
            items: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerializeError::Unsupported("tuple variant"))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(FormMapSerializer {
            entries: IndexMap::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(FormStructSerializer {
            entries: IndexMap::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerializeError::Unsupported("struct variant"))
    }
}

struct FormSeqSerializer {
    items: Vec<FormValue>,
}

impl SerializeSeq for FormSeqSerializer {
    type Ok = FormValue;
    type Error = SerializeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        match value.serialize(FormSerializer)? {
            FormValue::Skip => Ok(()),
            other => {
                self.items.push(other);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::Array(self.items))
    }
}

impl serde::ser::SerializeTuple for FormSeqSerializer {
    type Ok = FormValue;
    type Error = SerializeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for FormSeqSerializer {
    type Ok = FormValue;
    type Error = SerializeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for FormSeqSerializer {
    type Ok = FormValue;
    type Error = SerializeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Self::Error> {
        Err(SerializeError::Unsupported("tuple variant"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::Unsupported("tuple variant"))
    }
}

struct FormMapSerializer {
    entries: IndexMap<String, FormValue>,
    next_key: Option<String>,
}

impl SerializeMap for FormMapSerializer {
    type Ok = FormValue;
    type Error = SerializeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let serialized = key.serialize(MapKeySerializer)?;
        self.next_key = Some(serialized);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| SerializeError::Message("serialize_value called before serialize_key".into()))?;
        match value.serialize(FormSerializer)? {
            FormValue::Skip => Ok(()),
            other => {
                self.entries.insert(key, other);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::Object(self.entries))
    }
}

struct FormStructSerializer {
    entries: IndexMap<String, FormValue>,
}

impl SerializeStruct for FormStructSerializer {
    type Ok = FormValue;
    type Error = SerializeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        match value.serialize(FormSerializer)? {
            FormValue::Skip => Ok(()),
            other => {
                self.entries.insert(key.to_string(), other);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(FormValue::Object(self.entries))
    }
}

struct MapKeySerializer;

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
        Ok(value.to_string())
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(String::from_utf8_lossy(value).into_owned())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::InvalidKey("unit".into()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::InvalidKey("unit struct".into()))
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

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::InvalidKey("enum variant".into()))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(SerializeError::InvalidKey("seq".into()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(SerializeError::InvalidKey("tuple".into()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(SerializeError::InvalidKey("tuple struct".into()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerializeError::InvalidKey("tuple variant".into()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerializeError::InvalidKey("map".into()))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(SerializeError::InvalidKey("struct".into()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerializeError::InvalidKey("struct variant".into()))
    }
}