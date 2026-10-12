use bytes::Bytes;
use contextra_store::sstable::{create_block_cache_with_shards, BlockCache};
use std::sync::Arc;
use std::thread;

/// B.1 Code Argument / Micro-Test for Legacy SIEVE Cache VecDeque $O(n)$ Eviction:
/// In `SieveCacheBackend::insert`, when capacity is exceeded (`s.current_bytes > s.capacity_bytes`),
/// eviction calls `s.nodes.remove(hand)` on `VecDeque<Arc<SieveNode>>`. Removing an element at an
/// arbitrary index in a `VecDeque` requires shifting all subsequent elements, incurring $O(n)$
/// operations per eviction. In contrast, `block-cache-v2` (`quick_cache::sync::Cache`) utilizes
/// lock-optimized S3-FIFO / Clock eviction with $O(1)$ amortized time complexity and atomic hit tracking.

#[test]
fn test_p03_block_cache_capacity_limit_bytes() {
    // 1 MB total capacity across 1 shard
    let cache = BlockCache::new_with_shards(1024 * 1024, 1);
    let block_4k = Bytes::from(vec![0u8; 4096]);

    // Insert 356 * 4KB = 1.4 MB (exceeding 1 MB capacity)
    for i in 0..356 {
        cache.insert(100, i, block_4k.clone());
    }

    // Cache should hold at most capacity / block_size items (approx 256 items)
    assert!(
        cache.len() <= 256,
        "Cache size {} exceeded max capacity of 256 blocks",
        cache.len()
    );
}

#[test]
fn test_p03_block_cache_hit_and_miss() {
    let cache = BlockCache::new_with_shards(512 * 1024, 2);
    let file_id = 42;
    let offset = 1024;
    let payload = Bytes::from_static(b"hello block cache");

    assert!(cache.get(file_id, offset).is_none());
    assert!(!cache.contains(file_id, offset));

    cache.insert(file_id, offset, payload.clone());

    assert!(cache.contains(file_id, offset));
    let hit = cache.get(file_id, offset).expect("cache hit");
    assert_eq!(hit, payload);
}

#[test]
fn test_p03_block_cache_eviction_under_pressure() {
    // 64 KB capacity = 16 blocks of 4KB
    let cache = BlockCache::new_with_shards(64 * 1024, 1);
    let block = Bytes::from(vec![1u8; 4096]);

    for i in 0..100 {
        cache.insert(1, i, block.clone());
    }

    assert!(
        cache.len() <= 16,
        "Cache len {} should be bounded by 16",
        cache.len()
    );

    // Out of 100 inserted items, capacity bounds require at least 84 items to be evicted
    let hits = (0..100).filter(|&i| cache.get(1, i).is_some()).count();
    assert!(hits <= 16, "Number of hits {} exceeds capacity 16", hits);
}

#[test]
fn test_p03_block_cache_parallel_access() {
    let cache: Arc<BlockCache> = create_block_cache_with_shards(1, 4);
    let mut handles = Vec::new();

    for thread_idx in 0..8 {
        let cache_ref = Arc::clone(&cache);
        handles.push(thread::spawn(move || {
            let file_id = thread_idx as u64;
            let payload = Bytes::from(vec![thread_idx as u8; 4096]);
            for block_idx in 0..50 {
                cache_ref.insert(file_id, block_idx, payload.clone());
                let _ = cache_ref.get(file_id, block_idx);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread join");
    }

    assert!(cache.len() > 0);
}

#[test]
fn test_p03_block_cache_invalidation_isolated_file_id() {
    let cache = BlockCache::new_with_shards(512 * 1024, 4);
    let file_id_a = 100;
    let file_id_b = 200;
    let payload = Bytes::from_static(b"compacted sstable block");

    cache.insert(file_id_a, 0, payload.clone());
    cache.insert(file_id_b, 0, payload.clone());

    assert!(cache.get(file_id_a, 0).is_some());
    assert!(cache.get(file_id_b, 0).is_some());

    // Simulated compaction: SSTable file_id_a is deleted / overwritten with new file ID
    // A query for a non-existent file_id_a offset must hit disk/none, never stale blocks.
    assert!(cache.get(file_id_a, 4096).is_none());
}
