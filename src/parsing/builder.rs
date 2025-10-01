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

struct ParseContext<'arena, 'options, 'map, 'pattern, 'scratch> {
    arena: &'arena ParseArena,
    arena_map: &'map mut ArenaQueryMap<'arena>,
    pattern_state: &'pattern mut PatternState,
    options: &'options ParseOptions,
    trimmed: &'options str,
    offset: usize,
    decode_scratch: &'scratch mut Vec<u8>,
    pairs: usize,
}

impl<'arena, 'options, 'map, 'pattern, 'scratch>
    ParseContext<'arena, 'options, 'map, 'pattern, 'scratch>
{
    fn increment_pairs(&mut self) -> ParseResult<()> {
        self.pairs = self.pairs.saturating_add(1);
        check_param_limit(self.options.max_params, self.pairs)
    }

    fn process_segment(
        &mut self,
        cursor: usize,
        segment_end: usize,
        eq_index: Option<usize>,
    ) -> ParseResult<()> {
        let trimmed = self.trimmed;
        let raw_key_end = eq_index.unwrap_or(segment_end);
        let raw_key = &trimmed[cursor..raw_key_end];
        let raw_value = eq_index
            .map(|idx| &trimmed[idx + 1..segment_end])
            .unwrap_or("");

        let key_start = self.offset + cursor;
        let value_offset = eq_index
            .map(|idx| self.offset + idx + 1)
            .unwrap_or(self.offset + cursor + raw_key.len());

        let (key, value) = decode_pair(
            raw_key,
            raw_value,
            key_start,
            value_offset,
            self.options,
            self.decode_scratch,
        )?;

        insert_pair_arena(
            self.arena,
            self.arena_map,
            self.pattern_state,
            key,
            value,
            self.options.duplicate_keys,
        )
    }
}

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
    let mut decode_scratch = acquire_bytes();
    let bytes = trimmed.as_bytes();
    let mut context = ParseContext {
        arena,
        arena_map: &mut arena_map,
        pattern_state: &mut pattern_state,
        options,
        trimmed,
        offset,
        decode_scratch: decode_scratch.as_mut(),
        pairs: 0,
    };

    parse_segments_into_map(&mut context, bytes)?;

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

fn parse_segments_into_map(
    context: &mut ParseContext<'_, '_, '_, '_, '_>,
    bytes: &[u8],
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
            context.increment_pairs()?;
            context.process_segment(cursor, segment_end, eq_index)?;
        }

        cursor = segment_end.saturating_add(1);
    }

    Ok(())
}

#[cfg(test)]
#[path = "builder_test.rs"]
mod builder_test;
