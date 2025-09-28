mod encode;
mod runtime;
mod validate;
mod walker;
mod writer;

use crate::config::StringifyOptions;
use crate::error::{SerdeStringifyError, SerdeStringifyResult};
use crate::serde_adapter::to_query_map;
use serde::Serialize;

pub(crate) use runtime::stringify_query_map_with;

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
    let map = to_query_map(data).map_err(SerdeStringifyError::from)?;
    stringify_query_map_with(&map, options).map_err(SerdeStringifyError::from)
}
