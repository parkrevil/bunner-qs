mod builder;
mod decoder;
mod errors;
mod key_path;
mod preflight;
mod state;

pub(crate) mod arena;

pub use self::errors::ParseError;

use std::any::TypeId;
use std::mem::ManuallyDrop;

use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;

use crate::config::ParseOptions;
use crate::serde_adapter::{SerdeQueryError, arena_map_to_json_value, deserialize_from_arena_map};

use builder::with_arena_query_map;
use preflight::preflight;

pub type ParseResult<T> = Result<T, ParseError>;

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
    let (trimmed, offset) = preflight(raw, options)?;

    if trimmed.is_empty() {
        return Ok(T::default());
    }

    with_arena_query_map(trimmed, offset, options, |_, arena_map| {
        if arena_map.len() == 0 {
            Ok(T::default())
        } else {
            if TypeId::of::<T>() == TypeId::of::<JsonValue>() {
                let json_value = arena_map_to_json_value(arena_map);
                let json_value = ManuallyDrop::new(json_value);
                let ptr = (&*json_value) as *const JsonValue as *const T;
                // SAFETY: TypeId equality guarantees T is exactly JsonValue.
                let value = unsafe { ptr.read() };
                return Ok(value);
            }
            deserialize_from_arena_map::<T>(arena_map)
                .map_err(SerdeQueryError::Deserialize)
                .map_err(ParseError::from)
        }
    })
}
