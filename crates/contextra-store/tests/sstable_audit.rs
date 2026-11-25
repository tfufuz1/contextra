#![allow(clippy::expect_used, clippy::unwrap_used, clippy::type_complexity)]

use contextra_store::sstable::{create_block_cache, BloomFilter, SstableBuilder, SstableReader};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeMap;
use std::ops::Bound;
use tempfile::TempDir;

/// Differential test against std BTreeMap comparing point lookups and range/prefix scans.
#[tokio::test]
async fn test_sstable_btreemap_differential() {
    let mut rng = StdRng::seed_from_u64(0x1234_5678);
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("differential.sst");
    let bc = create_block_cache(2);

    let mut map: BTreeMap<Vec<u8>, (Vec<u8>, u64, u64)> = BTreeMap::new();
    let mut builder = SstableBuilder::create(&sst_path)
        .await
        .expect("create builder");

    let num_entries = 150;
    let mut keys = Vec::new();
    for i in 0..num_entries {
        let key_len = rng.gen_range(1..=24);
        let mut k = vec![0u8; key_len];
        rng.fill(&mut k[..]);

        let val_len = rng.gen_range(0..=256);
        let mut v = vec![0u8; val_len];
        rng.fill(&mut v[..]);

        let seq = (i + 1) as u64;
        let tx = (i / 10 + 1) as u64;

        keys.push(k.clone());
        map.insert(k, (v, seq, tx));
    }

    let sorted_entries: Vec<(Vec<u8>, (Vec<u8>, u64, u64))> = map.clone().into_iter().collect();

    for (k, (v, seq, tx)) in &sorted_entries {
        builder.add(k, v, *seq, *tx).await.expect("add entry");
    }
    builder.finish().await.expect("finish builder");

    let reader = SstableReader::open(&sst_path, bc)
        .await
        .expect("open reader");

    // 1. Point lookup check for all written keys
    for (k, (v, seq, tx)) in &map {
        let res = reader.get(k).await.expect("get key");
        assert!(res.is_some(), "Key not found: {:?}", k);
        let (found_val, found_seq, found_tx) = res.unwrap();
        assert_eq!(found_val.as_ref(), v.as_slice());
        assert_eq!(found_seq, *seq);
        assert_eq!(found_tx, *tx);
    }

    // 2. Point lookup check for random non-existent keys
    for _ in 0..50 {
        let key_len = rng.gen_range(1..=24);
        let mut k = vec![0u8; key_len];
        rng.fill(&mut k[..]);
        if !map.contains_key(&k) {
            let res = reader.get(&k).await.expect("get non-existent");
            assert!(res.is_none(), "Non-existent key returned data");
        }
    }

    // 3. Scan prefix test
    let prefixes = [vec![], vec![keys[0][0]], vec![0x00], vec![0xFF]];
    for prefix in &prefixes {
        let expected: Vec<(&Vec<u8>, &(Vec<u8>, u64, u64))> =
            map.iter().filter(|(k, _)| k.starts_with(prefix)).collect();
        let sst_prefix_res = reader.scan_prefix(prefix).await.expect("scan_prefix");
        assert_eq!(
            sst_prefix_res.len(),
            expected.len(),
            "Prefix scan length mismatch for prefix {:?}",
            prefix
        );
        for (idx, (k, v, seq, tx)) in sst_prefix_res.iter().enumerate() {
            assert_eq!(k.as_ref(), expected[idx].0.as_slice());
            assert_eq!(v.as_ref(), expected[idx].1 .0.as_slice());
            assert_eq!(*seq, expected[idx].1 .1);
            assert_eq!(*tx, expected[idx].1 .2);
        }
    }

    // 4. Range scan test
    if map.len() >= 2 {
        let sorted_keys: Vec<&Vec<u8>> = map.keys().collect();
        let start_k = sorted_keys[5];
        let end_k = sorted_keys[sorted_keys.len() - 5];

        let expected_range: Vec<(&Vec<u8>, &(Vec<u8>, u64, u64))> =
            map.range(start_k.clone()..=end_k.clone()).collect();

        let sst_range_res = reader
            .scan_range(Bound::Included(start_k), Bound::Included(end_k))
            .await
            .expect("scan_range");

        assert_eq!(sst_range_res.len(), expected_range.len());
        for (idx, (k, v, seq, tx)) in sst_range_res.iter().enumerate() {
            assert_eq!(k.as_ref(), expected_range[idx].0.as_slice());
            assert_eq!(v.as_ref(), expected_range[idx].1 .0.as_slice());
            assert_eq!(*seq, expected_range[idx].1 .1);
            assert_eq!(*tx, expected_range[idx].1 .2);
        }
    }
}

/// Bitflip mutation test across every single byte offset of a small SSTable file.
/// Verifies ZERO PANICS and ZERO INCORRECT DATA returned.
#[tokio::test]
async fn test_sstable_bitflip_all_byte_positions() {
    let tmp = TempDir::new().expect("temp dir");
    let valid_sst_path = tmp.path().join("valid_bitflip.sst");
    let corrupt_path = tmp.path().join("corrupt_bitflip.sst");
    let bc = create_block_cache(1);

    let mut builder = SstableBuilder::create(&valid_sst_path)
        .await
        .expect("create");
    let keys = vec![
        (b"a".to_vec(), b"va".to_vec()),
        (b"b".to_vec(), b"vb".to_vec()),
    ];

    for (k, v) in &keys {
        builder.add(k, v, 10, 1).await.expect("add");
    }
    builder.finish().await.expect("finish");

    let original_bytes = std::fs::read(&valid_sst_path).expect("read valid sst");

    for byte_offset in 0..original_bytes.len() {
        for bit_idx in [0, 7] {
            let mut corrupted_bytes = original_bytes.clone();
            corrupted_bytes[byte_offset] ^= 1 << bit_idx;

            std::fs::write(&corrupt_path, &corrupted_bytes).expect("write corrupted sst");

            if let Ok(reader) = SstableReader::open(&corrupt_path, bc.clone()).await {
                for (k, v) in &keys {
                    match reader.get(k).await {
                        Ok(Some((found_val, _, _))) => {
                            assert_eq!(
                                found_val.as_ref(),
                                v.as_slice(),
                                "Bitflip at offset {} bit {} produced WRONG DATA!",
                                byte_offset,
                                bit_idx
                            );
                        }
                        Ok(None) => {}
                        Err(_) => {}
                    }
                }

                let _ = reader.scan_prefix(b"").await;
                let _ = reader.scan_range(Bound::Unbounded, Bound::Unbounded).await;
            }
        }
    }
}

/// Truncation test across every single byte offset (0..len) of a valid SSTable file.
/// Verifies ZERO PANICS and ZERO INCORRECT DATA returned.
#[tokio::test]
async fn test_sstable_truncation_all_byte_positions() {
    let tmp = TempDir::new().expect("temp dir");
    let valid_sst_path = tmp.path().join("valid_trunc.sst");
    let trunc_path = tmp.path().join("corrupt_trunc.sst");
    let bc = create_block_cache(1);

    let mut builder = SstableBuilder::create(&valid_sst_path)
        .await
        .expect("create");
    let keys = vec![
        (b"key_a".to_vec(), b"val_a".to_vec()),
        (b"key_b".to_vec(), b"val_b".to_vec()),
    ];

    for (k, v) in &keys {
        builder.add(k, v, 1, 1).await.expect("add");
    }
    builder.finish().await.expect("finish");

    let original_bytes = std::fs::read(&valid_sst_path).expect("read valid sst");

    for trunc_len in 0..original_bytes.len() {
        let truncated_bytes = &original_bytes[..trunc_len];
        std::fs::write(&trunc_path, truncated_bytes).expect("write truncated sst");

        if let Ok(reader) = SstableReader::open(&trunc_path, bc.clone()).await {
            for (k, v) in &keys {
                if let Ok(Some((found_val, _, _))) = reader.get(k).await {
                    assert_eq!(
                        found_val.as_ref(),
                        v.as_slice(),
                        "Truncation at length {} produced WRONG DATA!",
                        trunc_len
                    );
                }
            }
        }
    }
}

/// Verifies that replacing an SSTable file at the exact same path with new content
/// does NOT yield stale/alias cache hits when using a shared BlockCache.
#[tokio::test]
async fn test_shared_block_cache_file_replacement_no_alias_hits() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("replaced.sst");
    let bc = create_block_cache(1);

    // 1. Write SSTable A
    {
        let mut builder = SstableBuilder::create(&sst_path).await.expect("create");
        builder
            .add(b"shared_key", b"content_A", 1, 1)
            .await
            .expect("add");
        builder.finish().await.expect("finish");
    }

    let reader_a = SstableReader::open(&sst_path, bc.clone())
        .await
        .expect("open A");
    let res_a = reader_a
        .get(b"shared_key")
        .await
        .expect("get A")
        .expect("exists A");
    assert_eq!(res_a.0.as_ref(), b"content_A");

    // 2. Overwrite file at same path with SSTable B
    {
        let mut builder = SstableBuilder::create(&sst_path).await.expect("create B");
        builder
            .add(b"shared_key", b"content_B_updated", 2, 2)
            .await
            .expect("add B");
        builder.finish().await.expect("finish B");
    }

    // 3. Open SSTable B with the same shared block cache instance
    let reader_b = SstableReader::open(&sst_path, bc.clone())
        .await
        .expect("open B");
    let res_b = reader_b
        .get(b"shared_key")
        .await
        .expect("get B")
        .expect("exists B");

    assert_eq!(
        res_b.0.as_ref(),
        b"content_B_updated",
        "Block cache returned stale alias hit from old file!"
    );
}

/// Verifies that SstableBuilder rejects keys added out of order or duplicate keys with non-descending seq numbers.
#[tokio::test]
async fn test_sstable_builder_rejects_out_of_order_keys() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("out_of_order.sst");

    let mut builder = SstableBuilder::create(&sst_path).await.expect("create");
    builder
        .add(b"key_b", b"v1", 10, 1)
        .await
        .expect("add key_b");

    // Adding key_a after key_b must fail
    let err_order = builder.add(b"key_a", b"v2", 10, 1).await;
    assert!(
        err_order.is_err(),
        "SstableBuilder must reject keys added out of lexicographical order"
    );

    // Adding same key with ascending sequence number must fail
    let mut builder2 = SstableBuilder::create(tmp.path().join("seq_order.sst"))
        .await
        .expect("create 2");
    builder2
        .add(b"dup_key", b"v1", 10, 1)
        .await
        .expect("add seq 10");
    let err_seq = builder2.add(b"dup_key", b"v2", 20, 1).await;
    assert!(
        err_seq.is_err(),
        "SstableBuilder must reject duplicate keys with ascending sequence numbers"
    );
}

/// Verifies that BloomFilter::from_bytes handles corrupted num_hashes and truncated bytes safely.
#[test]
fn test_bloom_filter_from_bytes_hardening() {
    // 1. Corrupted num_hashes (> 64)
    let mut corrupt_num_hashes = vec![0u8; 16];
    corrupt_num_hashes[0..8].copy_from_slice(&100u64.to_le_bytes()); // num_hashes = 100
    corrupt_num_hashes[8..16].copy_from_slice(&64u64.to_le_bytes()); // num_bits = 64
    assert!(BloomFilter::from_bytes(&corrupt_num_hashes).is_err());

    // 2. Truncated payload bytes relative to num_bits
    let mut truncated = vec![0u8; 24]; // header 16 bytes + 8 payload bytes (1 u64 word)
    truncated[0..8].copy_from_slice(&7u64.to_le_bytes()); // num_hashes = 7
    truncated[8..16].copy_from_slice(&1024u64.to_le_bytes()); // num_bits = 1024 (requires 16 u64 words = 128 bytes)
    assert!(
        BloomFilter::from_bytes(&truncated).is_err(),
        "from_bytes must reject bloom filter data shorter than num_bits requirement"
    );
}
