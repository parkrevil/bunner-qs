use indexmap::IndexMap;

use crate::{StringifyOptions, StringifyResult};

#[cfg(feature = "serde")]
use crate::serde_bridge::{SerdeQueryError, from_query_map, to_query_map};
#[cfg(feature = "serde")]
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Array(Vec<Value>),
    Object(IndexMap<String, Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&IndexMap<String, Value>> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

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
pub struct QueryMap(IndexMap<String, Value>);

impl QueryMap {
    pub fn new() -> Self {
        Self(IndexMap::new())
    }
}

impl std::ops::Deref for QueryMap {
    type Target = IndexMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for QueryMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<IndexMap<String, Value>> for QueryMap {
    fn from(map: IndexMap<String, Value>) -> Self {
        Self(map)
    }
}

impl From<QueryMap> for IndexMap<String, Value> {
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
    type IntoIter = indexmap::map::IntoIter<String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a QueryMap {
    type Item = (&'a String, &'a Value);
    type IntoIter = indexmap::map::Iter<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut QueryMap {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = indexmap::map::IterMut<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl QueryMap {
    pub fn to_string(&self) -> StringifyResult<String> {
        crate::stringify::stringify(self)
    }

    pub fn to_string_with(&self, options: &StringifyOptions) -> StringifyResult<String> {
        crate::stringify::stringify_with(self, options)
    }

    #[cfg(feature = "serde")]
    pub fn from_struct<T>(data: &T) -> Result<Self, SerdeQueryError>
    where
        T: Serialize,
    {
        to_query_map(data)
    }

    #[cfg(feature = "serde")]
    pub fn to_struct<T>(&self) -> Result<T, SerdeQueryError>
    where
        T: DeserializeOwned + Default,
    {
        if self.is_empty() {
            Ok(T::default())
        } else {
            from_query_map(self)
        }
    }

    #[cfg(all(feature = "serde", feature = "serde_json"))]
    pub fn to_json(&self) -> Result<serde_json::Value, SerdeQueryError> {
        fn convert_value(value: &Value) -> serde_json::Value {
            match value {
                Value::String(s) => string_to_json_value(s),
                Value::Array(items) => {
                    serde_json::Value::Array(items.iter().map(convert_value).collect())
                }
                Value::Object(map) => serde_json::Value::Object(convert_object(map)),
            }
        }

        fn convert_object(
            map: &IndexMap<String, Value>,
        ) -> serde_json::Map<String, serde_json::Value> {
            map.iter()
                .map(|(key, value)| (key.clone(), convert_value(value)))
                .collect()
        }

        fn string_to_json_value(s: &str) -> serde_json::Value {
            match s {
                "true" => serde_json::Value::Bool(true),
                "false" => serde_json::Value::Bool(false),
                "null" => serde_json::Value::Null,
                _ => {
                    if let Ok(int) = s.parse::<i64>() {
                        serde_json::Value::Number(int.into())
                    } else if let Ok(uint) = s.parse::<u64>() {
                        serde_json::Value::Number(uint.into())
                    } else if let Ok(float) = s.parse::<f64>() {
                        match serde_json::Number::from_f64(float) {
                            Some(number) => serde_json::Value::Number(number),
                            None => serde_json::Value::String(s.to_string()),
                        }
                    } else {
                        serde_json::Value::String(s.to_string())
                    }
                }
            }
        }

        Ok(serde_json::Value::Object(convert_object(&self.0)))
    }
}
