use std::any::TypeId;
use std::mem::ManuallyDrop;

use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;

use crate::config::ParseOptions;
use crate::serde_adapter::{arena_map_to_json_value, deserialize_from_arena_map};

use super::builder::with_arena_query_map;
use super::errors::ParseError;
use super::preflight::preflight;

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse<T>(input: impl AsRef<str>, options: &ParseOptions) -> ParseResult<T>
where
    T: DeserializeOwned + Default + 'static,
{
    let raw = input.as_ref();
    let (trimmed, offset) = preflight(raw, options)?;

    if trimmed.is_empty() {
        return Ok(T::default());
    }

    with_arena_query_map(trimmed, offset, options, |_, arena_map| {
        if arena_map.is_empty() {
            Ok(T::default())
        } else {
            if TypeId::of::<T>() == TypeId::of::<JsonValue>() {
                let json_value = arena_map_to_json_value(arena_map);
                let value = unsafe { assume_json_value::<T>(json_value) };
                return Ok(value);
            }
            deserialize_from_arena_map::<T>(arena_map).map_err(ParseError::from)
        }
    })
}

#[inline]
unsafe fn assume_json_value<T>(value: JsonValue) -> T
where
    T: 'static,
{
    debug_assert_eq!(TypeId::of::<T>(), TypeId::of::<JsonValue>());
    let value = ManuallyDrop::new(value);
    let ptr = (&*value) as *const JsonValue as *const T;
    unsafe { ptr.read() }
}

#[cfg(test)]
#[path = "api_test.rs"]
mod api_test;
