use thiserror::Error;
use crate::serde_adapter::SerdeQueryError;

#[derive(Debug, Error)]
pub enum StringifyError {
    #[error("key contains disallowed control character: '{key}'")]
    InvalidKey { key: String },
    #[error("value for key '{key}' contains disallowed control character")]
    InvalidValue { key: String },
}

#[derive(Debug, Error)]
pub enum SerdeStringifyError {
    #[error(transparent)]
    Serialize(#[from] SerdeQueryError),
    #[error(transparent)]
    Stringify(#[from] StringifyError),
}
