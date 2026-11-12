//! # Block-Cache Trait-Abstraktion
//!
//! Zweck: Definiert das `BlockCacheBackend`-Trait fuer polymorphe Cache-Backends.
// ANCHOR[contextra-store:SST-RDR-140]

use bytes::Bytes;

pub trait BlockCacheBackend: Send + Sync {
    /// Creates a new cache backend instance with the specified capacity.
    fn new(capacity: usize) -> Self
    where
        Self: Sized;

    /// Retrieves a cached block by `(file_id, offset)` key.
    fn get(&self, key: &(u64, u64)) -> Option<Bytes>;

    /// Inserts a block into the cache.
    fn insert(&self, key: (u64, u64), value: Bytes);

    /// Returns the number of cached blocks in this backend instance.
    fn len(&self) -> usize;

    /// Returns `true` if the backend contains no cached blocks.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if a key is present in the cache.
    fn contains(&self, key: &(u64, u64)) -> bool {
        self.get(key).is_some()
    }

    /// Returns the current memory usage in bytes.
    fn current_bytes(&self) -> usize;

    /// Returns the capacity limit in bytes.
    fn capacity_bytes(&self) -> usize;
}
