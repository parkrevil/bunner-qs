use crate::model::OrderedMap;
use ahash::RandomState;
use indexmap::map::{IntoIter, Iter, IterMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
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
