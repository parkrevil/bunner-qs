mod encode;
mod errors;
mod runtime;
mod validate;
mod walker;
mod writer;

use crate::config::StringifyOptions;
use crate::model::QueryMap;
use crate::serde_adapter::{SerdeQueryError, serialize_to_query_map};

pub use self::errors::{SerdeStringifyError, StringifyError};

use serde::Serialize;

pub(crate) use runtime::stringify_query_map_with;

pub type StringifyResult<T> = Result<T, StringifyError>;

pub type SerdeStringifyResult<T> = Result<T, SerdeStringifyError>;

pub fn stringify<T>(data: &T) -> SerdeStringifyResult<String>
where
    T: Serialize,
{
    stringify_with(data, &StringifyOptions::default())
}

pub fn stringify_with<T>(data: &T, options: &StringifyOptions) -> SerdeStringifyResult<String>
where
    T: Serialize,
{
    let map = serialize_to_query_map(data).map_err(SerdeQueryError::from)?;
    let query_map = QueryMap::from(map);
    stringify_query_map_with(&query_map, options).map_err(SerdeStringifyError::from)
}
