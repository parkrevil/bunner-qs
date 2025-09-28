use crate::core::ordered_map::{OrderedMap, new_map, with_capacity};
use crate::serde::{SerdeQueryError, from_query_map, to_query_map};
use indexmap::map::{IntoIter, Iter, IterMut};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    String(String),
    Array(Vec<Value>),
    Object(OrderedMap<String, Value>),
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
pub(crate) struct QueryMap(OrderedMap<String, Value>);

impl QueryMap {
    pub(crate) fn new() -> Self {
        Self(new_map())
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            Self::new()
        } else {
            Self(with_capacity(capacity))
        }
    }
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

impl QueryMap {
    pub(crate) fn from_struct<T>(data: &T) -> Result<Self, SerdeQueryError>
    where
        T: Serialize,
    {
        to_query_map(data)
    }

    #[allow(dead_code)]
    pub(crate) fn to_struct<T>(&self) -> Result<T, SerdeQueryError>
    where
        T: DeserializeOwned + Default,
    {
        if self.is_empty() {
            Ok(T::default())
        } else {
            from_query_map(self)
        }
    }
}
