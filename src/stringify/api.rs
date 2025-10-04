use crate::config::StringifyOptions;
use crate::model::QueryMap;
use crate::serde_adapter::{SerdeAdapterError, serialize_to_query_map};
use serde::Serialize;

use super::runtime::stringify_query_map_with;
use super::{StringifyError, StringifyResult};

pub fn stringify<T>(data: &T) -> StringifyResult<String>
where
    T: Serialize,
{
    stringify_with(data, &StringifyOptions::default())
}

pub fn stringify_with<T>(data: &T, options: &StringifyOptions) -> StringifyResult<String>
where
    T: Serialize,
{
    let map = serialize_to_query_map(data)
        .map_err(SerdeAdapterError::from)
        .map_err(StringifyError::from)?;
    let query_map = QueryMap::from(map);
    stringify_query_map_with(&query_map, options)
}

#[cfg(test)]
#[path = "api_test.rs"]
mod api_test;
