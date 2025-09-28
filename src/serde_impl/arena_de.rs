use crate::parsing::arena::{ArenaQueryMap, ArenaValue};
use crate::serde_impl::de::DeserializeError;
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};
use std::collections::HashSet;

fn format_expected(fields: &'static [&'static str]) -> String {
    if fields.is_empty() {
        "(none)".into()
    } else {
        fields.join(", ")
    }
}

pub(crate) fn deserialize_from_arena_map<T: DeserializeOwned>(
    map: &ArenaQueryMap<'_>,
) -> Result<T, DeserializeError> {
    T::deserialize(ArenaValueDeserializer {
        value: ArenaValueRef::Map(map.entries_slice()),
    })
}

#[derive(Clone, Copy)]
enum ArenaValueRef<'de> {
    String(&'de str),
    Seq(&'de [ArenaValue<'de>]),
    Map(&'de [(&'de str, ArenaValue<'de>)]),
}

impl<'de> ArenaValueRef<'de> {
    fn from_value(value: &'de ArenaValue<'de>) -> Self {
        match value {
            ArenaValue::String(s) => ArenaValueRef::String(s),
            ArenaValue::Seq(_) => ArenaValueRef::Seq(value.as_seq_slice().unwrap()),
            ArenaValue::Map { .. } => ArenaValueRef::Map(value.as_map_slice().unwrap()),
        }
    }
}

struct ArenaValueDeserializer<'de> {
    value: ArenaValueRef<'de>,
}

impl<'de> ArenaValueDeserializer<'de> {
    fn unexpected(&self) -> &'static str {
        match self.value {
            ArenaValueRef::String(_) => "string",
            ArenaValueRef::Seq(_) => "array",
            ArenaValueRef::Map(_) => "object",
        }
    }

    fn as_str(&self) -> Result<&'de str, DeserializeError> {
        match self.value {
            ArenaValueRef::String(s) => Ok(s),
            other => Err(DeserializeError::ExpectedString {
                found: match other {
                    ArenaValueRef::Seq(_) => "array",
                    ArenaValueRef::Map(_) => "object",
                    ArenaValueRef::String(_) => unreachable!(),
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

impl<'de> de::Deserializer<'de> for ArenaValueDeserializer<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ArenaValueRef::String(_) => self.deserialize_str(visitor),
            ArenaValueRef::Seq(_) => self.deserialize_seq(visitor),
            ArenaValueRef::Map(_) => self.deserialize_map(visitor),
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
            ArenaValueRef::String("") => visitor.visit_unit(),
            _ => Err(DeserializeError::UnexpectedType {
                expected: name,
                found: self.unexpected(),
            }),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ArenaValueRef::Seq(items) => {
                visitor.visit_seq(ArenaSequenceAccess { iter: items.iter() })
            }
            _ => Err(DeserializeError::UnexpectedType {
                expected: "array",
                found: self.unexpected(),
            }),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ArenaValueRef::Map(entries) => visitor.visit_map(ArenaMapDeserializer {
                iter: entries.iter(),
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
            ArenaValueRef::Map(entries) => visitor.visit_map(ArenaStructDeserializer {
                iter: entries.iter(),
                value: None,
                allowed: fields,
                seen: HashSet::with_capacity(entries.len()),
            }),
            _ => Err(DeserializeError::ExpectedObject {
                struct_name: name,
                found: self.unexpected(),
            }),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeserializeError::UnexpectedType {
            expected: "enum",
            found: self.unexpected(),
        })
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

struct ArenaSequenceAccess<'de> {
    iter: std::slice::Iter<'de, ArenaValue<'de>>,
}

impl<'de> SeqAccess<'de> for ArenaSequenceAccess<'de> {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(value) = self.iter.next() {
            let deserializer = ArenaValueDeserializer {
                value: ArenaValueRef::from_value(value),
            };
            seed.deserialize(deserializer).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct ArenaMapDeserializer<'de> {
    iter: std::slice::Iter<'de, (&'de str, ArenaValue<'de>)>,
    value: Option<&'de ArenaValue<'de>>,
}

impl<'de> MapAccess<'de> for ArenaMapDeserializer<'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            let key_deser = (*key).into_deserializer();
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
        seed.deserialize(ArenaValueDeserializer {
            value: ArenaValueRef::from_value(value),
        })
    }
}

struct ArenaStructDeserializer<'de> {
    iter: std::slice::Iter<'de, (&'de str, ArenaValue<'de>)>,
    value: Option<&'de ArenaValue<'de>>,
    allowed: &'static [&'static str],
    seen: HashSet<&'de str>,
}

impl<'de> MapAccess<'de> for ArenaStructDeserializer<'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            let key_str = *key;
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
        seed.deserialize(ArenaValueDeserializer {
            value: ArenaValueRef::from_value(value),
        })
    }
}
