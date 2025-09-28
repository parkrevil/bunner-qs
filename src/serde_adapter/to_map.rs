use super::{SerdeQueryError, serialize_to_query_map};
use crate::model::QueryMap;
use serde::Serialize;

pub fn to_query_map<T: Serialize>(data: &T) -> Result<QueryMap, SerdeQueryError> {
    let map = serialize_to_query_map(data)?;
    Ok(QueryMap::from(map))
}
