use crate::parsing::arena::{ArenaValue, ParseArena};
use crate::ParseError;

use super::segment::ContainerType;

pub(crate) fn arena_initial_container<'arena>(
    arena: &'arena ParseArena,
    container_type: ContainerType,
    capacity_hint: usize,
) -> ArenaValue<'arena> {
    match container_type {
        ContainerType::Array => ArenaValue::seq_with_capacity(arena, capacity_hint),
        ContainerType::Object => ArenaValue::map_with_capacity(arena, capacity_hint),
    }
}

pub(crate) fn arena_ensure_container<'arena>(
    arena: &'arena ParseArena,
    value: &mut ArenaValue<'arena>,
    expected: ContainerType,
    root_key: &str,
) -> Result<(), ParseError> {
    match expected {
        ContainerType::Array => match value {
            ArenaValue::Seq(_) => Ok(()),
            ArenaValue::Map { .. } => {
                *value = ArenaValue::seq_with_capacity(arena, 0);
                Ok(())
            }
            ArenaValue::String(_) => Err(ParseError::DuplicateKey {
                key: root_key.to_string(),
            }),
        },
        ContainerType::Object => match value {
            ArenaValue::Map { .. } => Ok(()),
            ArenaValue::Seq(_) => {
                *value = ArenaValue::map_with_capacity(arena, 0);
                Ok(())
            }
            ArenaValue::String(_) => Err(ParseError::DuplicateKey {
                key: root_key.to_string(),
            }),
        },
    }
}