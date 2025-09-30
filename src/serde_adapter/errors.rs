use serde::de;
use serde::ser;
use std::fmt::Display;
use thiserror::Error;

pub(crate) fn format_expected(fields: &'static [&'static str]) -> String {
    if fields.is_empty() {
        "(none)".into()
    } else {
        fields.join(", ")
    }
}

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

#[derive(Debug, Error)]
pub enum SerdeQueryError {
    #[error("failed to serialize values into query map: {0}")]
    Serialize(#[from] SerializeError),
    #[error("failed to deserialize query map: {0}")]
    Deserialize(#[from] DeserializeError),
}

#[cfg(test)]
#[path = "errors_test.rs"]
mod errors_test;
