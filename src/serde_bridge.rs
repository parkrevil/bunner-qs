use crate::arena::ArenaQueryMap;
use crate::serde_impl::{
    DeserializeError, SerializeError, deserialize_from_arena_map, deserialize_from_query_map,
    serialize_to_query_map,
};
use crate::value::QueryMap;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerdeQueryError {
    #[error("failed to serialize values into query map: {0}")]
    Serialize(#[from] SerializeError),
    #[error("failed to deserialize query map: {0}")]
    Deserialize(#[from] DeserializeError),
}

pub(crate) fn to_query_map<T: Serialize>(data: &T) -> Result<QueryMap, SerdeQueryError> {
    let map = serialize_to_query_map(data)?;
    Ok(QueryMap::from(map))
}

#[allow(dead_code)]
pub(crate) fn from_query_map<T: DeserializeOwned>(
    query_map: &QueryMap,
) -> Result<T, SerdeQueryError> {
    Ok(deserialize_from_query_map(query_map)?)
}

pub(crate) fn from_arena_query_map<T: DeserializeOwned>(
    query_map: &ArenaQueryMap<'_>,
) -> Result<T, SerdeQueryError> {
    Ok(deserialize_from_arena_map(query_map)?)
}
