//! # QuickCache Backend
//!
//! Zweck: Hochperformantes Block-Cache-Backend hinter Feature `block-cache-v2`.
// ANCHOR[contextra-store:SST-RDR-143]

use super::traits::BlockCacheBackend;
use bytes::Bytes;

#[cfg(feature = "block-cache-v2")]
#[derive(Clone)]
struct BlockWeighter;

#[cfg(feature = "block-cache-v2")]
impl quick_cache::Weighter<(u64, u64), Bytes> for BlockWeighter {
    fn weight(&self, _key: &(u64, u64), value: &Bytes) -> u64 {
        value.len() as u64
    }
}

/// Lock-optimized block cache backend using `quick_cache::sync::Cache` (S3-FIFO / Clock-based eviction) with byte-based capacity.
#[cfg(feature = "block-cache-v2")]
pub struct QuickCacheBlockCacheBackend {
    cache: quick_cache::sync::Cache<(u64, u64), Bytes, BlockWeighter>,
}

#[cfg(feature = "block-cache-v2")]
impl BlockCacheBackend for QuickCacheBlockCacheBackend {
    fn new(capacity_bytes: usize) -> Self {
        let estimated_items = (capacity_bytes / 4096).max(16);
        Self {
            cache: quick_cache::sync::Cache::with_weighter(
                estimated_items,
                capacity_bytes as u64,
                BlockWeighter,
            ),
        }
    }

    #[inline]
    fn get(&self, key: &(u64, u64)) -> Option<Bytes> {
        self.cache.get(key)
    }

    #[inline]
    fn insert(&self, key: (u64, u64), value: Bytes) {
        self.cache.insert(key, value);
    }

    #[inline]
    fn len(&self) -> usize {
        self.cache.len()
    }

    #[inline]
    fn contains(&self, key: &(u64, u64)) -> bool {
        self.cache.get(key).is_some()
    }

    #[inline]
    fn current_bytes(&self) -> usize {
        self.cache.weight() as usize
    }

    #[inline]
    fn capacity_bytes(&self) -> usize {
        self.cache.capacity() as usize
    }
}
