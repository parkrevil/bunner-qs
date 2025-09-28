use ahash::RandomState;
use indexmap::IndexMap;

pub(crate) type OrderedMap<K, V> = IndexMap<K, V, RandomState>;

pub(crate) fn new_map<K, V>() -> OrderedMap<K, V> {
    OrderedMap::with_hasher(RandomState::default())
}

pub(crate) fn with_capacity<K, V>(capacity: usize) -> OrderedMap<K, V> {
    OrderedMap::with_capacity_and_hasher(capacity, RandomState::default())
}
