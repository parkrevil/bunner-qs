#![allow(dead_code)]

use ahash::AHashMap;
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;

use crate::value::{QueryMap, Value};

/// Arena allocator used during parsing to reduce per-node allocations.
#[derive(Default)]
pub(crate) struct ParseArena {
    bump: Bump,
}

impl ParseArena {
    pub(crate) fn new() -> Self {
        Self { bump: Bump::new() }
    }

    pub(crate) fn with_capacity(bytes: usize) -> Self {
        Self {
            bump: Bump::with_capacity(bytes),
        }
    }

    #[inline]
    pub(crate) fn alloc_str<'arena>(&'arena self, value: &str) -> &'arena str {
        self.bump.alloc_str(value)
    }

    #[inline]
    pub(crate) fn alloc_slice_copy<'arena, T: Copy>(&'arena self, slice: &[T]) -> &'arena [T] {
        self.bump.alloc_slice_copy(slice)
    }

    #[inline]
    pub(crate) fn alloc_vec<'arena, T>(&'arena self) -> BumpVec<'arena, T> {
        BumpVec::new_in(&self.bump)
    }

    #[inline]
    pub(crate) fn bump(&self) -> &Bump {
        &self.bump
    }
}

pub(crate) type ArenaVec<'arena, T> = BumpVec<'arena, T>;

pub(crate) struct ArenaQueryMap<'arena> {
    entries: ArenaVec<'arena, (&'arena str, ArenaValue<'arena>)>,
    index: AHashMap<&'arena str, usize>,
}

impl<'arena> ArenaQueryMap<'arena> {
    pub(crate) fn new(arena: &'arena ParseArena) -> Self {
        Self {
            entries: ArenaVec::new_in(arena.bump()),
            index: AHashMap::new(),
        }
    }

    pub(crate) fn with_capacity(arena: &'arena ParseArena, capacity: usize) -> Self {
        let mut entries = ArenaVec::new_in(arena.bump());
        if capacity > 0 {
            entries.reserve(capacity);
        }
        let index = if capacity > 0 {
            AHashMap::with_capacity(capacity)
        } else {
            AHashMap::new()
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
        if self.contains_key(key) {
            return Err(());
        }

        let key_ref = arena.alloc_str(key);
        self.push_allocated(key_ref, value);
        Ok(())
    }

    pub(crate) fn push_allocated(&mut self, key: &'arena str, value: ArenaValue<'arena>) {
        let idx = self.entries.len();
        self.entries.push((key, value));
        self.index.insert(key, idx);
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn to_owned(&self) -> QueryMap {
        let mut map = QueryMap::with_capacity(self.entries.len());
        for (key, value) in self.entries.iter() {
            map.insert((*key).to_string(), value.to_owned());
        }
        map
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
        index: AHashMap<&'arena str, usize>,
    },
}

impl<'arena> ArenaValue<'arena> {
    pub(crate) fn string(value: &'arena str) -> Self {
        ArenaValue::String(value)
    }

    pub(crate) fn seq(values: ArenaVec<'arena, ArenaValue<'arena>>) -> Self {
        ArenaValue::Seq(values)
    }

    pub(crate) fn map(arena: &'arena ParseArena) -> Self {
        ArenaValue::Map {
            entries: ArenaVec::new_in(arena.bump()),
            index: AHashMap::new(),
        }
    }

    pub(crate) fn map_with_capacity(arena: &'arena ParseArena, capacity: usize) -> Self {
        let mut entries = ArenaVec::new_in(arena.bump());
        entries.reserve(capacity);
        ArenaValue::Map {
            entries,
            index: AHashMap::with_capacity(capacity),
        }
    }

    pub(crate) fn to_owned(&self) -> Value {
        match self {
            ArenaValue::String(s) => Value::String((*s).to_string()),
            ArenaValue::Seq(items) => {
                let owned = items.iter().map(|item| item.to_owned()).collect();
                Value::Array(owned)
            }
            ArenaValue::Map { entries, .. } => {
                let mut map = QueryMap::with_capacity(entries.len());
                for (key, value) in entries.iter() {
                    map.insert((*key).to_string(), value.to_owned());
                }
                Value::Object(map.into())
            }
        }
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
