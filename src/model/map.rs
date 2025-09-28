use ahash::RandomState;
use indexmap::IndexMap;

pub type OrderedMap<K, V> = IndexMap<K, V, RandomState>;
