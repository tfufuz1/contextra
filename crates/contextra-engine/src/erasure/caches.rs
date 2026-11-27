//! RAM-cache handling for erasure (Spezifikation §7 "Caches", §9a).
//!
//! The engine itself holds no per-document RAM caches today (checked: no `*Cache` type in
//! this crate; `kv_locks` are sharded mutexes without per-key state; the deferred hyperedge
//! queue holds edge ids and its write-back goes through the gated store). Embedders and
//! future caches use these two helpers:
//!
//! * [`EpochCache`] — entries carry the gate epoch they were created under; an entry from an
//!   older epoch is a miss, so a cache can never serve data across a gate publication.
//! * [`ObserverFn`] — turns a closure into an [`ErasureObserver`] for eager eviction.

use contextra_ports::observer::{ErasureEvent, ErasureObserver, ObserverError};
use contextra_ports::BoxFuture;
use parking_lot::Mutex;
use std::collections::BTreeMap;

/// Closure-backed [`ErasureObserver`]. The closure must be fast and non-blocking.
pub struct ObserverFn<F>(pub F);

impl<F> ErasureObserver for ObserverFn<F>
where
    F: Fn(&ErasureEvent) + Send + Sync + 'static,
{
    fn on_erased(&self, ev: ErasureEvent) -> BoxFuture<'static, Result<(), ObserverError>> {
        (self.0)(&ev);
        Box::pin(async { Ok(()) })
    }
}

/// Cache whose entries are only valid for the gate epoch they were inserted under.
pub struct EpochCache<K: Ord, V: Clone> {
    map: Mutex<BTreeMap<K, (u64, V)>>,
}

impl<K: Ord, V: Clone> Default for EpochCache<K, V> {
    fn default() -> Self {
        Self {
            map: Mutex::new(BTreeMap::new()),
        }
    }
}

impl<K: Ord, V: Clone> EpochCache<K, V> {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the value only if it was inserted under `current_epoch`; stale entries are
    /// dropped and reported as a miss.
    pub fn get(&self, key: &K, current_epoch: u64) -> Option<V> {
        let mut map = self.map.lock();
        match map.get(key) {
            Some((epoch, v)) if *epoch == current_epoch => Some(v.clone()),
            Some(_) => {
                map.remove(key);
                None
            }
            None => None,
        }
    }

    /// Inserts a value created under `epoch`.
    pub fn insert(&self, key: K, epoch: u64, value: V) {
        self.map.lock().insert(key, (epoch, value));
    }

    /// Evicts everything (called on `Gated`/`Completed`).
    pub fn clear(&self) {
        self.map.lock().clear();
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.map.lock().len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.map.lock().is_empty()
    }
}

impl<K, V> ErasureObserver for EpochCache<K, V>
where
    K: Ord + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn on_erased(&self, _ev: ErasureEvent) -> BoxFuture<'static, Result<(), ObserverError>> {
        self.clear();
        Box::pin(async { Ok(()) })
    }
}
