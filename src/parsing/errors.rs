pub use crate::serde_adapter::{DeserializeError, DeserializeErrorKind, PathSegment};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseLocation {
    Query,
    Key,
    Value,
    Parameter,
}

impl ParseLocation {
    fn as_str(self) -> &'static str {
        match self {
            ParseLocation::Query => "query",
            ParseLocation::Key => "key",
            ParseLocation::Value => "value",
            ParseLocation::Parameter => "parameter",
        }
    }
}

impl fmt::Display for ParseLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("input exceeds maximum length of {limit} characters (received {actual})")]
    InputTooLong { limit: usize, actual: usize },
    #[error("too many parameters: received {actual}, limit {limit}")]
    TooManyParameters { limit: usize, actual: usize },
    #[error("duplicate root key '{key}' not allowed")]
    DuplicateRootKey { key: String },
    #[error("duplicate map entry '{segment}' under '{parent}' not allowed")]
    DuplicateMapEntry { parent: String, segment: String },
    #[error("duplicate sequence index {index} under '{parent}' not allowed")]
    DuplicateSequenceIndex { parent: String, index: usize },
    #[error("invalid sequence index '{segment}' under '{parent}' not allowed")]
    InvalidSequenceIndex { parent: String, segment: String },
    #[error("nested value conflict under '{parent}' mixes scalars and structured data")]
    NestedValueConflict { parent: String },
    #[error("incompatible key pattern for segment '{segment}' under '{parent}'")]
    KeyPatternConflict { parent: String, segment: String },
    #[error("invalid percent-encoding in {location} at byte offset {index}")]
    InvalidPercentEncoding {
        index: usize,
        location: ParseLocation,
    },
    #[error("invalid character `{character}` in {location} at byte offset {index}")]
    InvalidCharacter {
        character: char,
        index: usize,
        location: ParseLocation,
    },
    #[error("unexpected '?' character in {location} at byte offset {index}")]
    UnexpectedQuestionMark {
        index: usize,
        location: ParseLocation,
    },
    #[error("unmatched '{bracket}' bracket sequence in key '{key}'")]
    UnmatchedBracket { key: String, bracket: char },
    #[error("maximum bracket depth exceeded in key '{key}' (depth {depth}, limit {limit})")]
    DepthExceeded {
        key: String,
        limit: usize,
        depth: usize,
    },
    #[error("decoded component in {location} is not valid UTF-8")]
    InvalidUtf8 { location: ParseLocation },
    #[error("failed to deserialize parsed query into target type: {0}")]
    Serde(#[from] DeserializeError),
}

#[cfg(test)]
#[path = "errors_test.rs"]
mod errors_test;
