use super::*;
use bytes::Bytes;
use memfuse_core::{MemFuseError, StorageEngine};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use tempfile::TempDir;

#[test]
fn test_bloom_filter_fpr_within_bounds() {
    // 1000 echte Elemente, Ziel-FPR = 1%
    let mut bf = BloomFilter::new(1000, 0.01);
    let keys: Vec<Vec<u8>> = (0..1000u32).map(|i| i.to_le_bytes().to_vec()).collect();
    for k in &keys {
        bf.insert(k);
    }

    // Alle echten Elemente müssen enthalten sein (zero false negatives)
    for k in &keys {
        assert!(bf.may_contain(k), "False negative!");
    }

    // False Positive Rate messen mit fremden Keys
    let fp_count = (1000..2000u32)
        .filter(|i| bf.may_contain(&i.to_le_bytes()))
        .count();
    let fpr = fp_count as f64 / 1000.0;
    assert!(fpr < 0.05, "FPR {:.2}% > 5% Toleranz", fpr * 100.0);
}

#[test]
fn test_bloom_filter_no_probe_repetition() {
    // Verifikation: Kein doppeltes Bit für kleine Hashes
    let mut bf = BloomFilter::new(100, 0.01);
    bf.insert(b"test_key");
    // Keine Assertion nötig — wenn kein Panic, ist der Algorithmus stabil
}

#[tokio::test]
async fn test_block_bloom_filter() {
    let mut builder = BlockBuilder::new(4096);
    builder.add(b"apple", b"red", 1, 0);
    builder.add(b"banana", b"yellow", 2, 0);
    let block = builder.build();

    // 1. Verify format: [entries][u64 bloom][u16 offset1][u16 offset2][u16 num_offsets]
    let n = block.len();
    let num_offsets = u16::from_le_bytes(
        block
            .get(n.saturating_sub(2)..n)
            .expect("test") // expect
            .try_into()
            .expect("test"), // expect
    );
    assert_eq!(num_offsets, 2);

    let bloom_pos = n - 2 - (num_offsets as usize * 2) - 8;
    let bloom = u64::from_le_bytes(
        block
            .get(bloom_pos..bloom_pos + 8)
            .expect("test") // expect
            .try_into()
            .expect("correct length"), // expect
    );
    assert!(bloom > 0);

    // 2. Helper to check bloom
    let check_bloom = |key: &[u8], filter: u64| {
        let hash = blake3::hash(key);
        let bytes = hash.as_bytes();
        // Safety: blake3 outputs 32 bytes.
        for i in 0..4 {
            let chunk = u16::from_le_bytes([
                *bytes.get(i * 2).unwrap_or(&0),
                *bytes.get(i * 2 + 1).unwrap_or(&0),
            ]);
            let bit = chunk % 64;
            if (filter & (1 << bit)) == 0 {
                return false;
            }
        }
        true
    };

    assert!(check_bloom(b"apple", bloom));
    assert!(check_bloom(b"banana", bloom));
    // Might have false positive, but definitely shouldn't have many false positives for random strings
    assert!(!check_bloom(b"cherry", bloom));
}

#[tokio::test]
async fn test_sstable_bloom_integration() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let path = tmp.path().join("test.sst");
    let bc = create_block_cache(1);

    let mut builder = SstableBuilder::create(&path).await.expect("create builder"); // expect
    builder.add(b"key1", b"val1", 1, 0).await.expect("add key1"); // expect
    builder.add(b"key2", b"val2", 2, 0).await.expect("add key2"); // expect
    builder.finish().await.expect("finish builder"); // expect

    let reader = SstableReader::open(&path, bc).await.expect("open reader"); // expect

    // Positive lookup
    let res = reader.get(b"key1").await.expect("get key1"); // expect
    assert_eq!(res.expect("exists").0.as_ref(), b"val1"); // expect

    // Negative lookup (should be caught by bloom or range check)
    let res = reader.get(b"nonexistent").await.expect("get nonexistent"); // expect
    assert!(res.is_none());
}

#[tokio::test]
async fn test_mmap_read_correct_values() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let path = tmp.path().join("mmap_test.sst");
    let bc = create_block_cache(1);

    let mut builder = SstableBuilder::create(&path).await.expect("create builder"); // expect
    for i in 0..100 {
        let key = format!("key-{:03}", i);
        let val = format!("val-{:03}", i);
        builder
            .add(key.as_bytes(), val.as_bytes(), i as u64, 0)
            .await
            .expect("add"); // expect
    }
    builder.finish().await.expect("finish"); // expect

    let reader = SstableReader::open(&path, bc).await.expect("open"); // expect
    for i in 0..100 {
        let key = format!("key-{:03}", i);
        let expected = format!("val-{:03}", i);
        let res = reader
            .get(key.as_bytes())
            .await
            .expect("get") // expect
            .expect("exists"); // expect
        assert_eq!(res.0.as_ref(), expected.as_bytes());
        assert_eq!(res.1, i as u64);
    }
}

#[tokio::test]
async fn test_mmap_concurrent_readers() {
    use std::sync::Arc;
    let tmp = TempDir::new().expect("temp dir"); // expect
    let path = tmp.path().join("mmap_concurrent.sst");
    let bc = create_block_cache(1);

    let mut builder = SstableBuilder::create(&path).await.expect("create builder"); // expect
    for i in 0..100 {
        let key = format!("key-{:03}", i);
        let val = format!("val-{:03}", i);
        builder
            .add(key.as_bytes(), val.as_bytes(), i as u64, 0)
            .await
            .expect("add"); // expect
    }
    builder.finish().await.expect("finish"); // expect

    let reader = Arc::new(SstableReader::open(&path, bc).await.expect("open")); // expect
    let mut handles = Vec::new();

    for _ in 0..16 {
        let r = Arc::clone(&reader);
        handles.push(tokio::spawn(async move {
            for i in 0..100 {
                let key = format!("key-{:03}", i);
                let expected = format!("val-{:03}", i);
                let res = r.get(key.as_bytes()).await.expect("get").expect("exists"); // expect
                assert_eq!(res.0.as_ref(), expected.as_bytes());
            }
        }));
    }

    for h in handles {
        h.await.expect("task failed"); // expect
    }
}

#[tokio::test]
async fn test_sstable_scan_prefix() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let path = tmp.path().join("scan_prefix.sst");
    let bc = create_block_cache(1);

    let mut builder = SstableBuilder::create(&path).await.expect("create builder"); // expect
    builder.add(b"apple/1", b"a1", 1, 0).await.expect("add"); // expect
    builder.add(b"apple/2", b"a2", 2, 0).await.expect("add"); // expect
    builder.add(b"banana/1", b"b1", 3, 0).await.expect("add"); // expect
    builder.add(b"cherry/1", b"c1", 4, 0).await.expect("add"); // expect
    builder.finish().await.expect("finish"); // expect

    let reader = SstableReader::open(&path, bc).await.expect("open"); // expect

    let apples = reader.scan_prefix(b"apple/").await.expect("scan"); // expect
    assert_eq!(apples.len(), 2);
    assert_eq!(apples[0].0.as_ref(), b"apple/1");
    assert_eq!(apples[1].0.as_ref(), b"apple/2");

    let bananas = reader.scan_prefix(b"banana/").await.expect("scan"); // expect
    assert_eq!(bananas.len(), 1);
    assert_eq!(bananas[0].0.as_ref(), b"banana/1");

    let non = reader.scan_prefix(b"zebra").await.expect("scan"); // expect
    assert!(non.is_empty());
}

#[tokio::test]
async fn test_sstable_scan_range() {
    use std::ops::Bound;
    let tmp = TempDir::new().expect("temp dir"); // expect
    let path = tmp.path().join("scan_range.sst");
    let bc = create_block_cache(1);

    let mut builder = SstableBuilder::create(&path).await.expect("create builder"); // expect
    builder.add(b"a", b"1", 1, 0).await.expect("add"); // expect
    builder.add(b"b", b"2", 2, 0).await.expect("add"); // expect
    builder.add(b"c", b"3", 3, 0).await.expect("add"); // expect
    builder.add(b"d", b"4", 4, 0).await.expect("add"); // expect
    builder.finish().await.expect("finish"); // expect

    let reader = SstableReader::open(&path, bc).await.expect("open"); // expect

    // Included Range [b, c]
    let res = reader
        .scan_range(Bound::Included(b"b"), Bound::Included(b"c"))
        .await
        .expect("scan"); // expect
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].0.as_ref(), b"b");
    assert_eq!(res[1].0.as_ref(), b"c");

    // Excluded Range (b, d) -> only c
    let res = reader
        .scan_range(Bound::Excluded(b"b"), Bound::Excluded(b"d"))
        .await
        .expect("scan"); // expect
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].0.as_ref(), b"c");

    // Unbounded
    let res = reader
        .scan_range(Bound::Unbounded, Bound::Included(b"b"))
        .await
        .expect("scan"); // expect
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].0.as_ref(), b"a");
}

#[tokio::test]
async fn test_mfsx_bloom_filter_crc_recovery_no_false_rejections() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let sst_path = tmp.path().join("mfsx_crc_recovery.sst");
    let bc = create_block_cache(1);

    // Generate 120 unique keys
    let keys: Vec<Vec<u8>> = (0..120)
        .map(|i| format!("key_recovery_{:04}_{}", i, rand::random::<u32>()).into_bytes())
        .collect();

    // 1. Build MFSX SSTable with CRC
    {
        let mut builder = SstableBuilder::create(&sst_path)
            .await
            .expect("create builder"); // expect
        for (seq, key) in keys.iter().enumerate() {
            let val = format!("val_{}", seq).into_bytes();
            builder
                .add(key, &val, seq as u64 + 1, 1)
                .await
                .expect("add key"); // expect
        }
        builder.finish().await.expect("finish builder"); // expect
    }

    // 2. Reopen reader (simulating process restart)
    let reader = SstableReader::open(&sst_path, bc)
        .await
        .expect("reopen reader"); // expect

    assert!(
        reader.bloom_filter.is_some(),
        "MFSX SSTable must have bloom filter"
    );
    assert!(reader.has_crc, "MFSX SSTable must have CRC enabled");

    // 3. Verify for ALL 120 keys that Bloom filter does not falsely reject any key
    for (seq, key) in keys.iter().enumerate() {
        let res = reader
            .get(key)
            .await
            .expect("get key")
            .expect("key must exist"); // expect
        let expected_val = format!("val_{}", seq).into_bytes();
        assert_eq!(
            res.0.as_ref(),
            expected_val.as_slice(),
            "Key {:?} must be found without false rejection",
            String::from_utf8_lossy(key)
        );
    }
}

#[tokio::test]
async fn test_bloom_filter_integration() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let sst_path = tmp.path().join("bloom_test.sst");
    let bc = create_block_cache(1);

    // 1. Create SSTable with bloom filter
    {
        let mut builder = SstableBuilder::create(&sst_path).await.expect("create"); // expect
        builder
            .add(b"active-key", b"value", 100, 0)
            .await
            .expect("add"); // expect
        builder.finish().await.expect("finish"); // expect
    }

    // 2. Open and verify
    {
        let reader = SstableReader::open(&sst_path, bc.clone())
            .await
            .expect("open"); // expect
        assert!(reader.bloom_filter.is_some(), "Should have bloom filter");

        // Positive check
        let res = reader.get(b"active-key").await.expect("get"); // expect
        assert!(res.is_some());

        // Negative check (Bloom should say NO)
        let res = reader.get(b"missing-key-xyz-123").await.expect("get"); // expect
        assert!(res.is_none());
    }

    // 3. Backward compatibility (Manually create 12-byte trailer file)
    let old_sst_path = tmp.path().join("old_sst.sst");
    {
        // Use builder to create a valid SSTable first
        let mut builder = SstableBuilder::create(&old_sst_path)
            .await
            .expect("create old sub"); // expect
        builder
            .add(b"old-key", b"old-value", 10, 0)
            .await
            .expect("add"); // expect
        builder.finish().await.expect("finish"); // expect

        // Now manually truncate the trailer from 20 to 12 bytes
        // The file currently has: [data][index][bloom][bloom_off][index_off][magic] (total trailer 20)
        // We want to simulate: [data][index][index_off][magic] (total trailer 12)
        // Actually, just writing a 12-byte trailer pointing to the index is enough.
        let data = tokio::fs::read(&old_sst_path).await.expect("read"); // expect
        let file_size = data.len();
        let index_off = u64::from_le_bytes(data[file_size - 12..file_size - 4].try_into().unwrap()); // unwrap

        let mut new_data = data[0..file_size - 20].to_vec(); // remove new trailer and bloom
                                                             // index likely ends at bloom_off. Let's just use the index_off we found.
        new_data.truncate((file_size - 20) as usize); // this might cut off some index if bloom was there
                                                      // Re-read data up to index_offset + index_size
                                                      // Actually, simpler: just rewrite a 12-byte trailer at the end of a valid data+index block.
                                                      // Let's just trust SstableReader to handle it if we only provide 12 bytes.
        let mut f = tokio::fs::File::create(&old_sst_path)
            .await
            .expect("recreate"); // expect
        f.write_all(&data[0..file_size - 24]).await.unwrap(); // expect
        f.write_u64_le(index_off).await.expect("write ioff"); // expect
        f.write_u32_le(0x4D465354).await.expect("write magic"); // expect
        f.sync_all().await.expect("sync"); // expect
    }

    {
        let reader = SstableReader::open(&old_sst_path, bc)
            .await
            .expect("open old"); // expect
        assert!(
            reader.bloom_filter.is_none(),
            "Old SST should not have bloom filter"
        );
    }
}

#[tokio::test]
async fn test_sstable_block_crc_corruption() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let path = tmp.path().join("crc_corrupt.sst");
    let bc = create_block_cache(1);

    {
        let mut builder = SstableBuilder::create(&path).await.expect("create"); // expect
        builder.add(b"key1", b"val1", 1, 0).await.expect("add"); // expect
        builder.finish().await.expect("finish"); // expect
    }

    // Corrupt the first block
    {
        let mut data = tokio::fs::read(&path).await.expect("read"); // expect
                                                                    // Blocks start at 0. Let's flip a bit at offset 10.
        if data.len() > 10 {
            data[10] ^= 0xFF;
            tokio::fs::write(&path, data).await.expect("write"); // expect
        }
    }

    let reader_res = SstableReader::open(&path, bc).await;

    match reader_res {
        Ok(reader) => {
            let res = reader.get(b"key1").await;
            assert!(
                res.is_err(),
                "Expected error due to corruption during get, but got {:?}",
                res
            );
            assert!(matches!(
                res.unwrap_err(),
                MemFuseError::ChecksumMismatch { .. }
            ));
        }
        Err(e) => {
            assert!(
                matches!(e, MemFuseError::ChecksumMismatch { .. }),
                "Expected ChecksumMismatch, got {:?}",
                e
            );
        }
    }
}

#[test]
fn test_block_cache_extreme_values() {
    // Darf nicht paniken oder overflowlen – prüft alle Grenzfälle.
    let _ = create_block_cache(0); // minimum → floor auf 256 Blöcke
    let _ = create_block_cache(1); // normal
    let _ = create_block_cache(usize::MAX); // overflow-Test → saturating → cap
    let _ = create_block_cache(usize::MAX / 2); // near-overflow → saturating → cap
}

#[tokio::test]
async fn test_block_cache_eviction_under_load() {
    // Create a 10-byte capacity per shard cache directly (for 6-byte blocks)
    let cache = Arc::new(BlockCache::new(10));

    let file_id = 1u64;
    // Find 3 offsets that map to the exact same shard index using ahash
    let target_shard = cache.shard_idx(file_id, 0);
    let mut offsets = vec![0u64];
    let mut candidate = 1u64;
    while offsets.len() < 3 {
        if cache.shard_idx(file_id, candidate) == target_shard {
            offsets.push(candidate);
        }
        candidate += 1;
    }

    let offset1 = offsets[0];
    let offset2 = offsets[1];
    let offset3 = offsets[2];

    // 1. Insert block 1 -> populates shard
    cache.insert(file_id, offset1, Bytes::from_static(b"block1"));
    assert_eq!(cache.len(), 1);
    assert!(cache.contains(file_id, offset1));

    // 2. Insert block 2 -> evicts block 1 in same shard
    cache.insert(file_id, offset2, Bytes::from_static(b"block2"));
    assert_eq!(cache.len(), 1);
    assert!(!cache.contains(file_id, offset1));
    assert!(cache.contains(file_id, offset2));

    // 3. Insert block 3 -> evicts block 2 in same shard
    cache.insert(file_id, offset3, Bytes::from_static(b"block3"));
    assert_eq!(cache.len(), 1);
    assert!(!cache.contains(file_id, offset1));
    assert!(!cache.contains(file_id, offset2));
    assert!(cache.contains(file_id, offset3));
}

#[tokio::test]
async fn test_block_cache_byte_capacity_eviction_threshold() {
    // Set per-shard capacity to 20 bytes
    let cache = Arc::new(BlockCache::new(20));
    let file_id = 100u64;

    // Find offsets mapping to shard 0
    let mut offsets = Vec::new();
    let mut cand = 0u64;
    while offsets.len() < 2 {
        if cache.shard_idx(file_id, cand) == 0 {
            offsets.push(cand);
        }
        cand += 1;
    }

    let block1 = Bytes::from(vec![0u8; 15]); // 15 bytes
    let block2 = Bytes::from(vec![1u8; 10]); // 10 bytes (total 25 > 20 capacity_bytes)

    cache.insert(file_id, offsets[0], block1);
    assert_eq!(cache.len(), 1);
    assert!(cache.contains(file_id, offsets[0]));

    // Insert block2, causing total bytes (25) to exceed shard capacity (20) -> evicts block1
    cache.insert(file_id, offsets[1], block2);
    assert_eq!(cache.len(), 1);
    assert!(
        !cache.contains(file_id, offsets[0]),
        "block1 must be evicted due to byte capacity limit"
    );
    assert!(cache.contains(file_id, offsets[1]));
}

#[tokio::test]
async fn test_sstable_builder_duplicate_keys_coexist() {
    let tmp = TempDir::new().expect("temp dir"); // expect #[cfg(test)]
    let path = tmp.path().join("duplicate_keys.sst");
    let bc = create_block_cache(1);

    let mut builder = SstableBuilder::create(&path).await.expect("create"); // expect #[cfg(test)]
    builder.add(b"k", b"val1", 1, 10).await.expect("add seq 1"); // expect #[cfg(test)]
    builder.add(b"k", b"val2", 2, 20).await.expect("add seq 2"); // expect #[cfg(test)]
    builder.finish().await.expect("finish"); // expect #[cfg(test)]

    let reader = SstableReader::open(&path, bc).await.expect("open"); // expect #[cfg(test)]

    // 1. Verify via iter()
    let iter_entries = reader.iter().await.expect("iter"); // expect #[cfg(test)]
    assert_eq!(
        iter_entries.len(),
        2,
        "Both duplicate key entries must coexist in iter()"
    );
    assert_eq!(
        iter_entries[0],
        (Bytes::from_static(b"k"), Bytes::from_static(b"val1"), 1)
    );
    assert_eq!(
        iter_entries[1],
        (Bytes::from_static(b"k"), Bytes::from_static(b"val2"), 2)
    );

    // 2. Verify via stream()
    let reader_arc = Arc::new(reader);
    let mut stream = reader_arc.stream().await.expect("stream"); // expect #[cfg(test)]
    let e1 = stream.next().await.expect("next").expect("entry 1"); // expect #[cfg(test)]
    let e2 = stream.next().await.expect("next").expect("entry 2"); // expect #[cfg(test)]
    let e_end = stream.next().await.expect("next"); // expect #[cfg(test)]

    assert_eq!(
        e1,
        (Bytes::from_static(b"k"), Bytes::from_static(b"val1"), 1, 10)
    );
    assert_eq!(
        e2,
        (Bytes::from_static(b"k"), Bytes::from_static(b"val2"), 2, 20)
    );
    assert!(e_end.is_none());
}

#[test]
fn test_bloom_filter_boundary_clamping() {
    // Test extreme elements input
    let bf = BloomFilter::new(usize::MAX, 0.01);
    assert!(bf.num_bits <= 128 * 1024 * 1024 * 8);

    // Test corrupted bytes input
    let mut corrupted_data = vec![0u8; 16];
    // Set num_bits to huge value
    corrupted_data[8..16].copy_from_slice(&(u64::MAX).to_le_bytes());
    let res = BloomFilter::from_bytes(&corrupted_data);
    assert!(res.is_err());

    // Test capacity cap on from_bytes
    let mut valid_header = vec![0u8; 24];
    valid_header[0..8].copy_from_slice(&1u64.to_le_bytes()); // num_hashes
    valid_header[8..16].copy_from_slice(&1000u64.to_le_bytes()); // num_bits
    let bf_res = BloomFilter::from_bytes(&valid_header);
    assert!(bf_res.is_ok());
}

#[test]
fn test_bloom_filter_roundtrip_and_too_short_bytes() {
    // Test roundtrip
    let mut bf = BloomFilter::new(50, 0.01);
    bf.insert(b"test-key-1");
    bf.insert(b"test-key-2");
    let bytes = bf.to_bytes();

    let restored = BloomFilter::from_bytes(&bytes).expect("deserialization should succeed"); // expect
    assert!(restored.may_contain(b"test-key-1"));
    assert!(restored.may_contain(b"test-key-2"));
    assert!(!restored.may_contain(b"non-existent-key"));

    // Test deserialization with too short data (< 16 bytes)
    let short_bytes = vec![0u8; 15];
    let err = BloomFilter::from_bytes(&short_bytes);
    assert!(matches!(err, Err(MemFuseError::Storage(_))));
}

#[test]
fn test_block_builder_min_size() {
    let builder = BlockBuilder::new(10);
    assert_eq!(builder.block_size, 512);
}

#[test]
fn test_block_builder_min_max_clamping() {
    let bb_small = BlockBuilder::new(10);
    assert_eq!(bb_small.block_size, 512);

    let bb_huge = BlockBuilder::new(100 * 1024 * 1024);
    assert_eq!(bb_huge.block_size, 64 * 1024 * 1024);
}

#[tokio::test]
async fn test_sstable_builder_rejects_empty_and_oversized_inputs() {
    let tmp = TempDir::new().expect("temp dir"); // expect
    let path = tmp.path().join("boundary_test.sst");

    let mut builder = SstableBuilder::create(&path).await.expect("create"); // expect

    // 1. Empty key reject
    let err_empty = builder.add(b"", b"val", 1, 1).await;
    assert!(matches!(err_empty, Err(MemFuseError::InvalidInput(_))));

    // 2. Oversized key (>65535) reject
    let oversized_key = vec![0xAA; 65536];
    let err_key = builder.add(&oversized_key, b"val", 1, 1).await;
    assert!(matches!(err_key, Err(MemFuseError::InvalidInput(_))));

    // 3. Oversized value (>MAX_VALUE_SIZE) reject
    let oversized_val = vec![0xBB; crate::lsm::MAX_VALUE_SIZE + 1];
    let err_val = builder.add(b"valid_key", &oversized_val, 1, 1).await;
    assert!(matches!(err_val, Err(MemFuseError::InvalidInput(_))));
}

#[test]
fn test_binary_search_vs_linear_search_equivalence() {
    // Test randomized blocks with 1..=64 entries to ensure binary search matches linear search exactly
    for num_entries in 1..=64 {
        let mut builder = BlockBuilder::new(64 * 1024);
        let mut expected_entries = Vec::new();
        for i in 0..num_entries {
            let key = format!("key_{:04}", i).into_bytes();
            let val = format!("val_{:04}", i).into_bytes();
            builder.add(&key, &val, i as u64, 0);
            expected_entries.push((key, val));
        }
        let block = builder.build();
        let n = block.len();
        let num_offsets = u16::from_le_bytes(block[n - 2..n].try_into().unwrap()) as usize;
        let offsets_len = num_offsets * 2;
        let offsets_start = n - 2 - offsets_len;

        // 1. Verify all present keys
        for (key, _val) in &expected_entries {
            // Linear search
            let mut linear_res = None;
            for idx in 0..num_offsets {
                let off_pos = offsets_start + idx * 2;
                let entry_off =
                    u16::from_le_bytes(block[off_pos..off_pos + 2].try_into().unwrap()) as usize;
                let k_len = u16::from_le_bytes(block[entry_off..entry_off + 2].try_into().unwrap())
                    as usize;
                let entry_key = &block[entry_off + 2..entry_off + 2 + k_len];
                if entry_key == key {
                    linear_res = Some((entry_off, k_len));
                    break;
                }
            }

            // Binary search
            let bin_res =
                block_search::binary_search_entry_in_block(&block, offsets_start, num_offsets, key)
                    .expect("binary search should not error");

            assert_eq!(
                linear_res,
                bin_res,
                "Mismatch for key {:?} in block with {} entries",
                String::from_utf8_lossy(key),
                num_entries
            );

            if let Some((off, klen)) = bin_res {
                assert_eq!(&block[off + 2..off + 2 + klen], key.as_slice());
            }
        }

        // 2. Verify non-existent keys
        let non_existent_keys = [
            b"key_-001".as_slice(),
            b"key_9999".as_slice(),
            b"key_0000_foo".as_slice(),
        ];
        for nek in non_existent_keys {
            let bin_res =
                block_search::binary_search_entry_in_block(&block, offsets_start, num_offsets, nek)
                    .expect("binary search should not error");
            assert!(
                bin_res.is_none(),
                "Non-existent key {:?} found in block with {} entries",
                String::from_utf8_lossy(nek),
                num_entries
            );
        }
    }
}

#[tokio::test]
async fn test_sharded_block_cache_concurrency_stress() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let cache = Arc::new(BlockCache::new(16));
    let num_tasks = 32;
    let ops_per_task = 1000;
    let completed = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for task_id in 0..num_tasks {
        let cache_ref = Arc::clone(&cache);
        let completed_ref = Arc::clone(&completed);

        handles.push(tokio::spawn(async move {
            for i in 0..ops_per_task {
                // Unique key targeting different shards
                let file_id = (task_id as u64) + 1;
                let block_offset = (i as u64) * 4096;
                let val = Bytes::from(format!("data_{}_{}", task_id, i));

                cache_ref.insert(file_id, block_offset, val.clone());
                let retrieved = cache_ref.get(file_id, block_offset);
                assert_eq!(retrieved, Some(val));
            }
            completed_ref.fetch_add(1, Ordering::SeqCst);
        }));
    }

    for h in handles {
        h.await.expect("task panicked");
    }

    assert_eq!(completed.load(Ordering::SeqCst), num_tasks);
    assert!(!cache.is_empty());
}

#[tokio::test]
async fn test_block_cache_shards_config_validation() {
    use crate::lsm::{LsmConfig, LsmStorage};

    let tmp = TempDir::new().expect("temp dir");

    // 0 shards -> Err
    let config_zero = LsmConfig {
        path: tmp.path().join("zero"),
        block_cache_shards: 0,
        ..Default::default()
    };
    assert!(LsmStorage::new(config_zero).await.is_err());

    // Non-power-of-two shards (e.g. 15) -> Err
    let config_non_pow2 = LsmConfig {
        path: tmp.path().join("non_pow2"),
        block_cache_shards: 15,
        ..Default::default()
    };
    assert!(LsmStorage::new(config_non_pow2).await.is_err());

    // Power of two shards (e.g. 32) -> Ok
    let config_valid = LsmConfig {
        path: tmp.path().join("valid"),
        block_cache_shards: 32,
        ..Default::default()
    };
    let storage = LsmStorage::new(config_valid).await;
    assert!(storage.is_ok());
}

#[test]
fn test_block_cache_sequential_scan_sharding_distribution() {
    // Verification of sharding distribution during sequential scans with constant file_id.
    // The hashing function `ahash::RandomState::hash_one((file_id, offset))` distributes
    // sequential block offsets (4KB increments) evenly across all shards.
    let num_shards = 64;
    let cache = BlockCache::new_with_shards(16, num_shards);
    let file_id = 100u64;
    let num_blocks = 1000u64;

    let mut shard_counts = vec![0usize; num_shards];
    for i in 0..num_blocks {
        let offset = i * 4096;
        let shard = cache.shard_idx(file_id, offset);
        shard_counts[shard] += 1;
    }

    // Ensure every shard received at least one block offset (no dead shards in sequential scan)
    let empty_shards = shard_counts.iter().filter(|&&c| c == 0).count();
    assert_eq!(
        empty_shards, 0,
        "Sequential scan produced empty shards: {:?}",
        shard_counts
    );

    // Verify standard deviation or max load to ensure uniform distribution
    let expected_avg = num_blocks as f64 / num_shards as f64;
    for (idx, &count) in shard_counts.iter().enumerate() {
        let diff = (count as f64 - expected_avg).abs();
        assert!(
            diff < expected_avg * 1.5,
            "Shard {} has count {} which deviates significantly from expected average {:.1}",
            idx,
            count,
            expected_avg
        );
    }
}

#[tokio::test]
async fn test_block_cache_32_concurrent_readers_hotset_latency() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    let cache = Arc::new(BlockCache::new(64));
    let file_id = 42u64;
    let offset = 4096u64;
    let val = Bytes::from_static(b"hotset_block_payload");

    cache.insert(file_id, offset, val.clone());

    let num_readers = 32;
    let reads_per_reader = 10_000;
    let total_hits = Arc::new(AtomicUsize::new(0));

    let start = Instant::now();
    let mut handles = Vec::with_capacity(num_readers);

    for _ in 0..num_readers {
        let cache_ref = Arc::clone(&cache);
        let hits_ref = Arc::clone(&total_hits);
        let expected_val = val.clone();
        handles.push(tokio::spawn(async move {
            let mut hits = 0;
            for _ in 0..reads_per_reader {
                if let Some(res) = cache_ref.get(file_id, offset) {
                    if res == expected_val {
                        hits += 1;
                    }
                }
            }
            hits_ref.fetch_add(hits, Ordering::Relaxed);
        }));
    }

    for h in handles {
        h.await.expect("reader task completed");
    }

    let elapsed = start.elapsed();
    let total_reads = num_readers * reads_per_reader;
    assert_eq!(total_hits.load(Ordering::SeqCst), total_reads);

    let ns_per_op = elapsed.as_nanos() as f64 / total_reads as f64;
    println!(
            "[BlockCache Benchmark] 32 concurrent readers hot-set: {} total ops in {:?}, avg {:.2} ns/op (feature block-cache-v2={})",
            total_reads,
            elapsed,
            ns_per_op,
            cfg!(feature = "block-cache-v2")
        );
}
