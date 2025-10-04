use std::ops::{Deref, DerefMut};

use super::arena::{ParseArena, ParseArenaGuard, acquire_parse_arena};

const ARENA_REUSE_UPPER_BOUND: usize = 256 * 1024;

pub(crate) enum ArenaLease {
    Guard(ParseArenaGuard),
    Owned(ParseArena),
}

impl std::fmt::Debug for ArenaLease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArenaLease::Guard(_) => f.write_str("ArenaLease::Guard"),
            ArenaLease::Owned(_) => f.write_str("ArenaLease::Owned"),
        }
    }
}

impl ArenaLease {
    pub(crate) fn acquire(min_capacity: usize) -> Self {
        if min_capacity == 0 {
            return ArenaLease::Guard(acquire_parse_arena(0));
        }

        if min_capacity <= ARENA_REUSE_UPPER_BOUND {
            ArenaLease::Guard(acquire_parse_arena(min_capacity))
        } else {
            ArenaLease::Owned(ParseArena::with_capacity(min_capacity))
        }
    }
}

impl Deref for ArenaLease {
    type Target = ParseArena;

    fn deref(&self) -> &Self::Target {
        match self {
            ArenaLease::Guard(guard) => guard,
            ArenaLease::Owned(arena) => arena,
        }
    }
}

impl DerefMut for ArenaLease {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ArenaLease::Guard(guard) => guard,
            ArenaLease::Owned(arena) => arena,
        }
    }
}

#[cfg(test)]
#[path = "state_test.rs"]
mod state_test;
