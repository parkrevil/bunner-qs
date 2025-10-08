use crate::model::OrderedMap;
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use crate::serde_adapter::{
    DeserializeError, DeserializeErrorKind, SerializeError, deserialize_from_arena_map,
    serialize_to_query_map,
};
use ahash::RandomState;
use indexmap::map::{IntoIter, Iter, IterMut};
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Array(Vec<Value>),
    Object(OrderedMap<String, Value>),
}

impl Value {
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items.as_slice()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_object(&self) -> Option<&OrderedMap<String, Value>> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QueryMap(OrderedMap<String, Value>);

impl QueryMap {
    pub fn new() -> Self {
        Self(OrderedMap::with_hasher(RandomState::default()))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            Self::new()
        } else {
            Self(OrderedMap::with_capacity_and_hasher(
                capacity,
                RandomState::default(),
            ))
        }
    }

    pub fn to_struct<T>(&self) -> Result<T, DeserializeError>
    where
        T: DeserializeOwned,
    {
        let arena = ParseArena::new();
        let mut arena_map = ArenaQueryMap::with_capacity(&arena, self.len());

        for (key, value) in self.iter() {
            insert_value_into_arena_map(&arena, &mut arena_map, key, value)?;
        }

        deserialize_from_arena_map(&arena_map)
    }

    pub fn from_struct<T>(value: &T) -> Result<Self, SerializeError>
    where
        T: Serialize,
    {
        let map = serialize_to_query_map(value)?;
        Ok(QueryMap::from(map))
    }
}

pub(crate) fn clone_value_into_arena<'arena>(
    arena: &'arena ParseArena,
    value: &Value,
) -> ArenaValue<'arena> {
    match value {
        Value::String(text) => ArenaValue::string(arena.alloc_str(text)),
        Value::Array(items) => {
            let mut seq = arena.alloc_vec();
            if items.len() > 4 {
                seq.reserve(items.len());
            }
            for item in items {
                seq.push(clone_value_into_arena(arena, item));
            }
            ArenaValue::Seq(seq)
        }
        Value::Object(map) => {
            let mut object = if map.is_empty() {
                ArenaValue::map(arena)
            } else {
                ArenaValue::map_with_capacity(arena, map.len())
            };

            let (entries, index) = object
                .map_parts_mut()
                .expect("ArenaValue::map should produce map variant");

            for (key, child) in map.iter() {
                let key_ref = arena.alloc_str(key);
                let idx = entries.len();
                entries.push((key_ref, clone_value_into_arena(arena, child)));
                index.insert(key_ref, idx);
            }

            object
        }
    }
}

fn insert_value_into_arena_map<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    key: &str,
    value: &Value,
) -> Result<(), DeserializeError> {
    let arena_value = clone_value_into_arena(arena, value);
    map.try_insert_str(arena, key, arena_value)
        .map_err(|()| duplicate_field_error(key))
}

fn duplicate_field_error(key: &str) -> DeserializeError {
    DeserializeError::from_kind(DeserializeErrorKind::DuplicateField {
        field: key.to_string(),
    })
}

impl std::ops::Deref for QueryMap {
    type Target = OrderedMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for QueryMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<OrderedMap<String, Value>> for QueryMap {
    fn from(map: OrderedMap<String, Value>) -> Self {
        Self(map)
    }
}

impl From<QueryMap> for OrderedMap<String, Value> {
    fn from(map: QueryMap) -> Self {
        map.0
    }
}

impl<K, V> FromIterator<(K, V)> for QueryMap
where
    K: Into<String>,
    V: Into<Value>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = QueryMap::new();
        for (key, value) in iter {
            map.insert(key.into(), value.into());
        }
        map
    }
}

impl IntoIterator for QueryMap {
    type Item = (String, Value);
    type IntoIter = IntoIter<String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a QueryMap {
    type Item = (&'a String, &'a Value);
    type IntoIter = Iter<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut QueryMap {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = IterMut<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[cfg(test)]
pub(crate) use clone_value_into_arena as clone_value_into_arena_for_test;

#[cfg(test)]
#[path = "value_test.rs"]
mod value_test;
