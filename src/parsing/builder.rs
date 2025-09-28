use std::borrow::Cow;

use crate::memory::acquire_bytes;
use crate::nested::{
    insertion::insert_nested_value_arena,
    parse_key_path,
    pattern_state::{PatternState, acquire_pattern_state},
};
use crate::parsing::{ParseError, ParseResult};
use memchr::{memchr, memchr_iter};

use super::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use super::decoder::decode_component;
use super::key_path::{duplicate_key_label, estimate_param_capacity, validate_brackets};
use super::runtime::ParseRuntime;
use super::state::ArenaLease;

pub(crate) fn with_arena_query_map<R, F>(
    trimmed: &str,
    offset: usize,
    runtime: &ParseRuntime,
    finalize: F,
) -> ParseResult<R>
where
    F: for<'arena> FnOnce(&'arena ParseArena, &ArenaQueryMap<'arena>) -> ParseResult<R>,
{
    let arena_capacity = trimmed.len().saturating_mul(2);
    let arena_lease = ArenaLease::acquire(arena_capacity);
    let arena: &ParseArena = &arena_lease;
    let estimated_pairs = estimate_param_capacity(trimmed);
    let mut arena_map = ArenaQueryMap::with_capacity(arena, estimated_pairs);
    let mut pattern_state = acquire_pattern_state();
    let mut pairs = 0usize;
    let mut decode_scratch = acquire_bytes();
    let bytes = trimmed.as_bytes();
    let mut cursor = 0usize;

    for segment_end in memchr_iter(b'&', bytes).chain(std::iter::once(bytes.len())) {
        if segment_end > cursor {
            pairs += 1;
            if let Some(limit) = runtime.max_params
                && pairs > limit
            {
                return Err(ParseError::TooManyParameters {
                    limit,
                    actual: pairs,
                });
            }

            let eq_relative = memchr(b'=', &bytes[cursor..segment_end]);
            let eq_index = eq_relative.map(|rel| cursor + rel);

            let raw_key_end = eq_index.unwrap_or(segment_end);
            let raw_key = &trimmed[cursor..raw_key_end];
            let raw_value = eq_index
                .map(|idx| &trimmed[idx + 1..segment_end])
                .unwrap_or("");

            let key_start = offset + cursor;
            let key = decode_component(
                raw_key,
                runtime.space_as_plus,
                key_start,
                decode_scratch.as_mut(),
            )?;
            validate_brackets(key.as_ref(), runtime.max_depth)?;

            let value_offset = eq_index
                .map(|idx| offset + idx + 1)
                .unwrap_or(offset + cursor + raw_key.len());
            let value = decode_component(
                raw_value,
                runtime.space_as_plus,
                value_offset,
                decode_scratch.as_mut(),
            )?;

            insert_pair_arena(arena, &mut arena_map, &mut pattern_state, key, value)?;
        }

        cursor = segment_end.saturating_add(1);
    }

    finalize(arena, &arena_map)
}

fn insert_pair_arena<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    pattern_state: &mut PatternState,
    key: Cow<'_, str>,
    value: Cow<'_, str>,
) -> ParseResult<()> {
    if !key.is_empty() && !key.contains('[') {
        let key_str = key.as_ref();
        let value_ref = arena.alloc_str(value.as_ref());
        map.try_insert_str(arena, key_str, ArenaValue::string(value_ref))
            .map_err(|_| ParseError::DuplicateKey {
                key: duplicate_key_label(key_str),
            })?;
        return Ok(());
    }

    let key_segments = parse_key_path(key.as_ref());
    let value_ref = arena.alloc_str(value.as_ref());
    insert_nested_value_arena(arena, map, &key_segments, value_ref, pattern_state)
}
