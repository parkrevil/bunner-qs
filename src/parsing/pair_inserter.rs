use std::borrow::Cow;

use crate::config::DuplicateKeyBehavior;
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
    duplicate_keys: DuplicateKeyBehavior,
) -> ParseResult<()> {
    let value_ref = arena.alloc_str(value.as_ref());

    if key.is_empty() {
        return insert_root_value(arena, map, "", value_ref, duplicate_keys);
    }

    if !key.is_empty() && !key.contains('[') {
        let key_str = key.as_ref();
        return insert_root_value(arena, map, key_str, value_ref, duplicate_keys);
    }

    let key_segments = parse_key_path(key.as_ref());
    insert_nested_value_arena(
        arena,
        map,
        &key_segments,
        value_ref,
        pattern_state,
        duplicate_keys,
    )
}

fn insert_root_value<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    key: &str,
    value: &'arena str,
    duplicate_keys: DuplicateKeyBehavior,
) -> ParseResult<()> {
    match map.try_insert_str(arena, key, ArenaValue::string(value)) {
        Ok(()) => Ok(()),
        Err(()) => {
            let existing = map
                .get_mut(key)
                .expect("duplicate key should exist in query map");
            match duplicate_keys {
                DuplicateKeyBehavior::Reject => Err(ParseError::DuplicateKey {
                    key: duplicate_key_label(key),
                }),
                DuplicateKeyBehavior::FirstWins => Ok(()),
                DuplicateKeyBehavior::LastWins => {
                    *existing = ArenaValue::string(value);
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "pair_inserter_test.rs"]
mod pair_inserter_test;
