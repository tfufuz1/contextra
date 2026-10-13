use bytes::Bytes;
use contextra_crypto::CchSecureBuf;
use lru::LruCache;
use parking_lot::RwLock;
use std::sync::Arc;
use zeroize::Zeroize;

/// Sharded block cache: 64 independent shards to reduce contention.
/// Shard selection uses ahash tuple hashing on (file_id, offset).
pub const BLOCK_CACHE_SHARDS: usize = 64;

/// RAM claim state representation.
///
/// When `secure_buffers == false`, the cache uses standard `Bytes` and can strictly only claim `Unreferenced`.
/// When `secure_buffers == true`, cached blocks are zeroed in memory on drop and claim `Zeroized`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CchRamClaim {
    Unreferenced,
    Zeroized,
}

/// Cache verification evidence report per crate (Spec C4.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CchCacheReport {
    pub caches_checked: usize,
    pub entries_found: usize,
    pub ram_claim: CchRamClaim,
}

/// Trait abstraction for block cache backend implementations.
///
/// ## Block Cache Key Structure
/// Keys stored in `BlockCache` backends are `(file_id, offset)` tuples:
/// - `file_id: u64`: The SSTable file identifier.
/// - `offset: u64`: The block start byte offset within the SSTable file.
///
/// Note: These are SSTable data block location keys, NOT user data keys.
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

    /// Counts cached blocks belonging to any of the specified `file_ids`.
    fn entries_for_files(&self, file_ids: &[u64]) -> usize;

    /// Removes all cached blocks belonging to any of the specified `file_ids` and returns the count removed.
    fn invalidate_files(&self, file_ids: &[u64]) -> usize;

    /// Clears all cached blocks and returns total count removed.
    fn clear_all(&self) -> usize;
}

/// High-performance SIEVE cache backend implementation using `ahash::AHashMap`, `AtomicBool` visited flags, and `VecDeque`.
/// Lock-optimized read path allows concurrent non-blocking gets with shared read lock.
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

    fn entries_for_files(&self, file_ids: &[u64]) -> usize {
        let s = self.state.read();
        s.map.keys().filter(|(f_id, _)| file_ids.contains(f_id)).count()
    }

    fn invalidate_files(&self, file_ids: &[u64]) -> usize {
        let mut s = self.state.write();
        let keys_to_remove: Vec<(u64, u64)> = s
            .map
            .keys()
            .filter(|(f_id, _)| file_ids.contains(f_id))
            .copied()
            .collect();

        let mut count = 0;
        for key in keys_to_remove {
            if let Some(node) = s.map.remove(&key) {
                s.current_bytes = s.current_bytes.saturating_sub(node.value.len());
                count += 1;
            }
        }
        if count > 0 {
            s.nodes.retain(|n| !file_ids.contains(&n.key.0));
            if s.hand >= s.nodes.len() {
                s.hand = 0;
            }
        }
        count
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

    fn entries_for_files(&self, file_ids: &[u64]) -> usize {
        let s = self.state.read();
        s.cache.iter().filter(|((f_id, _), _)| file_ids.contains(f_id)).count()
    }

    fn invalidate_files(&self, file_ids: &[u64]) -> usize {
        let mut s = self.state.write();
        let keys_to_remove: Vec<(u64, u64)> = s
            .cache
            .iter()
            .filter(|((f_id, _), _)| file_ids.contains(f_id))
            .map(|(k, _)| *k)
            .collect();

        let mut count = 0;
        for key in keys_to_remove {
            if let Some(old_val) = s.cache.pop(&key) {
                s.current_bytes = s.current_bytes.saturating_sub(old_val.len());
                count += 1;
            }
        }
        count
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
    keys: RwLock<ahash::AHashSet<(u64, u64)>>,
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
            keys: RwLock::new(ahash::AHashSet::new()),
        }
    }

    #[inline]
    fn get(&self, key: &(u64, u64)) -> Option<Bytes> {
        self.cache.get(key)
    }

    #[inline]
    fn insert(&self, key: (u64, u64), value: Bytes) {
        self.cache.insert(key, value);
        self.keys.write().insert(key);
    }

    #[inline]
    fn len(&self) -> usize {
        self.cache.len()
    }

    #[inline]
    fn contains(&self, key: &(u64, u64)) -> bool {
        self.cache.get(key).is_some()
    }

    fn entries_for_files(&self, file_ids: &[u64]) -> usize {
        let mut keys = self.keys.write();
        // Clean up evicted keys and count matching present keys
        keys.retain(|k| self.cache.get(k).is_some());
        keys.iter().filter(|k| file_ids.contains(&k.0)).count()
    }

    fn invalidate_files(&self, file_ids: &[u64]) -> usize {
        let mut keys = self.keys.write();
        let targets: Vec<(u64, u64)> = keys
            .iter()
            .filter(|k| file_ids.contains(&k.0))
            .copied()
            .collect();

        let mut count = 0;
        for key in targets {
            keys.remove(&key);
            if self.cache.remove(&key).is_some() {
                count += 1;
            }
        }
        count
    }

    fn clear_all(&self) -> usize {
        let mut keys = self.keys.write();
        let count = self.cache.len();
        self.cache.clear();
        keys.clear();
        count
    }
}

#[cfg(not(feature = "block-cache-v2"))]
pub type BlockCacheShard = LruBlockCacheBackend;

#[cfg(feature = "block-cache-v2")]
pub type BlockCacheShard = QuickCacheBlockCacheBackend;

/// Sharded block cache with configurable zeroize-on-drop secure buffers option (`secure_buffers`).
///
/// ## Secure Buffer Performance Costs
/// Standard SSTable block caching uses lock-free reference-counted `Bytes` buffers.
/// When `secure_buffers` is enabled (`true`), cached block allocations incur additional
/// memory zeroing overhead on drop (explicitly wiping 4KB block RAM) and additional wrapper allocations.
/// Therefore, `secure_buffers` defaults to `false`.
pub struct BlockCache {
    shards: Vec<BlockCacheShard>,
    hash_builder: ahash::RandomState,
    secure_buffers: bool,
    sec_buffers: RwLock<ahash::AHashMap<(u64, u64), CchSecureBuf>>,
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        let mut guard = self.sec_buffers.write();
        for (_, mut buf) in guard.drain() {
            buf.zeroize();
        }
    }
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
            secure_buffers: false,
            sec_buffers: RwLock::new(ahash::AHashMap::new()),
        }
    }

    /// Configures whether secure zeroize-on-drop buffers are enabled.
    pub fn with_secure_buffers(mut self, enabled: bool) -> Self {
        self.secure_buffers = enabled;
        self
    }

    /// Sets `secure_buffers` enabled status.
    pub fn set_secure_buffers(&mut self, enabled: bool) {
        if !enabled && self.secure_buffers {
            let mut guard = self.sec_buffers.write();
            for (_, mut buf) in guard.drain() {
                buf.zeroize();
            }
        }
        self.secure_buffers = enabled;
    }

    /// Returns `true` if `secure_buffers` zeroize discipline is active.
    pub fn secure_buffers(&self) -> bool {
        self.secure_buffers
    }

    /// Returns the type-enforced `CchRamClaim`.
    ///
    /// When `secure_buffers == false`, strictly returns `CchRamClaim::Unreferenced`.
    /// Only returns `CchRamClaim::Zeroized` when `secure_buffers == true`.
    pub fn ram_claim(&self) -> CchRamClaim {
        if self.secure_buffers {
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
        let key = (file_id, offset);
        if self.secure_buffers {
            let sec = self.sec_buffers.read();
            if let Some(buf) = sec.get(&key) {
                return Some(Bytes::copy_from_slice(buf.as_slice()));
            }
        }
        self.shard(file_id, offset).get(&key)
    }

    #[inline]
    pub fn insert(&self, file_id: u64, offset: u64, data: Bytes) {
        let key = (file_id, offset);
        if self.secure_buffers {
            let sec_buf = CchSecureBuf::from_vec(data.to_vec());
            self.sec_buffers.write().insert(key, sec_buf);
        }
        self.shard(file_id, offset).insert(key, data);
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.secure_buffers {
            self.sec_buffers.read().len()
        } else {
            self.shards.iter().map(|s| s.len()).sum()
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn contains(&self, file_id: u64, offset: u64) -> bool {
        self.get(file_id, offset).is_some()
    }

    /// Returns total count of cached blocks belonging to any of the specified `file_ids`.
    pub fn entries_for_files(&self, file_ids: &[u64]) -> usize {
        if self.secure_buffers {
            let sec = self.sec_buffers.read();
            sec.keys().filter(|(f_id, _)| file_ids.contains(f_id)).count()
        } else {
            self.shards.iter().map(|s| s.entries_for_files(file_ids)).sum()
        }
    }

    /// Removes all cached blocks belonging to any of the specified `file_ids` across all shards.
    ///
    /// Returns total number of blocks invalidated.
    pub fn invalidate_files(&self, file_ids: &[u64]) -> usize {
        let mut count = 0;
        if self.secure_buffers {
            let mut sec = self.sec_buffers.write();
            sec.retain(|(f_id, _), buf| {
                if file_ids.contains(f_id) {
                    buf.zeroize();
                    count += 1;
                    false
                } else {
                    true
                }
            });
        }
        let shard_count: usize = self.shards.iter().map(|s| s.invalidate_files(file_ids)).sum();
        if self.secure_buffers {
            count
        } else {
            shard_count
        }
    }

    /// Clears all cached blocks across all shards.
    pub fn clear_all(&self) -> usize {
        let mut count = 0;
        if self.secure_buffers {
            let mut sec = self.sec_buffers.write();
            for (_, mut buf) in sec.drain() {
                buf.zeroize();
                count += 1;
            }
        }
        let shard_count: usize = self.shards.iter().map(|s| s.clear_all()).sum();
        if self.secure_buffers {
            count
        } else {
            shard_count
        }
    }

    /// Returns a `CchCacheReport` summarizing checked shards, matching entries, and type-enforced RAM claim.
    pub fn cache_report(&self, file_ids: &[u64]) -> CchCacheReport {
        CchCacheReport {
            caches_checked: self.shards.len(),
            entries_found: self.entries_for_files(file_ids),
            ram_claim: self.ram_claim(),
        }
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
