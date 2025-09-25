use indexmap::IndexMap;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
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

pub type QueryMap = IndexMap<String, Value>;

#[derive(Debug, Error)]
pub enum SingleValueError {
    #[error("key '{key}' has {count} values; expected exactly one")]
    MultipleValues { key: String, count: usize },
}

pub type SingleValueResult<T> = Result<T, SingleValueError>;

pub fn to_single_map(map: &QueryMap) -> SingleValueResult<HashMap<String, String>> {
    let mut single = HashMap::with_capacity(map.len());

    for (key, value) in map.iter() {
        match value {
            Value::String(s) => {
                single.insert(key.clone(), s.clone());
            }
            Value::Array(arr) if arr.is_empty() => {
                single.insert(key.clone(), String::new());
            }
            Value::Array(arr) if arr.len() == 1 => {
                if let Value::String(s) = &arr[0] {
                    single.insert(key.clone(), s.clone());
                } else {
                    return Err(SingleValueError::MultipleValues {
                        key: key.clone(),
                        count: 1,
                    });
                }
            }
            Value::Array(arr) => {
                return Err(SingleValueError::MultipleValues {
                    key: key.clone(),
                    count: arr.len(),
                });
            }
            Value::Object(_) => {
                return Err(SingleValueError::MultipleValues {
                    key: key.clone(),
                    count: 1,
                });
            }
        }
    }

    Ok(single)
}

pub fn from_single_map<I, K, V>(iter: I) -> QueryMap
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    iter.into_iter()
        .fold(QueryMap::new(), |mut acc, (key, value)| {
            acc.insert(key.into(), Value::String(value.into()));
            acc
        })
}
