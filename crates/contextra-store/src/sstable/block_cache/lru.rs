//! # LRU-Cache Backend
//!
//! Zweck: Standardmäßiges Least-Recently-Used Block-Cache-Backend.
// ANCHOR[contextra-store:SST-RDR-142]

use super::traits::BlockCacheBackend;
use bytes::Bytes;
use lru::LruCache;
use parking_lot::RwLock;

struct LruBlockCacheState {
    cache: LruCache<(u64, u64), Bytes>,
    current_bytes: usize,
    capacity_bytes: usize,
}

pub struct LruBlockCacheBackend {
    state: RwLock<LruBlockCacheState>,
}

impl BlockCacheBackend for LruBlockCacheBackend {
    fn new(capacity_bytes: usize) -> Self {
        let capacity_bytes = capacity_bytes.max(1);
        Self {
            state: RwLock::new(LruBlockCacheState {
                cache: LruCache::unbounded(),
                current_bytes: 0,
                capacity_bytes,
            }),
        }
    }

    #[inline]
    fn current_bytes(&self) -> usize {
        self.state.read().current_bytes
    }

    #[inline]
    fn capacity_bytes(&self) -> usize {
        self.state.read().capacity_bytes
    }

    #[inline]
    fn get(&self, key: &(u64, u64)) -> Option<Bytes> {
        self.state.write().cache.get(key).cloned()
    }

    #[inline]
    fn insert(&self, key: (u64, u64), value: Bytes) {
        let mut s = self.state.write();
        let val_len = value.len();
        if let Some(old_val) = s.cache.put(key, value) {
            s.current_bytes = s.current_bytes.saturating_sub(old_val.len());
        }
        s.current_bytes += val_len;

        while s.current_bytes > s.capacity_bytes && !s.cache.is_empty() {
            if let Some((_k, popped)) = s.cache.pop_lru() {
                s.current_bytes = s.current_bytes.saturating_sub(popped.len());
            } else {
                break;
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.state.read().cache.len()
    }

    #[inline]
    fn contains(&self, key: &(u64, u64)) -> bool {
        self.state.read().cache.contains(key)
    }
}
