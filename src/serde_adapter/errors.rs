use serde::de;
use serde::ser;
use std::fmt::{self, Display};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

struct PathDisplay<'a>(&'a [PathSegment]);

impl<'a> fmt::Display for PathDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for segment in self.0 {
            match segment {
                PathSegment::Key(key) => {
                    if first {
                        write!(f, "{key}")?;
                        first = false;
                    } else {
                        write!(f, ".{key}")?;
                    }
                }
                PathSegment::Index(index) => {
                    write!(f, "[{index}]")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone)]
pub enum DeserializeErrorKind {
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

#[derive(Debug, Clone)]
pub struct DeserializeError {
    path: Vec<PathSegment>,
    kind: DeserializeErrorKind,
}

impl DeserializeError {
    pub fn from_kind(kind: DeserializeErrorKind) -> Self {
        Self {
            path: Vec::new(),
            kind,
        }
    }

    pub fn with_path(mut self, path: Vec<PathSegment>) -> Self {
        if self.path.is_empty() {
            self.path = path;
        }
        self
    }

    pub fn push_segment(mut self, segment: PathSegment) -> Self {
        self.path.push(segment);
        self
    }

    pub fn kind(&self) -> &DeserializeErrorKind {
        &self.kind
    }

    pub fn path(&self) -> &[PathSegment] {
        &self.path
    }
}

impl From<DeserializeErrorKind> for DeserializeError {
    fn from(kind: DeserializeErrorKind) -> Self {
        DeserializeError::from_kind(kind)
    }
}

impl Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{} at {}", self.kind, PathDisplay(&self.path))
        }
    }
}

impl std::error::Error for DeserializeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

impl de::Error for DeserializeError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserializeErrorKind::Message(msg.to_string()).into()
    }
}

#[cfg(test)]
#[path = "errors_test.rs"]
mod errors_test;
