use indexmap::IndexMap;
use std::fmt;
use std::time::SystemTime;

#[derive(Clone, PartialEq)]
pub enum QsValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<QsValue>),
    Object(IndexMap<String, QsValue>),
    Bytes(Vec<u8>),
    Date(SystemTime),
    Regex(String),
    Custom(String),
}

impl fmt::Debug for QsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QsValue::Null => write!(f, "Null"),
            QsValue::Bool(b) => write!(f, "Bool({b})"),
            QsValue::Number(n) => write!(f, "Number({n})"),
            QsValue::String(s) => write!(f, "String({s:?})"),
            QsValue::Array(values) => f.debug_list().entries(values).finish(),
            QsValue::Object(map) => map.fmt(f),
            QsValue::Bytes(bytes) => write!(f, "Bytes({bytes:?})"),
            QsValue::Date(dt) => write!(f, "Date({:?})", dt),
            QsValue::Regex(pattern) => write!(f, "Regex({pattern:?})"),
            QsValue::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}

impl From<&str> for QsValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for QsValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for QsValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for QsValue {
    fn from(value: i64) -> Self {
        Self::Number(value as f64)
    }
}

impl From<f64> for QsValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<Vec<QsValue>> for QsValue {
    fn from(values: Vec<QsValue>) -> Self {
        Self::Array(values)
    }
}

impl From<IndexMap<String, QsValue>> for QsValue {
    fn from(values: IndexMap<String, QsValue>) -> Self {
        Self::Object(values)
    }
}
