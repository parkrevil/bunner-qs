mod builder;
mod decoder;
mod direct;
mod key_path;
mod preflight;
mod runtime;
mod state;

pub(crate) mod arena;

use std::any::TypeId;
use std::mem::ManuallyDrop;

use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;

use crate::config::ParseOptions;
use crate::error::{ParseError, ParseResult};
use crate::model::QueryMap;
use crate::serde_adapter::{arena_map_to_json_value, from_arena_query_map};

use builder::with_arena_query_map;
use direct::try_parse_direct;
use preflight::preflight;
use runtime::ParseRuntime;

pub fn parse<T>(input: impl AsRef<str>) -> ParseResult<T>
where
    T: DeserializeOwned + Default + 'static,
{
    parse_with(input, &ParseOptions::default())
}

pub fn parse_with<T>(input: impl AsRef<str>, options: &ParseOptions) -> ParseResult<T>
where
    T: DeserializeOwned + Default + 'static,
{
    let raw = input.as_ref();
    let runtime = ParseRuntime::new(options);
    let (trimmed, offset) = preflight(raw, &runtime)?;

    if trimmed.is_empty() {
        return Ok(T::default());
    }

    if let Some(result) = try_parse_direct(trimmed, &runtime) {
        return result;
    }

    with_arena_query_map(trimmed, offset, &runtime, |_, arena_map| {
        if arena_map.len() == 0 {
            Ok(T::default())
        } else {
            if runtime.serde_fastpath && TypeId::of::<T>() == TypeId::of::<JsonValue>() {
                let json_value = arena_map_to_json_value(arena_map);
                let json_value = ManuallyDrop::new(json_value);
                let ptr = (&*json_value) as *const JsonValue as *const T;
                // SAFETY: TypeId equality guarantees T is exactly JsonValue.
                let value = unsafe { ptr.read() };
                return Ok(value);
            }
            from_arena_query_map::<T>(arena_map).map_err(ParseError::from)
        }
    })
}

#[allow(dead_code)]
pub(crate) fn parse_query_map(input: &str, options: &ParseOptions) -> ParseResult<QueryMap> {
    let runtime = ParseRuntime::new(options);
    let (trimmed, offset) = preflight(input, &runtime)?;

    if trimmed.is_empty() {
        return Ok(QueryMap::new());
    }

    with_arena_query_map(trimmed, offset, &runtime, |_, arena_map| {
        Ok(arena_map.to_owned())
    })
}
