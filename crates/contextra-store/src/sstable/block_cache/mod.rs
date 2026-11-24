//! # Geshardeter Block-Cache (Ring 1)
//!
//! Zweck: Zentraler Block-Cache für dekomprimierte SSTable-Datenblöcke mit Sharding.
// ANCHOR[contextra-store:SST-RDR-144]

mod lru;
mod quick;
mod sieve;
mod traits;

pub use lru::LruBlockCacheBackend;
#[cfg(feature = "block-cache-v2")]
pub use quick::QuickCacheBlockCacheBackend;
pub use sieve::SieveCacheBackend;
pub use traits::BlockCacheBackend;

pub use super::format::{BLOCK_SIZE, SSTABLE_MAGIC_LEGACY, SSTABLE_MAGIC_MFSX};

use bytes::Bytes;
use std::sync::Arc;

/// Sharded block cache: 64 independent shards to reduce contention.
/// Shard selection uses ahash tuple hashing on (file_id, offset).
pub const BLOCK_CACHE_SHARDS: usize = 64;

#[cfg(not(feature = "block-cache-v2"))]
pub type BlockCacheShard = LruBlockCacheBackend;

#[cfg(feature = "block-cache-v2")]
pub type BlockCacheShard = QuickCacheBlockCacheBackend;

pub struct BlockCache {
    shards: Vec<BlockCacheShard>,
    hash_builder: ahash::RandomState,
    hits: std::sync::atomic::AtomicU64,
    misses: std::sync::atomic::AtomicU64,
}

impl BlockCache {
    pub fn new(capacity_per_shard: usize) -> Self {
        Self::new_with_shards(capacity_per_shard, BLOCK_CACHE_SHARDS)
    }

    pub fn new_with_shards(capacity_per_shard: usize, num_shards: usize) -> Self {
        let num_shards = num_shards.max(1);
        let cap_shard = capacity_per_shard.max(1);

        let shards = (0..num_shards)
            .map(|_| BlockCacheShard::new(cap_shard))
            .collect();

        // P28 / B11: Deterministic documented seeds for BlockCache shard hashing.
        // Golden ratio & Pi fractional hex constants provide strong avalanche distribution
        // across shards without non-deterministic entropy initialization.
        const BLOCK_CACHE_SEED_0: u64 = 0x9e37_79b9_7f4a_7c15;
        const BLOCK_CACHE_SEED_1: u64 = 0x243f_6a88_85a3_08d3;
        const BLOCK_CACHE_SEED_2: u64 = 0x1319_8a2e_0370_7344;
        const BLOCK_CACHE_SEED_3: u64 = 0xa409_3822_299f_31d0;

        Self {
            shards,
            hash_builder: ahash::RandomState::with_seeds(
                BLOCK_CACHE_SEED_0,
                BLOCK_CACHE_SEED_1,
                BLOCK_CACHE_SEED_2,
                BLOCK_CACHE_SEED_3,
            ),
            hits: std::sync::atomic::AtomicU64::new(0),
            misses: std::sync::atomic::AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn shard_idx(&self, file_id: u64, offset: u64) -> usize {
        let hash = self.hash_builder.hash_one((file_id, offset));
        (hash as usize) % self.shards.len()
    }

    #[inline]
    fn shard(&self, file_id: u64, offset: u64) -> &BlockCacheShard {
        let idx = self.shard_idx(file_id, offset);
        &self.shards[idx]
    }

    #[inline]
    pub fn get(&self, file_id: u64, offset: u64) -> Option<Bytes> {
        let res = self.shard(file_id, offset).get(&(file_id, offset));
        if res.is_some() {
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.misses
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        res
    }

    #[inline]
    pub fn insert(&self, file_id: u64, offset: u64, data: Bytes) {
        self.shard(file_id, offset).insert((file_id, offset), data);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.shards.iter().map(|s| s.len()).sum()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn contains(&self, file_id: u64, offset: u64) -> bool {
        self.shard(file_id, offset).contains(&(file_id, offset))
    }

    /// Returns the total number of cache hits recorded.
    #[inline]
    pub fn hits(&self) -> u64 {
        self.hits.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns the total number of cache misses recorded.
    #[inline]
    pub fn misses(&self) -> u64 {
        self.misses.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns the cache hit ratio (0.0 .. 1.0).
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.hits() as f64;
        let total = hits + self.misses() as f64;
        if total == 0.0 {
            0.0
        } else {
            hits / total
        }
    }

    /// Resets hit and miss metrics counters.
    pub fn reset_metrics(&self) {
        self.hits.store(0, std::sync::atomic::Ordering::Relaxed);
        self.misses.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Returns current memory usage in bytes across all shards.
    pub fn current_bytes(&self) -> usize {
        self.shards.iter().map(|s| s.current_bytes()).sum()
    }

    /// Returns total configured byte capacity across all shards.
    pub fn total_capacity_bytes(&self) -> usize {
        self.shards.iter().map(|s| s.capacity_bytes()).sum()
    }
}

/// Binary search for a key index inside a SSTable data block.
///
/// Returns `Ok(Ok(index))` if the exact key is found, or `Ok(Err(index))`
/// indicating the insertion index (lower-bound entry offset index) if not found.
/// Exposed as `pub` for fuzz testing in `contextra-store-fuzz`.

/// Creates a new block cache instance with default shard count. Capacity is in MB (assuming 4KB blocks).
pub fn create_block_cache(capacity_mb: usize) -> Arc<BlockCache> {
    create_block_cache_with_shards(capacity_mb, BLOCK_CACHE_SHARDS)
}

/// Creates a new block cache instance with configurable shard count. Capacity is in MB.
pub fn create_block_cache_with_shards(capacity_mb: usize, num_shards: usize) -> Arc<BlockCache> {
    let num_shards = num_shards.max(1);
    // Enforce a minimum total block cache floor of 512 MB to ensure realistic working set coverage
    // and prevent severe cache thrashing on production / benchmark random-read workloads.
    let total_bytes = capacity_mb
        .saturating_mul(1024 * 1024)
        .clamp(512 * 1024 * 1024, 8 * 1024 * 1024 * 1024);
    let per_shard_bytes = (total_bytes / num_shards).max(64 * 1024);
    Arc::new(BlockCache::new_with_shards(per_shard_bytes, num_shards))
}
