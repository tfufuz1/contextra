use bytes::Bytes;
use contextra_core::{StorageEngine, TxId, TOMBSTONE_BIT};
use contextra_store::memtable::MemTable;
use contextra_store::{LsmConfig, LsmStorage};
use std::collections::BTreeMap;
use std::ops::Bound;
use std::sync::Arc;
use tempfile::TempDir;

async fn test_storage() -> (LsmStorage, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: std::time::Duration::from_secs(60),
        compaction: Default::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");
    (storage, tmp)
}

#[tokio::test]
async fn test_prefix_scan_ff_bytes_and_empty_prefix() {
    let (storage, _tmp) = test_storage().await;

    // 1. Insert keys with 0xFF bytes in prefix and empty prefix
    let tx1 = TxId::new(1);
    storage.put(tx1, b"\xFF\xFFkey1", b"val_ff1").await.unwrap();
    storage.put(tx1, b"\xFF\xFFkey2", b"val_ff2").await.unwrap();
    storage
        .put(tx1, b"abc\xFFkey", b"val_abc_ff")
        .await
        .unwrap();
    storage
        .put(tx1, b"normal_key", b"val_normal")
        .await
        .unwrap();
    storage.commit(tx1).await.unwrap();

    // Flush to create SSTables
    storage.force_flush().await.unwrap();

    // Scan with empty prefix b""
    let res_empty = storage.scan_prefix(b"").await.unwrap();
    assert_eq!(
        res_empty.len(),
        4,
        "Empty prefix scan should return all 4 entries"
    );

    // Scan with 0xFF 0xFF prefix
    let res_ff = storage.scan_prefix(b"\xFF\xFF").await.unwrap();
    assert_eq!(
        res_ff.len(),
        2,
        "0xFF 0xFF prefix scan should return 2 entries"
    );
    assert_eq!(res_ff[0].0, b"\xFF\xFFkey1");
    assert_eq!(res_ff[1].0, b"\xFF\xFFkey2");

    // Scan with prefix ending in 0xFF
    let res_abc_ff = storage.scan_prefix(b"abc\xFF").await.unwrap();
    assert_eq!(
        res_abc_ff.len(),
        1,
        "abc\\xFF prefix scan should return 1 entry"
    );
    assert_eq!(res_abc_ff[0].0, b"abc\xFFkey");
}

#[tokio::test]
async fn test_memtable_get_at_seq_with_tombstone_bit_in_param() {
    let mt = MemTable::new();

    // Put a key with sequence number 10
    mt.put(Bytes::from("key1"), Bytes::from("val1"), 10, 1);
    // Put another version with sequence number 20
    mt.put(Bytes::from("key1"), Bytes::from("val2"), 20, 2);

    // Call get_at_seq passing seq_no with TOMBSTONE_BIT set (e.g. 15 | TOMBSTONE_BIT)
    // Expectation: get_at_seq should mask seq_no and return val1 (seq 10 <= 15)
    let res = mt.get_at_seq(b"key1", 15 | TOMBSTONE_BIT, u64::MAX);
    assert!(
        res.is_some(),
        "get_at_seq should succeed when seq_no param has TOMBSTONE_BIT set"
    );
    let (val, seq, tx) = res.unwrap();
    assert_eq!(val.as_ref(), b"val1");
    assert_eq!(seq, 10);
    assert_eq!(tx, 1);
}

#[tokio::test]
async fn test_model_btreemap_reference_comparison() {
    let (storage, _tmp) = test_storage().await;

    // Pin snapshot at seq 0 so flush retains historical MVCC versions
    let _snap_guard = storage.snapshot_registry.register(0);

    // Reference model for snapshot sequence numbers
    let mut ref_model: BTreeMap<u64, BTreeMap<Vec<u8>, Vec<u8>>> = BTreeMap::new();
    let mut current_state: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();

    // Perform operations with distinct transaction IDs
    let ops = vec![
        ("put", b"key1".to_vec(), b"v1_tx1".to_vec(), 10u64),
        ("put", b"key2".to_vec(), b"v2_tx2".to_vec(), 11u64),
        ("put", b"key3".to_vec(), b"v3_tx3".to_vec(), 12u64),
        ("delete", b"key2".to_vec(), vec![], 20u64),
        ("put", b"key1".to_vec(), b"v1_tx5".to_vec(), 30u64),
        ("put", b"key4".to_vec(), b"v4_tx6".to_vec(), 31u64),
    ];

    let mut checkpoints = Vec::new();

    for (op, k, v, tx_num) in ops {
        let tx = TxId::new(tx_num);
        if op == "put" {
            storage.put(tx, &k, &v).await.unwrap();
            current_state.insert(k, v);
        } else if op == "delete" {
            storage.delete(tx, &k).await.unwrap();
            current_state.remove(&k);
        }
        storage.commit(tx).await.unwrap();
        let seq = storage.last_seq_no().await.unwrap();
        ref_model.insert(seq, current_state.clone());
        checkpoints.push(seq);

        if tx_num == 20 {
            storage.force_flush().await.unwrap();
        }
    }

    // Verify snapshot isolation at every checkpoint
    for &seq in &checkpoints {
        let expected_map = ref_model.get(&seq).unwrap();

        // Check point lookups
        for (k, v) in expected_map {
            let actual_v = storage.get_at_seq(k, seq).await.unwrap();
            assert_eq!(
                actual_v.as_deref(),
                Some(v.as_slice()),
                "Mismatch for key {:?} at seq {}",
                String::from_utf8_lossy(k),
                seq
            );
        }

        // Check prefix scan
        let actual_scanned = storage.scan_prefix_at(b"key", seq).await.unwrap();
        let actual_scanned_map: BTreeMap<_, _> = actual_scanned.into_iter().collect();

        assert_eq!(
            actual_scanned_map.len(),
            expected_map.len(),
            "Len mismatch at seq {}",
            seq
        );
        for (k, v) in expected_map {
            assert_eq!(
                actual_scanned_map.get(k),
                Some(v),
                "Scan mismatch for key {:?} at seq {}",
                String::from_utf8_lossy(k),
                seq
            );
        }
    }
}

#[tokio::test]
async fn test_concurrent_readers_during_repeated_force_flush() {
    let (storage, _tmp) = test_storage().await;
    let storage = Arc::new(storage);

    // Initial setup
    for i in 0..100 {
        let tx = TxId::new(1);
        let key = format!("k:{:04}", i);
        let val = format!("v:{:04}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(TxId::new(1)).await.unwrap();

    let storage_writer = Arc::clone(&storage);
    let writer_handle = tokio::spawn(async move {
        for tx_num in 2..=50u64 {
            let tx = TxId::new(tx_num);
            for i in 0..10 {
                let key = format!("k:{:04}", (tx_num * 10 + i) % 100);
                let val = format!("v_tx{}:{:04}", tx_num, i);
                storage_writer
                    .put(tx, key.as_bytes(), val.as_bytes())
                    .await
                    .unwrap();
            }
            storage_writer.commit(tx).await.unwrap();
            if tx_num % 5 == 0 {
                storage_writer.force_flush().await.unwrap();
            }
        }
    });

    let storage_reader = Arc::clone(&storage);
    let reader_handle = tokio::spawn(async move {
        for _ in 0..100 {
            let (results, _) = storage_reader
                .scan_bounded(Bound::Unbounded, Bound::Unbounded, 200, None)
                .await
                .unwrap();

            // Check no gaps, always 100 unique keys
            let mut seen_keys = std::collections::HashSet::new();
            for (k, _v) in &results {
                seen_keys.insert(k.clone());
            }

            assert_eq!(
                seen_keys.len(),
                100,
                "Concurrent scan during flush must see exactly 100 unique keys without gap or duplicate"
            );

            tokio::task::yield_now().await;
        }
    });

    writer_handle.await.unwrap();
    reader_handle.await.unwrap();
}
