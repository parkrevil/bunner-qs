use crate::model::{OrderedMap, Value};
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};
use std::collections::HashSet;
use std::fmt::Display;
use thiserror::Error;

fn format_expected(fields: &'static [&'static str]) -> String {
    if fields.is_empty() {
        "(none)".into()
    } else {
        fields.join(", ")
    }
}

#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error("{0}")]
    Message(String),
    #[error("expected an object for struct `{struct_name}`, found {found}")]
    ExpectedObject {
        struct_name: &'static str,
        found: &'static str,
    },
    #[error("unknown field `{field}`; expected one of: {expected}")]
    UnknownField { field: String, expected: String },
    #[error("duplicate field `{field}` encountered during deserialization")]
    DuplicateField { field: String },
    #[error("expected string value, found {found}")]
    ExpectedString { found: &'static str },
    #[error("invalid boolean literal `{value}`")]
    InvalidBool { value: String },
    #[error("invalid number literal `{value}`")]
    InvalidNumber { value: String },
    #[error("expected {expected}, found {found}")]
    UnexpectedType {
        expected: &'static str,
        found: &'static str,
    },
}

impl de::Error for DeserializeError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserializeError::Message(msg.to_string())
    }
}

pub(crate) fn deserialize_from_query_map<T: DeserializeOwned>(
    map: &OrderedMap<String, Value>,
) -> Result<T, DeserializeError> {
    T::deserialize(ValueDeserializer {
        value: ValueRef::Object(map),
    })
}

#[derive(Clone, Copy)]
enum ValueRef<'de> {
    String(&'de str),
    Array(&'de [Value]),
    Object(&'de OrderedMap<String, Value>),
}

impl<'de> ValueRef<'de> {
    fn from_value(value: &'de Value) -> Self {
        match value {
            Value::String(s) => ValueRef::String(s),
            Value::Array(items) => ValueRef::Array(items),
            Value::Object(map) => ValueRef::Object(map),
        }
    }
}

struct ValueDeserializer<'de> {
    value: ValueRef<'de>,
}

impl<'de> ValueDeserializer<'de> {
    fn unexpected(&self) -> &'static str {
        match self.value {
            ValueRef::String(_) => "string",
            ValueRef::Array(_) => "array",
            ValueRef::Object(_) => "object",
        }
    }

    fn as_str(&self) -> Result<&'de str, DeserializeError> {
        match self.value {
            ValueRef::String(s) => Ok(s),
            other => Err(DeserializeError::ExpectedString {
                found: match other {
                    ValueRef::Array(_) => "array",
                    ValueRef::Object(_) => "object",
                    ValueRef::String(_) => unreachable!(),
                },
            }),
        }
    }

    fn parse_number<N, F>(&self, parse: F) -> Result<N, DeserializeError>
    where
        F: FnOnce(&str) -> Result<N, std::num::ParseIntError>,
    {
        let s = self.as_str()?;
        parse(s).map_err(|_| DeserializeError::InvalidNumber {
            value: s.to_string(),
        })
    }

    fn parse_float<N>(&self) -> Result<N, DeserializeError>
    where
        N: std::str::FromStr,
    {
        let s = self.as_str()?;
        s.parse::<N>().map_err(|_| DeserializeError::InvalidNumber {
            value: s.to_string(),
        })
    }
}

impl<'de> de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ValueRef::String(_) => self.deserialize_str(visitor),
            ValueRef::Array(_) => self.deserialize_seq(visitor),
            ValueRef::Object(_) => self.deserialize_map(visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s = self.as_str()?;
        match s {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            other => Err(DeserializeError::InvalidBool {
                value: other.to_string(),
            }),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i128(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u128(self.parse_number(|s| s.parse())?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s = self.as_str()?;
        let mut chars = s.chars();
        if let (Some(ch), None) = (chars.next(), chars.next()) {
            visitor.visit_char(ch)
        } else {
            Err(DeserializeError::InvalidNumber {
                value: s.to_string(),
            })
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s = self.as_str()?;
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s = self.as_str()?;
        visitor.visit_string(s.to_string())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s = self.as_str()?;
        visitor.visit_borrowed_bytes(s.as_bytes())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s = self.as_str()?;
        visitor.visit_byte_buf(s.as_bytes().to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s = self.as_str()?;
        if s.is_empty() {
            visitor.visit_unit()
        } else {
            Err(DeserializeError::UnexpectedType {
                expected: "empty string for unit",
                found: "non-empty string",
            })
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ValueRef::String("") => visitor.visit_unit(),
            _ => Err(DeserializeError::UnexpectedType {
                expected: name,
                found: self.unexpected(),
            }),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        debug_assert!(!name.is_empty(), "newtype struct should have a name");
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ValueRef::Array(items) => visitor.visit_seq(SequenceAccess { iter: items.iter() }),
            _ => Err(DeserializeError::UnexpectedType {
                expected: "array",
                found: self.unexpected(),
            }),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ValueRef::Array(items) => {
                if items.len() != len {
                    return Err(DeserializeError::Message(format!(
                        "expected tuple of length {len}, found {}",
                        items.len()
                    )));
                }
                visitor.visit_seq(SequenceAccess { iter: items.iter() })
            }
            _ => Err(DeserializeError::UnexpectedType {
                expected: "tuple",
                found: self.unexpected(),
            }),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ValueRef::Array(items) => {
                if items.len() != len {
                    return Err(DeserializeError::Message(format!(
                        "expected tuple struct `{name}` with {len} elements, found {}",
                        items.len()
                    )));
                }
                visitor.visit_seq(SequenceAccess { iter: items.iter() })
            }
            _ => Err(DeserializeError::UnexpectedType {
                expected: name,
                found: self.unexpected(),
            }),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ValueRef::Object(map) => visitor.visit_map(MapDeserializer {
                iter: map.iter(),
                value: None,
            }),
            _ => Err(DeserializeError::UnexpectedType {
                expected: "object",
                found: self.unexpected(),
            }),
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ValueRef::Object(map) => visitor.visit_map(StructDeserializer {
                iter: map.iter(),
                value: None,
                allowed: fields,
                seen: HashSet::with_capacity(map.len()),
            }),
            _ => Err(DeserializeError::ExpectedObject {
                struct_name: name,
                found: self.unexpected(),
            }),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(visitor);
        Err(DeserializeError::Message(format!(
            "enum `{name}` with variants [{}] cannot be deserialized from query strings",
            format_expected(variants)
        )))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct SequenceAccess<'de> {
    iter: std::slice::Iter<'de, Value>,
}

impl<'de> SeqAccess<'de> for SequenceAccess<'de> {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(value) = self.iter.next() {
            let deserializer = ValueDeserializer {
                value: ValueRef::from_value(value),
            };
            seed.deserialize(deserializer).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct MapDeserializer<'de> {
    iter: indexmap::map::Iter<'de, String, Value>,
    value: Option<&'de Value>,
}

impl<'de> MapAccess<'de> for MapDeserializer<'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            let key_deser = key.as_str().into_deserializer();
            seed.deserialize(key_deser).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .ok_or_else(|| DeserializeError::Message("value missing for map entry".into()))?;
        seed.deserialize(ValueDeserializer {
            value: ValueRef::from_value(value),
        })
    }
}

struct StructDeserializer<'de> {
    iter: indexmap::map::Iter<'de, String, Value>,
    value: Option<&'de Value>,
    allowed: &'static [&'static str],
    seen: HashSet<&'de str>,
}

impl<'de> MapAccess<'de> for StructDeserializer<'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            let key_str = key.as_str();
            if !self.allowed.contains(&key_str) {
                return Err(DeserializeError::UnknownField {
                    field: key_str.to_string(),
                    expected: format_expected(self.allowed),
                });
            }
            if !self.seen.insert(key_str) {
                return Err(DeserializeError::DuplicateField {
                    field: key_str.to_string(),
                });
            }
            self.value = Some(value);
            let key_deser = key_str.into_deserializer();
            seed.deserialize(key_deser).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .ok_or_else(|| DeserializeError::Message("value missing for struct field".into()))?;
        seed.deserialize(ValueDeserializer {
            value: ValueRef::from_value(value),
        })
    }
}
