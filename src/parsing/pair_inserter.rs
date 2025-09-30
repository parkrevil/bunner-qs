use std::borrow::Cow;

use crate::nested::pattern_state::PatternState;
use crate::nested::{insertion::insert_nested_value_arena, parse_key_path};
use crate::parsing::{ParseError, ParseResult};

use super::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use super::key_path::duplicate_key_label;

pub(crate) fn insert_pair_arena<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    pattern_state: &mut PatternState,
    key: Cow<'_, str>,
    value: Cow<'_, str>,
) -> ParseResult<()> {
    if key.is_empty() {
        let value_ref = arena.alloc_str(value.as_ref());
        map.try_insert_str(arena, "", ArenaValue::string(value_ref))
            .map_err(|_| ParseError::DuplicateKey {
                key: duplicate_key_label(""),
            })?;
        return Ok(());
    }

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

#[cfg(test)]
#[path = "pair_inserter_test.rs"]
mod pair_inserter_test;
