use crate::config::ParseOptions;
use crate::memory::acquire_bytes;
use crate::nested::pattern_state::{PatternState, acquire_pattern_state};
use crate::parsing::{ParseError, ParseResult};
use memchr::{memchr, memchr2};

use super::arena::{ArenaQueryMap, ParseArena};
use super::key_path::estimate_param_capacity;
use super::pair_decoder::decode_pair;
use super::pair_inserter::insert_pair_arena;
use super::state::ArenaLease;

pub fn with_arena_query_map<R, F>(
    trimmed: &str,
    offset: usize,
    options: &ParseOptions,
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

    parse_segments_into_map(
        arena,
        &mut arena_map,
        &mut pattern_state,
        options,
        trimmed,
        offset,
        bytes,
        &mut pairs,
        decode_scratch.as_mut(),
    )?;

    finalize(arena, &arena_map)
}

fn check_param_limit(limit: Option<usize>, current: usize) -> ParseResult<()> {
    if let Some(limit) = limit
        && current > limit
    {
        Err(ParseError::TooManyParameters {
            limit,
            actual: current,
        })
    } else {
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn process_segment<'arena>(
    arena: &'arena ParseArena,
    arena_map: &mut ArenaQueryMap<'arena>,
    pattern_state: &mut PatternState,
    options: &ParseOptions,
    decode_scratch: &mut Vec<u8>,
    trimmed: &str,
    offset: usize,
    cursor: usize,
    segment_end: usize,
    eq_index: Option<usize>,
) -> ParseResult<()> {
    let raw_key_end = eq_index.unwrap_or(segment_end);
    let raw_key = &trimmed[cursor..raw_key_end];
    let raw_value = eq_index
        .map(|idx| &trimmed[idx + 1..segment_end])
        .unwrap_or("");

    let key_start = offset + cursor;
    let value_offset = eq_index
        .map(|idx| offset + idx + 1)
        .unwrap_or(offset + cursor + raw_key.len());

    let (key, value) = decode_pair(
        raw_key,
        raw_value,
        key_start,
        value_offset,
        options,
        decode_scratch,
    )?;

    insert_pair_arena(
        arena,
        arena_map,
        pattern_state,
        key,
        value,
        options.duplicate_keys,
    )
}

#[allow(clippy::too_many_arguments)]
fn parse_segments_into_map<'arena>(
    arena: &'arena ParseArena,
    arena_map: &mut ArenaQueryMap<'arena>,
    pattern_state: &mut PatternState,
    options: &ParseOptions,
    trimmed: &str,
    offset: usize,
    bytes: &[u8],
    pairs: &mut usize,
    decode_scratch: &mut Vec<u8>,
) -> ParseResult<()> {
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        let mut search = cursor;
        let mut segment_end = bytes.len();
        let mut eq_index: Option<usize> = None;

        while search < bytes.len() {
            let rel = if eq_index.is_some() {
                memchr(b'&', &bytes[search..])
            } else {
                memchr2(b'=', b'&', &bytes[search..])
            };

            let Some(rel) = rel else {
                break;
            };

            let idx = search + rel;
            match bytes[idx] {
                b'=' if eq_index.is_none() => {
                    eq_index = Some(idx);
                    search = idx + 1;
                }
                b'&' => {
                    segment_end = idx;
                    break;
                }
                _ => unreachable!(),
            }
        }

        if segment_end > cursor {
            *pairs += 1;
            check_param_limit(options.max_params, *pairs)?;

            process_segment(
                arena,
                arena_map,
                pattern_state,
                options,
                decode_scratch,
                trimmed,
                offset,
                cursor,
                segment_end,
                eq_index,
            )?;
        }

        cursor = segment_end.saturating_add(1);
    }

    Ok(())
}

#[cfg(test)]
#[path = "builder_test.rs"]
mod builder_test;
