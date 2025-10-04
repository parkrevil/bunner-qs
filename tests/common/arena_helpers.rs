use crate::parsing::arena::{ArenaQueryMap, ParseArena};

pub fn map_with_capacity<'arena>(
    arena: &'arena ParseArena,
    capacity: usize,
) -> ArenaQueryMap<'arena> {
    ArenaQueryMap::with_capacity(arena, capacity)
}

pub fn map<'arena>(arena: &'arena ParseArena) -> ArenaQueryMap<'arena> {
    map_with_capacity(arena, 0)
}

pub fn alloc_key<'arena>(arena: &'arena ParseArena, key: &str) -> &'arena str {
    arena.alloc_str(key)
}
