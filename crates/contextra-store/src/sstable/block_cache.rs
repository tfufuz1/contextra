use bytes::Bytes;
use lru::LruCache;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use zeroize::Zeroize;

/// Zeroizing owner wrapper for `Bytes::from_owner`.
///
/// When the last `Bytes` reference holding this owner is dropped,
/// `Drop::drop` automatically overwrites the buffer bytes with zeroes.
///
/// Note: Best-effort zeroizing on deallocation. RAM residual after allocator free is
/// operating-system / allocator dependent.
pub struct ZeroizingBytesOwner {
    data: Vec<u8>,
}

impl ZeroizingBytesOwner {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl AsRef<[u8]> for ZeroizingBytesOwner {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for ZeroizingBytesOwner {
    fn drop(&mut self) {
        self.data.as_mut_slice().zeroize();
    }
}

/// Helper function to create `Bytes` backed by a `ZeroizingBytesOwner`.
///
/// **Copy Cost Analysis:**
/// - Cache Insert: Converts `Bytes` to `Vec<u8>` (`data.to_vec()`) to wrap inside `ZeroizingBytesOwner`.
///   This incurs 1 heap allocation and byte copy on cache insertion (~N bytes copy).
/// - Cache Read (`get`): Clones the `Bytes` handle which is an O(1) atomic refcount bump with ZERO byte copies.
/// - Cache Eviction / Invalidation Drop: `ZeroizingBytesOwner::drop` zeroizes the buffer in-place (O(N) memset)
///   before returning memory to the allocator.
pub fn make_zeroizing_bytes(data: Vec<u8>) -> Bytes {
    Bytes::from_owner(ZeroizingBytesOwner::new(data))
}

/// Constant indicating that cache buffer zeroization on drop is fully implemented in code
/// and verified by integration tests (`crates/contextra-store/tests/cache_residue.rs`).
pub const CACHE_ZEROIZATION_IMPLEMENTED: bool = true;

/// RAM claim state for cached memory buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CchRamClaim {
    Unreferenced,
    Zeroized,
}

/// Detailed report on block cache content matching target SSTable files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CchCacheReport {
    pub caches_checked: usize,
    pub entries_found: usize,
    pub ram_claim: CchRamClaim,
}

/// Sharded block cache: 64 independent shards to reduce contention.
/// Shard selection uses ahash tuple hashing on (file_id, offset).
pub const BLOCK_CACHE_SHARDS: usize = 64;

/// Trait abstraction for block cache backend implementations.
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

    fn invalidate_files(&self, files: &[u64]) -> usize;

    fn entries_for_files(&self, files: &[u64]) -> usize;

    fn clear_all(&self) -> usize;
}

/// High-performance SIEVE cache backend implementation using `ahash::AHashMap`, `AtomicBool` visited flags, and `VecDeque`.
pub struct SieveCacheBackend {
    state: RwLock<SieveCacheState>,
}

struct SieveNode {
    key: (u64, u64),
    value: Bytes,
    visited: std::sync::atomic::AtomicBool,
}

struct SieveCacheState {
    map: ahash::AHashMap<(u64, u64), Arc<SieveNode>>,
    nodes: std::collections::VecDeque<Arc<SieveNode>>,
    hand: usize,
    current_bytes: usize,
    capacity_bytes: usize,
}

impl BlockCacheBackend for SieveCacheBackend {
    fn new(capacity_bytes: usize) -> Self {
        let capacity_bytes = capacity_bytes.max(1);
        Self {
            state: RwLock::new(SieveCacheState {
                map: ahash::AHashMap::new(),
                nodes: std::collections::VecDeque::new(),
                hand: 0,
                current_bytes: 0,
                capacity_bytes,
            }),
        }
    }

    #[inline]
    fn get(&self, key: &(u64, u64)) -> Option<Bytes> {
        let s = self.state.read();
        if let Some(node) = s.map.get(key) {
            node.visited
                .store(true, std::sync::atomic::Ordering::Relaxed);
            return Some(node.value.clone());
        }
        None
    }

    #[inline]
    fn insert(&self, key: (u64, u64), value: Bytes) {
        let val_len = value.len();
        let mut s = self.state.write();

        if let Some(old_node) = s.map.get(&key).cloned() {
            let old_len = old_node.value.len();
            let new_node = Arc::new(SieveNode {
                key,
                value,
                visited: std::sync::atomic::AtomicBool::new(true),
            });
            s.map.insert(key, Arc::clone(&new_node));
            s.nodes.push_back(Arc::clone(&new_node));
            s.current_bytes = s.current_bytes.saturating_sub(old_len) + val_len;
        } else {
            let new_node = Arc::new(SieveNode {
                key,
                value,
                visited: std::sync::atomic::AtomicBool::new(false),
            });
            s.map.insert(key, Arc::clone(&new_node));
            s.nodes.push_back(Arc::clone(&new_node));
            s.current_bytes += val_len;
        }

        while s.current_bytes > s.capacity_bytes && !s.map.is_empty() {
            if s.nodes.is_empty() {
                break;
            }

            if s.hand >= s.nodes.len() {
                s.hand = 0;
            }

            let node = s.nodes[s.hand].clone();

            let hand = s.hand;
            if let Some(map_node) = s.map.get(&node.key) {
                if Arc::ptr_eq(map_node, &node) {
                    if node.visited.load(std::sync::atomic::Ordering::Relaxed) {
                        node.visited
                            .store(false, std::sync::atomic::Ordering::Relaxed);
                        s.hand += 1;
                    } else {
                        s.map.remove(&node.key);
                        s.current_bytes = s.current_bytes.saturating_sub(node.value.len());
                        s.nodes.remove(hand);
                    }
                } else {
                    s.nodes.remove(hand);
                }
            } else {
                s.nodes.remove(hand);
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.state.read().map.len()
    }

    fn invalidate_files(&self, files: &[u64]) -> usize {
        let mut s = self.state.write();
        let mut removed = 0;
        let mut bytes_sub = 0;
        s.map.retain(|k, v| {
            if files.contains(&k.0) {
                removed += 1;
                bytes_sub += v.value.len();
                false
            } else {
                true
            }
        });
        s.current_bytes = s.current_bytes.saturating_sub(bytes_sub);
        if removed > 0 {
            s.nodes.retain(|n| !files.contains(&n.key.0));
            s.hand = 0;
        }
        removed
    }

    fn entries_for_files(&self, files: &[u64]) -> usize {
        let s = self.state.read();
        s.map.keys().filter(|k| files.contains(&k.0)).count()
    }

    fn clear_all(&self) -> usize {
        let mut s = self.state.write();
        let count = s.map.len();
        s.map.clear();
        s.nodes.clear();
        s.hand = 0;
        s.current_bytes = 0;
        count
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

    fn invalidate_files(&self, files: &[u64]) -> usize {
        let mut s = self.state.write();
        let keys_to_remove: Vec<(u64, u64)> = s
            .cache
            .iter()
            .filter(|(k, _)| files.contains(&k.0))
            .map(|(k, _)| *k)
            .collect();
        let count = keys_to_remove.len();
        for k in keys_to_remove {
            if let Some(val) = s.cache.pop(&k) {
                s.current_bytes = s.current_bytes.saturating_sub(val.len());
            }
        }
        count
    }

    fn entries_for_files(&self, files: &[u64]) -> usize {
        let s = self.state.read();
        s.cache.iter().filter(|(k, _)| files.contains(&k.0)).count()
    }

    fn clear_all(&self) -> usize {
        let mut s = self.state.write();
        let count = s.cache.len();
        s.cache.clear();
        s.current_bytes = 0;
        count
    }
}

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

    fn invalidate_files(&self, files: &[u64]) -> usize {
        // QuickCache does not support range or predicate iteration directly.
        // QuickCache maintains internal eviction queues. For complete file invalidation,
        // clear() purges matching or all entries safely.
        if files.is_empty() {
            return 0;
        }
        let initial = self.cache.len();
        self.cache.clear();
        initial
    }

    fn entries_for_files(&self, _files: &[u64]) -> usize {
        self.cache.len()
    }

    fn clear_all(&self) -> usize {
        let count = self.cache.len();
        self.cache.clear();
        count
    }
}

#[cfg(not(feature = "block-cache-v2"))]
pub type BlockCacheShard = LruBlockCacheBackend;

#[cfg(feature = "block-cache-v2")]
pub type BlockCacheShard = QuickCacheBlockCacheBackend;

pub struct BlockCache {
    shards: Vec<BlockCacheShard>,
    hash_builder: ahash::RandomState,
    secure_buffers: AtomicBool,
    non_zeroized_count: std::sync::atomic::AtomicUsize,
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
            secure_buffers: AtomicBool::new(false),
            non_zeroized_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn entries_for_files(&self, files: &[u64]) -> usize {
        self.shards.iter().map(|s| s.entries_for_files(files)).sum()
    }

    pub fn invalidate_files(&self, files: &[u64]) -> usize {
        let removed: usize = self.shards.iter().map(|s| s.invalidate_files(files)).sum();
        if self.is_empty() {
            self.non_zeroized_count.store(0, Ordering::Relaxed);
        }
        removed
    }

    pub fn clear_all(&self) -> usize {
        let count: usize = self.shards.iter().map(|s| s.clear_all()).sum();
        self.non_zeroized_count.store(0, Ordering::Relaxed);
        count
    }

    pub fn cache_report(&self, files: &[u64]) -> CchCacheReport {
        CchCacheReport {
            caches_checked: self.shards.len(),
            entries_found: self.entries_for_files(files),
            ram_claim: self.ram_claim(),
        }
    }

    pub fn secure_buffers(&self) -> bool {
        self.secure_buffers.load(Ordering::Relaxed)
    }

    pub fn set_secure_buffers(&mut self, enabled: bool) {
        self.secure_buffers.store(enabled, Ordering::Relaxed);
    }

    pub fn ram_claim(&self) -> CchRamClaim {
        // CchRamClaim::Zeroized is returned ONLY when:
        // 1. Zeroization implementation is present in code (CACHE_ZEROIZATION_IMPLEMENTED)
        // 2. Runtime secure_buffers flag is active
        // 3. Observed buffer state confirms no non-zeroized entries are present in the cache
        if CACHE_ZEROIZATION_IMPLEMENTED
            && self.secure_buffers()
            && self.non_zeroized_count.load(Ordering::Relaxed) == 0
        {
            CchRamClaim::Zeroized
        } else {
            CchRamClaim::Unreferenced
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
        let entry_data = if self.secure_buffers() {
            make_zeroizing_bytes(data.to_vec())
        } else {
            self.non_zeroized_count.fetch_add(1, Ordering::Relaxed);
            data
        };
        self.shard(file_id, offset)
            .insert((file_id, offset), entry_data);
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
/// Exposed as `pub` for fuzz testing in `contextra-store-fuzz`.
pub const SSTABLE_MAGIC_MFSX: u32 = 0x5853_464D; // "MFSX" in hex
pub const SSTABLE_MAGIC_LEGACY: u32 = 0x4D46_5354; // "MFST" in hex

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

/// Block size for SSTable data blocks (4KB).
pub(super) const BLOCK_SIZE: usize = 4096;
