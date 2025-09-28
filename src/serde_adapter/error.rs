use super::{DeserializeError, SerializeError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerdeQueryError {
    #[error("failed to serialize values into query map: {0}")]
    Serialize(#[from] SerializeError),
    #[error("failed to deserialize query map: {0}")]
    Deserialize(#[from] DeserializeError),
}
