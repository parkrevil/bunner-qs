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
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Error)]
pub enum StringifyError {
    #[error("key contains disallowed control character: '{key}'")]
    InvalidKey { key: String },
    #[error("value for key '{key}' contains disallowed control character")]
    InvalidValue { key: String },
}

pub type StringifyResult<T> = Result<T, StringifyError>;
