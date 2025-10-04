use crate::serde_adapter::SerdeAdapterError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StringifyError {
    #[error(transparent)]
    Serialize(#[from] SerdeAdapterError),
    #[error("key contains disallowed control character: '{key}'")]
    InvalidKey { key: String },
    #[error("value for key '{key}' contains disallowed control character")]
    InvalidValue { key: String },
}
