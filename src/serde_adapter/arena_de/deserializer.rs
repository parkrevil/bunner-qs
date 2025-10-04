use crate::parsing::arena::{ArenaQueryMap, ArenaValue};
use crate::serde_adapter::errors::{
    DeserializeError, DeserializeErrorKind, PathSegment, format_expected,
};
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};
use std::collections::HashSet;

pub(crate) fn deserialize_from_arena_map<T: DeserializeOwned>(
    map: &ArenaQueryMap<'_>,
) -> Result<T, DeserializeError> {
    T::deserialize(ArenaValueDeserializer::new(
        super::value_ref::ArenaValueRef::Map(map.entries_slice()),
        Vec::new(),
    ))
}

pub(crate) struct ArenaValueDeserializer<'de> {
    value: super::value_ref::ArenaValueRef<'de>,
    path: Vec<PathSegment>,
}

impl<'de> ArenaValueDeserializer<'de> {
    fn new(value: super::value_ref::ArenaValueRef<'de>, path: Vec<PathSegment>) -> Self {
        Self { value, path }
    }

    fn error(&self, kind: DeserializeErrorKind) -> DeserializeError {
        DeserializeError::from_kind(kind).with_path(self.path.clone())
    }

    fn unexpected(&self) -> &'static str {
        match self.value {
            super::value_ref::ArenaValueRef::String(_) => "string",
            super::value_ref::ArenaValueRef::Seq(_) => "array",
            super::value_ref::ArenaValueRef::Map(_) => "object",
        }
    }

    fn as_str(&self) -> Result<&'de str, DeserializeError> {
        match self.value {
            super::value_ref::ArenaValueRef::String(s) => Ok(s),
            super::value_ref::ArenaValueRef::Seq(_) => {
                Err(self.error(DeserializeErrorKind::ExpectedString { found: "array" }))
            }
            super::value_ref::ArenaValueRef::Map(_) => {
                Err(self.error(DeserializeErrorKind::ExpectedString { found: "object" }))
            }
        }
    }

    fn parse_number<N, F>(&self, parse: F) -> Result<N, DeserializeError>
    where
        F: FnOnce(&str) -> Result<N, std::num::ParseIntError>,
    {
        let s = self.as_str()?;
        parse(s).map_err(|_| {
            self.error(DeserializeErrorKind::InvalidNumber {
                value: s.to_string(),
            })
        })
    }

    fn parse_float<N>(&self) -> Result<N, DeserializeError>
    where
        N: std::str::FromStr,
    {
        let s = self.as_str()?;
        s.parse::<N>().map_err(|_| {
            self.error(DeserializeErrorKind::InvalidNumber {
                value: s.to_string(),
            })
        })
    }

    fn visit_sequence<V>(
        self,
        visitor: V,
        expected: &'static str,
    ) -> Result<V::Value, DeserializeError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            super::value_ref::ArenaValueRef::Seq(items) => {
                visitor.visit_seq(ArenaSequenceAccess::new(items.iter(), self.path.clone()))
            }
            _ => Err(self.error(DeserializeErrorKind::UnexpectedType {
                expected,
                found: self.unexpected(),
            })),
        }
    }

    fn visit_fixed_sequence<V, F>(
        self,
        expected_len: usize,
        visitor: V,
        expected_label: &'static str,
        format_mismatch: F,
    ) -> Result<V::Value, DeserializeError>
    where
        V: Visitor<'de>,
        F: FnOnce(usize) -> String,
    {
        match self.value {
            super::value_ref::ArenaValueRef::Seq(items) => {
                if items.len() != expected_len {
                    return Err(
                        self.error(DeserializeErrorKind::Message(format_mismatch(items.len())))
                    );
                }
                visitor.visit_seq(ArenaSequenceAccess::new(items.iter(), self.path.clone()))
            }
            _ => Err(self.error(DeserializeErrorKind::UnexpectedType {
                expected: expected_label,
                found: self.unexpected(),
            })),
        }
    }
}

impl<'de> de::Deserializer<'de> for ArenaValueDeserializer<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            super::value_ref::ArenaValueRef::String(_) => self.deserialize_str(visitor),
            super::value_ref::ArenaValueRef::Seq(_) => self.deserialize_seq(visitor),
            super::value_ref::ArenaValueRef::Map(_) => self.deserialize_map(visitor),
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
            other => Err(self.error(DeserializeErrorKind::InvalidBool {
                value: other.to_string(),
            })),
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
            Err(self.error(DeserializeErrorKind::InvalidNumber {
                value: s.to_string(),
            }))
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
            Err(self.error(DeserializeErrorKind::UnexpectedType {
                expected: "empty string for unit",
                found: "non-empty string",
            }))
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
            super::value_ref::ArenaValueRef::String("") => visitor.visit_unit(),
            _ => Err(self.error(DeserializeErrorKind::UnexpectedType {
                expected: name,
                found: self.unexpected(),
            })),
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
        self.visit_sequence(visitor, "array")
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.visit_fixed_sequence(len, visitor, "tuple", |actual| {
            format!("expected tuple of length {len}, found {actual}")
        })
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
        self.visit_fixed_sequence(len, visitor, name, |actual| {
            format!("expected tuple struct `{name}` with {len} elements, found {actual}")
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            super::value_ref::ArenaValueRef::Map(map) => visitor.visit_map(ArenaMapDeserializer {
                iter: map.iter(),
                value: None,
                path: self.path.clone(),
                pending_key: None,
            }),
            _ => Err(self.error(DeserializeErrorKind::UnexpectedType {
                expected: "object",
                found: self.unexpected(),
            })),
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
            super::value_ref::ArenaValueRef::Map(map) => {
                visitor.visit_map(ArenaStructDeserializer {
                    iter: map.iter(),
                    value: None,
                    allowed: fields,
                    seen: HashSet::with_capacity(map.len()),
                    path: self.path.clone(),
                    pending_key: None,
                })
            }
            _ => Err(self.error(DeserializeErrorKind::ExpectedObject {
                struct_name: name,
                found: self.unexpected(),
            })),
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
        Err(self.error(DeserializeErrorKind::Message(format!(
            "enum `{name}` with variants [{}] cannot be deserialized from query strings",
            format_expected(variants)
        ))))
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

pub(crate) struct ArenaSequenceAccess<'de> {
    iter: std::slice::Iter<'de, ArenaValue<'de>>,
    path: Vec<PathSegment>,
    index: usize,
}

impl<'de> ArenaSequenceAccess<'de> {
    fn new(iter: std::slice::Iter<'de, ArenaValue<'de>>, path: Vec<PathSegment>) -> Self {
        Self {
            iter,
            path,
            index: 0,
        }
    }
}

impl<'de> SeqAccess<'de> for ArenaSequenceAccess<'de> {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(value) = self.iter.next() {
            let mut path = self.path.clone();
            path.push(PathSegment::Index(self.index));
            self.index += 1;
            let deserializer = ArenaValueDeserializer::new(
                super::value_ref::ArenaValueRef::from_value(value),
                path,
            );
            seed.deserialize(deserializer).map(Some)
        } else {
            Ok(None)
        }
    }
}

pub(crate) struct ArenaMapDeserializer<'de> {
    iter: std::slice::Iter<'de, (&'de str, ArenaValue<'de>)>,
    value: Option<&'de ArenaValue<'de>>,
    path: Vec<PathSegment>,
    pending_key: Option<PathSegment>,
}

impl<'de> MapAccess<'de> for ArenaMapDeserializer<'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            self.pending_key = Some(PathSegment::Key(key.to_string()));
            let key_deser = key.to_string().into_deserializer();
            seed.deserialize(key_deser).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value.take().ok_or_else(|| {
            DeserializeError::from_kind(DeserializeErrorKind::Message(
                "value missing for map entry".into(),
            ))
            .with_path(self.path.clone())
        })?;

        let segment = self
            .pending_key
            .take()
            .unwrap_or_else(|| PathSegment::Key("<unknown>".into()));
        let mut path = self.path.clone();
        path.push(segment);
        seed.deserialize(ArenaValueDeserializer::new(
            super::value_ref::ArenaValueRef::from_value(value),
            path,
        ))
    }
}

pub(crate) struct ArenaStructDeserializer<'de> {
    iter: std::slice::Iter<'de, (&'de str, ArenaValue<'de>)>,
    value: Option<&'de ArenaValue<'de>>,
    allowed: &'static [&'static str],
    seen: HashSet<&'de str>,
    path: Vec<PathSegment>,
    pending_key: Option<PathSegment>,
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
                let mut path = self.path.clone();
                path.push(PathSegment::Key(key_str.to_string()));
                return Err(
                    DeserializeError::from_kind(DeserializeErrorKind::UnknownField {
                        field: key_str.to_string(),
                        expected: format_expected(self.allowed),
                    })
                    .with_path(path),
                );
            }
            if !self.seen.insert(key_str) {
                let mut path = self.path.clone();
                path.push(PathSegment::Key(key_str.to_string()));
                return Err(
                    DeserializeError::from_kind(DeserializeErrorKind::DuplicateField {
                        field: key_str.to_string(),
                    })
                    .with_path(path),
                );
            }
            self.value = Some(value);
            self.pending_key = Some(PathSegment::Key(key_str.to_string()));
            let key_deser = key_str.to_string().into_deserializer();
            seed.deserialize(key_deser).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value.take().ok_or_else(|| {
            DeserializeError::from_kind(DeserializeErrorKind::Message(
                "value missing for struct field".into(),
            ))
            .with_path(self.path.clone())
        })?;

        let segment = self
            .pending_key
            .take()
            .unwrap_or_else(|| PathSegment::Key("<unknown>".into()));
        let mut path = self.path.clone();
        path.push(segment);
        seed.deserialize(ArenaValueDeserializer::new(
            super::value_ref::ArenaValueRef::from_value(value),
            path,
        ))
    }
}

#[cfg(test)]
#[path = "deserializer_test.rs"]
mod deserializer_test;
