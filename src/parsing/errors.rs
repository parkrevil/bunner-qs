use crate::serde_adapter::SerdeQueryError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("input exceeds maximum length of {limit} characters")]
    InputTooLong { limit: usize },
    #[error("too many parameters: received {actual}, limit {limit}")]
    TooManyParameters { limit: usize, actual: usize },
    #[error("duplicate key '{key}' not allowed")]
    DuplicateKey { key: String },
    #[error("invalid percent-encoding at byte offset {index}")]
    InvalidPercentEncoding { index: usize },
    #[error("query contains invalid character `{character}` at byte offset {index}")]
    InvalidCharacter { character: char, index: usize },
    #[error("unexpected '?' character inside query at byte offset {index}")]
    UnexpectedQuestionMark { index: usize },
    #[error("unmatched bracket sequence in key '{key}'")]
    UnmatchedBracket { key: String },
    #[error("maximum bracket depth exceeded in key '{key}' (limit {limit})")]
    DepthExceeded { key: String, limit: usize },
    #[error("decoded component is not valid UTF-8")]
    InvalidUtf8,
    #[error("failed to deserialize parsed query into target type: {0}")]
    Serde(#[from] SerdeQueryError),
}
