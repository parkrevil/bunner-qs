use super::{SerdeQueryError, deserialize_from_query_map};
use crate::model::QueryMap;
use serde::de::DeserializeOwned;

pub fn from_query_map<T: DeserializeOwned>(query_map: &QueryMap) -> Result<T, SerdeQueryError> {
    deserialize_from_query_map(query_map).map_err(SerdeQueryError::from)
}
