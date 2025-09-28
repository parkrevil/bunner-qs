use ahash::{AHashMap, AHasher};
use smallvec::SmallVec;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, OnceLock, RwLock};

const SHARD_BITS: usize = 4;
const SHARD_COUNT: usize = 1 << SHARD_BITS;
const SHARD_MASK: usize = SHARD_COUNT - 1;
const DEFAULT_SHARD_CAPACITY: usize = 512;

#[derive(Clone, PartialEq, Eq, Hash)]
enum PathAtom {
    Empty,
    Index(u32),
    Text(Arc<str>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct PathKey(SmallVec<[PathAtom; 8]>);

impl PathKey {
    fn from_segments(segments: &[&str]) -> Self {
        let mut atoms = SmallVec::<[PathAtom; 8]>::with_capacity(segments.len());
        for segment in segments {
            atoms.push(segment_to_atom(segment));
        }
        PathKey(atoms)
    }

    fn fingerprint(&self) -> u64 {
        let mut hasher = AHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

fn segment_to_atom(segment: &str) -> PathAtom {
    if segment.is_empty() {
        PathAtom::Empty
    } else if segment.len() <= 9 && segment.as_bytes().iter().all(|b| b.is_ascii_digit()) {
        match segment.parse::<u32>() {
            Ok(value) => PathAtom::Index(value),
            Err(_) => PathAtom::Text(Arc::from(segment)),
        }
    } else {
        PathAtom::Text(Arc::from(segment))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StatsSnapshot {
    pub max: usize,
    pub ema_slow_q8: u32,
    pub ema_fast_q8: u32,
    pub count: u64,
    pub mean: f64,
    pub m2: f64,
}

impl StatsSnapshot {
    pub fn is_empty(&self) -> bool {
        self.max == 0 && self.count == 0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CapacityForecast {
    pub baseline: usize,
    pub aggressive: usize,
    pub burst: usize,
}

impl CapacityForecast {
    pub fn best_effort(&self) -> usize {
        self.aggressive.max(1)
    }
}

#[derive(Debug, Clone, Default)]
struct HintSummary {
    max: usize,
    ema_slow_q8: u32,
    ema_fast_q8: u32,
    count: u64,
    mean: f64,
    m2: f64,
}

impl HintSummary {
    fn update(&mut self, snapshot: &StatsSnapshot) {
        if snapshot.is_empty() {
            return;
        }

        self.max = self.max.max(snapshot.max);
        self.ema_slow_q8 = self.ema_slow_q8.max(snapshot.ema_slow_q8);
        self.ema_fast_q8 = self.ema_fast_q8.max(snapshot.ema_fast_q8);

        if snapshot.count == 0 {
            return;
        }

        let new_total = self.count + snapshot.count;
        if new_total == 0 {
            return;
        }

        let delta = snapshot.mean - self.mean;
        self.mean += delta * (snapshot.count as f64) / (new_total as f64);
        self.m2 += snapshot.m2
            + delta * delta * (self.count as f64) * (snapshot.count as f64) / (new_total as f64);
        self.count = new_total;
    }

    fn forecast(&self) -> Option<CapacityForecast> {
        if self.max == 0 && self.count == 0 {
            return None;
        }

        let avg_slow = ((self.ema_slow_q8 + 0x7F) >> 8) as usize;
        let avg_fast = ((self.ema_fast_q8 + 0x7F) >> 8) as usize;

        let percentile = if self.count > 1 {
            let variance = if self.count > 1 {
                self.m2 / ((self.count - 1) as f64)
            } else {
                0.0
            };
            let std_dev = variance.sqrt();
            (self.mean + 1.64485 * std_dev).max(self.mean).ceil() as usize
        } else {
            avg_fast.max(avg_slow)
        };

        let baseline = avg_slow.max(1);
        let aggressive = percentile.max(avg_fast).max(baseline);
        let burst = self.max.max(aggressive);

        Some(CapacityForecast {
            baseline,
            aggressive,
            burst,
        })
    }
}

struct CatalogEntry {
    summary: HintSummary,
}

struct CatalogShard {
    entries: AHashMap<PathKey, CatalogEntry>,
    order: VecDeque<PathKey>,
    capacity: usize,
}

impl CatalogShard {
    fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            entries: AHashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn touch(&mut self, key: &PathKey) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            let key = self.order.remove(pos).unwrap();
            self.order.push_back(key);
        }
    }

    fn insert(&mut self, key: PathKey, snapshot: &StatsSnapshot) {
        if snapshot.is_empty() {
            return;
        }

        if let Some(entry) = self.entries.get_mut(&key) {
            entry.summary.update(snapshot);
            self.touch(&key);
            return;
        }

        if self.order.len() >= self.capacity
            && let Some(oldest) = self.order.pop_front()
        {
            self.entries.remove(&oldest);
        }

        let mut entry = CatalogEntry {
            summary: HintSummary::default(),
        };
        entry.summary.update(snapshot);
        self.order.push_back(key.clone());
        self.entries.insert(key, entry);
    }

    fn hint_for(&mut self, key: &PathKey) -> Option<CapacityForecast> {
        let hint = self
            .entries
            .get(key)
            .and_then(|entry| entry.summary.forecast());
        if hint.is_some() {
            self.touch(key);
        }
        hint
    }
}

struct PatternCatalog {
    shards: Vec<RwLock<CatalogShard>>,
    mask: usize,
}

impl PatternCatalog {
    fn new(shard_capacity: usize) -> Self {
        let shards = (0..SHARD_COUNT)
            .map(|_| RwLock::new(CatalogShard::new(shard_capacity)))
            .collect();

        Self {
            shards,
            mask: SHARD_MASK,
        }
    }

    fn shard_for(&self, key: &PathKey) -> &RwLock<CatalogShard> {
        let hash = key.fingerprint();
        let idx = (hash as usize) & self.mask;
        &self.shards[idx]
    }

    fn global() -> &'static Self {
        static INSTANCE: OnceLock<PatternCatalog> = OnceLock::new();
        INSTANCE.get_or_init(|| PatternCatalog::new(DEFAULT_SHARD_CAPACITY))
    }

    fn record(&self, path: &[&str], snapshot: &StatsSnapshot) {
        if snapshot.is_empty() {
            return;
        }

        let key = PathKey::from_segments(path);
        let shard = self.shard_for(&key);
        if let Ok(mut guard) = shard.write() {
            guard.insert(key, snapshot);
        }
    }

    fn capacity_hint(&self, path: &[&str]) -> Option<CapacityForecast> {
        let key = PathKey::from_segments(path);
        let shard = self.shard_for(&key);
        shard
            .write()
            .ok()
            .and_then(|mut guard| guard.hint_for(&key))
    }
}

pub fn record_stats(path: &[&str], snapshot: &StatsSnapshot) {
    PatternCatalog::global().record(path, snapshot);
}

pub fn capacity_hint(path: &[&str]) -> Option<CapacityForecast> {
    PatternCatalog::global().capacity_hint(path)
}
