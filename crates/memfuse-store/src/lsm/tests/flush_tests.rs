use super::*;
use std::sync::atomic::Ordering;

#[tokio::test]
async fn test_sstable_ordering_after_consecutive_flushes() {
    let (storage, _tmp) = test_storage().await;

    for i in 1..=10u64 {
        let tx = TxId::new(i);
        let val = format!("val-{}", i);
        storage
            .put(tx, b"seq_key", val.as_bytes())
            .await
            .expect("put");
        storage.commit(tx).await.expect("commit");
        storage.force_flush().await.expect("flush");

        let current_val = storage.get(b"seq_key").await.expect("get");
        assert_eq!(
            current_val,
            Some(bytes::Bytes::from(val)),
            "After flush {}, get must return latest value",
            i
        );
    }

    let final_val = storage.get(b"seq_key").await.expect("final get");
    assert_eq!(final_val, Some(bytes::Bytes::from_static(b"val-10")));
}

#[tokio::test]
async fn test_flush_creates_sstable() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 64,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let tx = TxId::new(1);
    for i in 0..10u8 {
        let key = format!("key-{:03}", i);
        let val = format!("value-{:03}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .expect("put");
    }
    storage.commit(tx).await.expect("commit");

    for i in 0..10u8 {
        let key = format!("key-{:03}", i);
        let expected = format!("value-{:03}", i);
        let val = storage.get(key.as_bytes()).await.expect("get");
        assert_eq!(
            val,
            Some(bytes::Bytes::from(expected)),
            "key {} missing after flush",
            key
        );
    }

    let stats = storage.stats().await.expect("stats");
    assert!(
        stats.num_segments > 0,
        "Expected at least one SSTable segment after flush"
    );
}

#[tokio::test]
async fn test_pin_unpin_checkpoint_prevents_gc() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig {
            min_sstables_per_tier: 2,
            size_ratio: 4.0,
            check_interval: Duration::from_secs(30),
            yield_threshold: 1000,
            max_memory_bytes: Some(1024 * 1024),
            ..Default::default()
        },
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config.clone())
        .await
        .expect("create storage");

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();
    let seq1 = storage.last_seq_no().await.unwrap();

    storage.pin_checkpoint(seq1).await.expect("pin");

    let tx2 = TxId::new(2);
    storage.delete(tx2, b"key1").await.unwrap();
    storage.commit(tx2).await.unwrap();

    storage.force_flush().await.unwrap();

    let engine = CompactionEngine::new(
        config.compaction.clone(),
        storage.snapshot_registry.clone(),
        storage.block_cache.clone(),
        storage.key_manager.clone(),
        Arc::clone(&storage.budget),
        Some(Arc::clone(&storage.manifest)),
    );

    engine
        .maybe_compact(&storage.sstables, &storage.config.path)
        .await
        .expect("compact");

    assert_eq!(storage.snapshot_registry.min_active_seqno(), seq1);

    storage.unpin_checkpoint(seq1).await.expect("unpin");
    assert_eq!(storage.snapshot_registry.min_active_seqno(), u64::MAX);

    engine
        .maybe_compact(&storage.sstables, &storage.config.path)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_compaction_roundtrip() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig {
            min_sstables_per_tier: 2,
            size_ratio: 2.0,
            check_interval: Duration::from_secs(3600),
            yield_threshold: 100,
            max_memory_bytes: Some(1024 * 1024),
            ..Default::default()
        },
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config.clone())
        .await
        .expect("create storage");

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();
    storage.force_flush().await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"key2", b"val2").await.unwrap();
    storage.commit(tx2).await.unwrap();
    storage.force_flush().await.unwrap();

    let compact_res = storage.maybe_compact().await.expect("compact");
    assert!(compact_res, "Compaction should occur");

    assert_eq!(storage.get(b"key1").await.unwrap(), Some(bytes::Bytes::from_static(b"val1")));
    assert_eq!(storage.get(b"key2").await.unwrap(), Some(bytes::Bytes::from_static(b"val2")));
}

#[tokio::test]
async fn test_close_durability() {
    let tmp = TempDir::new().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };

    {
        let storage = LsmStorage::new(config.clone()).await.unwrap();
        let tx = TxId::new(1);
        storage.put(tx, b"close_key", b"close_val").await.unwrap();
        storage.commit(tx).await.unwrap();
        storage.close().await.unwrap();
    }

    {
        let storage = LsmStorage::new(config).await.unwrap();
        let val = storage.get(b"close_key").await.unwrap();
        assert_eq!(val, Some(bytes::Bytes::from_static(b"close_val")));
    }
}

#[tokio::test]
async fn test_flush_phase3_failure_retains_immutable_memtable_and_data() {
    let (storage, tmp) = test_storage().await;
    let tx = TxId::new(1);
    storage.put(tx, b"key1", b"val1").await.unwrap();
    storage.commit(tx).await.unwrap();

    let initial_budget_used = storage.budget.memory_used();

    let seq = storage.next_seq_no.load(Ordering::Relaxed);
    let count = storage.segment_counter.load(Ordering::Relaxed);
    let sst_path = tmp
        .path()
        .join(format!("sst-{:020}-{:06}.sst", seq, count % 1_000_000));
    tokio::fs::create_dir(&sst_path).await.unwrap();

    let res = storage.force_flush().await;
    assert!(
        res.is_err(),
        "Flush must return error when SSTable creation fails"
    );

    let state = storage.state.read().await;
    assert_eq!(
        state.immutable_memtables.len(),
        1,
        "immutable_memtables must retain old memtable on Phase 3 flush failure"
    );
    drop(state);

    assert_eq!(
        storage.budget.memory_used(),
        initial_budget_used,
        "Budget memory must not be released on flush failure"
    );

    let val = storage.get(b"key1").await.unwrap();
    assert_eq!(
        val,
        Some(bytes::Bytes::from_static(b"val1")),
        "Key must remain readable from retained immutable memtable after flush failure"
    );
}
