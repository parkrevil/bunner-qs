use ahash::RandomState;
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use hashbrown::HashMap;
use hashbrown::hash_map::RawEntryMut;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;

pub(crate) struct ParseArena {
    bump: Bump,
    capacity_hint: usize,
}

impl ParseArena {
    pub(crate) fn new() -> Self {
        Self {
            bump: Bump::new(),
            capacity_hint: 0,
        }
    }

    pub(crate) fn with_capacity(bytes: usize) -> Self {
        if bytes == 0 {
            Self::new()
        } else {
            Self {
                bump: Bump::with_capacity(bytes),
                capacity_hint: bytes,
            }
        }
    }

    #[inline]
    pub(crate) fn alloc_str<'arena>(&'arena self, value: &str) -> &'arena str {
        self.bump.alloc_str(value)
    }

    #[inline]
    pub(crate) fn alloc_vec<'arena, T>(&'arena self) -> BumpVec<'arena, T> {
        BumpVec::new_in(&self.bump)
    }

    #[inline]
    pub(crate) fn bump(&self) -> &Bump {
        &self.bump
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        self.bump.reset();
    }

    #[inline]
    pub(crate) fn prepare(&mut self, min_capacity: usize) {
        if min_capacity == 0 {
            self.reset();
        } else if min_capacity > self.capacity_hint {
            *self = ParseArena::with_capacity(min_capacity);
        } else {
            self.reset();
        }
    }
}

impl Default for ParseArena {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static PARSE_ARENA_POOL: RefCell<ParseArena> = RefCell::new(ParseArena::new());
}

pub(crate) struct ParseArenaGuard {
    arena: Option<ParseArena>,
}

impl ParseArenaGuard {
    fn new(mut arena: ParseArena) -> Self {
        arena.reset();
        Self { arena: Some(arena) }
    }
}

impl Deref for ParseArenaGuard {
    type Target = ParseArena;

    fn deref(&self) -> &Self::Target {
        self.arena.as_ref().expect("arena already released")
    }
}

impl DerefMut for ParseArenaGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.arena.as_mut().expect("arena already released")
    }
}

impl Drop for ParseArenaGuard {
    fn drop(&mut self) {
        if let Some(mut arena) = self.arena.take() {
            arena.reset();
            PARSE_ARENA_POOL.with(|cell| {
                *cell.borrow_mut() = arena;
            });
        }
    }
}

pub(crate) fn acquire_parse_arena(min_capacity: usize) -> ParseArenaGuard {
    PARSE_ARENA_POOL.with(|cell| {
        let mut stored = cell.borrow_mut();
        let mut arena = std::mem::take(&mut *stored);
        arena.prepare(min_capacity);
        ParseArenaGuard::new(arena)
    })
}

pub(crate) type ArenaVec<'arena, T> = BumpVec<'arena, T>;

type FastMap<K, V> = HashMap<K, V, RandomState>;

#[inline]
fn shared_random_state() -> RandomState {
    static STATE: OnceLock<RandomState> = OnceLock::new();
    STATE
        .get_or_init(|| RandomState::with_seeds(0x9E37_79B9, 0xB529_7A4D, 0x68E3_1DA4, 0x1B56_3F1B))
        .clone()
}

pub(crate) struct ArenaQueryMap<'arena> {
    entries: ArenaVec<'arena, (&'arena str, ArenaValue<'arena>)>,
    index: FastMap<&'arena str, usize>,
}

impl<'arena> ArenaQueryMap<'arena> {
    pub(crate) fn with_capacity(arena: &'arena ParseArena, capacity: usize) -> Self {
        let mut entries = ArenaVec::new_in(arena.bump());
        if capacity > 0 {
            entries.reserve(capacity);
        }
        let index = if capacity > 0 {
            FastMap::with_capacity_and_hasher(capacity, shared_random_state())
        } else {
            FastMap::with_capacity_and_hasher(0, shared_random_state())
        };

        Self { entries, index }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&'arena str, &ArenaValue<'arena>)> {
        self.entries.iter().map(|(k, v)| (*k, v))
    }

    pub(crate) fn contains_key(&self, key: &str) -> bool {
        self.index.contains_key(key)
    }

    pub(crate) fn get_mut(&mut self, key: &str) -> Option<&mut ArenaValue<'arena>> {
        self.index
            .get(key)
            .copied()
            .map(|idx| &mut self.entries[idx].1)
    }

    pub(crate) fn try_insert_str(
        &mut self,
        arena: &'arena ParseArena,
        key: &str,
        value: ArenaValue<'arena>,
    ) -> Result<(), ()> {
        match self.index.raw_entry_mut().from_key(key) {
            RawEntryMut::Occupied(_) => Err(()),
            RawEntryMut::Vacant(vacant) => {
                let key_ref = arena.alloc_str(key);
                let idx = self.entries.len();
                self.entries.push((key_ref, value));
                vacant.insert(key_ref, idx);
                Ok(())
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn entries_slice(&self) -> &[(&'arena str, ArenaValue<'arena>)] {
        self.entries.as_slice()
    }
}

pub(crate) enum ArenaValue<'arena> {
    String(&'arena str),
    Seq(ArenaVec<'arena, ArenaValue<'arena>>),
    Map {
        entries: ArenaVec<'arena, (&'arena str, ArenaValue<'arena>)>,
        index: FastMap<&'arena str, usize>,
    },
}

impl<'arena> ArenaValue<'arena> {
    pub(crate) fn string(value: &'arena str) -> Self {
        ArenaValue::String(value)
    }

    pub(crate) fn map(arena: &'arena ParseArena) -> Self {
        ArenaValue::Map {
            entries: ArenaVec::new_in(arena.bump()),
            index: FastMap::with_capacity_and_hasher(0, shared_random_state()),
        }
    }

    pub(crate) fn map_with_capacity(arena: &'arena ParseArena, capacity: usize) -> Self {
        if capacity <= 4 {
            return ArenaValue::map(arena);
        }
        let mut entries = ArenaVec::new_in(arena.bump());
        entries.reserve(capacity);
        ArenaValue::Map {
            entries,
            index: FastMap::with_capacity_and_hasher(capacity, shared_random_state()),
        }
    }

    pub(crate) fn seq_with_capacity(arena: &'arena ParseArena, capacity: usize) -> Self {
        let mut values = arena.alloc_vec();
        if capacity > 4 {
            values.reserve(capacity);
        }
        ArenaValue::Seq(values)
    }

    pub(crate) fn as_seq_slice(&self) -> Option<&[ArenaValue<'arena>]> {
        match self {
            ArenaValue::Seq(items) => Some(items.as_slice()),
            _ => None,
        }
    }

    pub(crate) fn as_map_slice(&self) -> Option<&[(&'arena str, ArenaValue<'arena>)]> {
        match self {
            ArenaValue::Map { entries, .. } => Some(entries.as_slice()),
            _ => None,
        }
    }
}
