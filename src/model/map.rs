use ahash::RandomState;
use indexmap::IndexMap;

pub type OrderedMap<K, V> = IndexMap<K, V, RandomState>;

pub fn new_map<K, V>() -> OrderedMap<K, V> {
    OrderedMap::with_hasher(RandomState::default())
}

pub fn with_capacity<K, V>(capacity: usize) -> OrderedMap<K, V> {
    OrderedMap::with_capacity_and_hasher(capacity, RandomState::default())
}
