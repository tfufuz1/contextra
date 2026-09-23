use bytes::Bytes;
use lru::LruCache;
use parking_lot::RwLock;
use std::sync::Arc;

/// Sharded block cache: 64 independent shards to reduce contention.
/// Shard selection uses ahash tuple hashing on (file_id, offset).
pub const BLOCK_CACHE_SHARDS: usize = 64;

/// Trait abstraction for block cache backend implementations.
///
/// ## Approximative LRU & Lock-Optimized Read Path
/// SSTable block caches store immutable 4KB data blocks. For point lookups and range scans,
/// read hits occur with high frequency on hot-set blocks.
///
/// While standard `LruCache` maintains strict LRU ordering by updating a doubly-linked list
/// on every access (requiring write lock acquisition even during `get()`), lock-optimized
/// implementations like `quick_cache::sync::Cache` (S3-FIFO / Clock eviction) achieve
/// lock-free read hit evaluation via atomic reference counting without mutating global eviction order on every hit.
///
/// Functionally, strict LRU order is NOT required for correctness in SSTable block caching.
/// Approximative LRU / S3-FIFO eviction provides equivalent or superior hit ratios while
/// eliminating lock contention across concurrent reader threads.
/// SIEVE cache backend implementation using `ahash::AHashMap` and `SieveOrder` hand pointer.
#[cfg(feature = "sieve-cache")]
pub struct SieveCacheBackend {
    state: RwLock<SieveCacheState>,
}

#[allow(dead_code)]
struct SieveNode {
    key: (u64, u64),
    value: Bytes,
    visited: bool,
}

#[allow(dead_code)]
struct SieveCacheState {
    map: ahash::AHashMap<(u64, u64), usize>, // key -> node index in queue
    nodes: Vec<Option<SieveNode>>,
    hand: usize,
    current_bytes: usize,
    capacity_bytes: usize,
}

#[cfg(feature = "sieve-cache")]
impl BlockCacheBackend for SieveCacheBackend {
    fn new(capacity_bytes: usize) -> Self {
        let capacity_bytes = capacity_bytes.max(1);
        Self {
            state: RwLock::new(SieveCacheState {
                map: ahash::AHashMap::new(),
                nodes: Vec::new(),
                hand: 0,
                current_bytes: 0,
                capacity_bytes,
            }),
        }
    }

    #[inline]
    fn get(&self, key: &(u64, u64)) -> Option<Bytes> {
        let mut s = self.state.write();
        if let Some(&idx) = s.map.get(key) {
            if let Some(node) = s.nodes.get_mut(idx).and_then(|n| n.as_mut()) {
                node.visited = true;
                return Some(node.value.clone());
            }
        }
        None
    }

    #[inline]
    fn insert(&self, key: (u64, u64), value: Bytes) {
        let mut s = self.state.write();
        let val_len = value.len();

        // If key exists, update value and mark visited
        if let Some(&idx) = s.map.get(&key) {
            if let Some(Some(node)) = s.nodes.get_mut(idx) {
                let old_len = node.value.len();
                node.value = value;
                node.visited = true;
                s.current_bytes = s.current_bytes.saturating_sub(old_len) + val_len;
                return;
            }
        }

        // Insert new node
        let node_idx = s.nodes.len();
        s.nodes.push(Some(SieveNode {
            key,
            value,
            visited: false,
        }));
        s.map.insert(key, node_idx);
        s.current_bytes += val_len;

        // Evict via SIEVE algorithm if capacity exceeded
        while s.current_bytes > s.capacity_bytes && !s.map.is_empty() {
            let mut evicted = false;
            let len = s.nodes.len();
            if len == 0 {
                break;
            }

            for _ in 0..(len * 2) {
                if s.hand >= s.nodes.len() {
                    s.hand = 0;
                }
                let hand_idx = s.hand;
                let is_visited = match s.nodes.get(hand_idx) {
                    Some(Some(n)) => Some(n.visited),
                    _ => None,
                };

                match is_visited {
                    Some(true) => {
                        if let Some(Some(n)) = s.nodes.get_mut(hand_idx) {
                            n.visited = false;
                        }
                        s.hand += 1;
                    }
                    Some(false) => {
                        if let Some(Some(node)) = s.nodes.get_mut(hand_idx).map(|n| n.take()) {
                            let key_to_remove = node.key;
                            let bytes_to_remove = node.value.len();
                            s.map.remove(&key_to_remove);
                            s.current_bytes = s.current_bytes.saturating_sub(bytes_to_remove);
                            s.hand += 1;
                            evicted = true;
                            break;
                        } else {
                            s.hand += 1;
                        }
                    }
                    None => {
                        s.hand += 1;
                    }
                }
            }

            if !evicted {
                break;
            }
        }

        // Periodic compaction of tombstoned slots in `nodes` if ratio of Nones is high
        if s.nodes.len() > 1024 && s.map.len() * 2 < s.nodes.len() {
            let mut new_nodes = Vec::with_capacity(s.map.len());
            let mut new_map = ahash::AHashMap::with_capacity(s.map.len());
            for node_opt in s.nodes.drain(..) {
                if let Some(node) = node_opt {
                    let new_idx = new_nodes.len();
                    new_map.insert(node.key, new_idx);
                    new_nodes.push(Some(node));
                }
            }
            s.nodes = new_nodes;
            s.map = new_map;
            s.hand = 0;
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.state.read().map.len()
    }
}

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
}

struct LruBlockCacheState {
    cache: LruCache<(u64, u64), Bytes>,
    current_bytes: usize,
    capacity_bytes: usize,
}

/// Standard strict-LRU block cache backend using `parking_lot::RwLock<LruCache>` with byte-based capacity.
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

#[cfg(feature = "block-cache-v2")]
#[derive(Clone)]
struct BlockWeighter;

#[cfg(feature = "block-cache-v2")]
impl quick_cache::Weighter<(u64, u64), Bytes> for BlockWeighter {
    fn weight(&self, _key: &(u64, u64), value: &Bytes) -> u32 {
        value.len().try_into().unwrap_or(u32::MAX)
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
}

#[cfg(not(feature = "block-cache-v2"))]
pub type BlockCacheShard = LruBlockCacheBackend;

#[cfg(feature = "block-cache-v2")]
pub type BlockCacheShard = QuickCacheBlockCacheBackend;

pub struct BlockCache {
    shards: Vec<BlockCacheShard>,
    hash_builder: ahash::RandomState,
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

        Self {
            shards,
            hash_builder: ahash::RandomState::new(),
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
        self.shard(file_id, offset).get(&(file_id, offset))
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
}

/// Binary search for a key index inside a SSTable data block.
///
/// Returns `Ok(Ok(index))` if the exact key is found, or `Ok(Err(index))`
/// indicating the insertion index (lower-bound entry offset index) if not found.
/// Exposed as `pub` for fuzz testing in `memfuse-store-fuzz`.
pub const SSTABLE_MAGIC_MFSX: u32 = 0x5853_464D; // "MFSX" in hex
pub const SSTABLE_MAGIC_LEGACY: u32 = 0x4D46_5354; // "MFST" in hex

/// Creates a new block cache instance with default shard count. Capacity is in MB (assuming 4KB blocks).
pub fn create_block_cache(capacity_mb: usize) -> Arc<BlockCache> {
    create_block_cache_with_shards(capacity_mb, BLOCK_CACHE_SHARDS)
}

/// Creates a new block cache instance with configurable shard count. Capacity is in MB.
pub fn create_block_cache_with_shards(capacity_mb: usize, num_shards: usize) -> Arc<BlockCache> {
    let num_shards = num_shards.max(1);
    let total_bytes = capacity_mb
        .saturating_mul(1024 * 1024)
        .clamp(1024 * 1024, 8 * 1024 * 1024 * 1024);
    let per_shard_bytes = (total_bytes / num_shards).max(64 * 1024);
    Arc::new(BlockCache::new_with_shards(per_shard_bytes, num_shards))
}

/// Block size for SSTable data blocks (4KB).
pub(super) const BLOCK_SIZE: usize = 4096;
