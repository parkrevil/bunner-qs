use crate::{QueryMap, parse, stringify};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerdeQueryError {
    #[error("failed to serialize values into query map: {0}")]
    Serialize(#[from] serde_urlencoded::ser::Error),
    #[error("failed to deserialize query map: {0}")]
    Deserialize(#[from] serde_urlencoded::de::Error),
    #[error(transparent)]
    Stringify(#[from] crate::StringifyError),
    #[error(transparent)]
    Parse(#[from] crate::ParseError),
}

/// Convert a serde-serializable struct to a QueryMap
pub(crate) fn to_query_map<T: Serialize>(data: &T) -> Result<QueryMap, SerdeQueryError> {
    let query_string = serde_urlencoded::to_string(data)?;
    Ok(parse(&query_string)?)
}

/// Convert a QueryMap to a serde-deserializable struct
pub(crate) fn from_query_map<T: DeserializeOwned>(
    query_map: &QueryMap,
) -> Result<T, SerdeQueryError> {
    let query_string = stringify(query_map)?;
    Ok(serde_urlencoded::from_str(&query_string)?)
}
