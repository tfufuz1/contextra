use super::*;
use std::sync::atomic::Ordering;
use tempfile::TempDir;

#[tokio::test]
async fn test_lsm_rollback_persistence() {
    let tmp = TempDir::new().expect("temp dir");
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
        let storage = LsmStorage::new(config.clone())
            .await
            .expect("create storage");

        let tx1 = TxId::new(1);
        storage.put(tx1, b"k1", b"v1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"k2", b"v2").await.unwrap();
        storage.commit(tx2).await.unwrap();

        assert_eq!(
            storage.get(b"k1").await.unwrap(),
            Some(bytes::Bytes::from_static(b"v1"))
        );
        assert_eq!(
            storage.get(b"k2").await.unwrap(),
            Some(bytes::Bytes::from_static(b"v2"))
        );

        storage.rollback_to_tx(tx1).await.expect("rollback");

        assert_eq!(
            storage.get(b"k1").await.unwrap(),
            Some(bytes::Bytes::from_static(b"v1"))
        );
        assert_eq!(storage.get(b"k2").await.unwrap(), None);
    }

    {
        let storage = LsmStorage::new(config).await.expect("restart storage");
        assert_eq!(
            storage.get(b"k1").await.unwrap(),
            Some(bytes::Bytes::from_static(b"v1"))
        );
        assert_eq!(
            storage.get(b"k2").await.unwrap(),
            None,
            "k2 should NOT be replayed after rollback"
        );

        let tx3 = TxId::new(3);
        storage.put(tx3, b"k3", b"v3").await.unwrap();
        storage.commit(tx3).await.unwrap();
        assert_eq!(
            storage.get(b"k3").await.unwrap(),
            Some(bytes::Bytes::from_static(b"v3"))
        );
    }
}

#[tokio::test]
async fn test_rollback_with_sstables() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"k2", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    storage.force_flush().await.unwrap();

    let tx3 = TxId::new(3);
    storage.put(tx3, b"k3", b"v3").await.unwrap();
    storage.commit(tx3).await.unwrap();

    let tx4 = TxId::new(4);
    storage.put(tx4, b"k4", b"v4").await.unwrap();
    storage.commit(tx4).await.unwrap();

    storage.force_flush().await.unwrap();

    {
        let sstables = storage.sstables.read().await;
        assert_eq!(sstables.len(), 2);
    }

    storage.rollback_to_tx(tx2).await.expect("rollback");

    {
        let sstables = storage.sstables.read().await;
        assert_eq!(sstables.len(), 1, "SSTable 2 should be deleted");
        assert_eq!(sstables[0].metadata().max_tx_id, 2);
    }

    assert_eq!(
        storage.get(b"k1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
    let val2 = storage.get(b"k2").await.unwrap();
    let ssts = storage.sstables.read().await;
    let sst_meta = if !ssts.is_empty() {
        format!(
            "min_tx: {}, max_tx: {}, range: [{:?}, {:?}]",
            ssts[0].metadata().min_tx_id,
            ssts[0].metadata().max_tx_id,
            ssts[0].metadata().first_key,
            ssts[0].metadata().last_key
        )
    } else {
        "NO SSTABLES".into()
    };
    assert_eq!(
        val2,
        Some(bytes::Bytes::from_static(b"v2")),
        "k2 should be found. SST 0 meta: {}",
        sst_meta
    );
    assert_eq!(storage.get(b"k3").await.unwrap(), None);
    assert_eq!(storage.get(b"k4").await.unwrap(), None);
}

#[tokio::test]
async fn test_rollback_recompacts_spanning_sstable() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    for i in 1..=15u64 {
        let tx = TxId::new(i);
        let key = format!("k{:02}", i);
        let val = format!("v{:02}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
        storage.commit(tx).await.unwrap();
    }
    storage.force_flush().await.unwrap();

    {
        let sstables = storage.sstables.read().await;
        assert_eq!(sstables.len(), 1);
        assert_eq!(sstables[0].metadata().min_tx_id, 1);
        assert_eq!(sstables[0].metadata().max_tx_id, 15);
    }

    storage
        .rollback_to_tx(TxId::new(10))
        .await
        .expect("rollback");

    {
        let sstables = storage.sstables.read().await;
        assert_eq!(
            sstables.len(),
            1,
            "Spanning SSTable should be recompacted into 1 new SSTable"
        );
        assert_eq!(sstables[0].metadata().max_tx_id, 10);

        let mut count = 0;
        let mut stream = sstables[0].stream().await.unwrap();
        while let Some((_k, _v, _seq, tx)) = stream.next_entry().await.unwrap() {
            assert!(
                tx <= 10,
                "SSTable on disk must not contain entries with tx_id > 10"
            );
            count += 1;
        }
        assert_eq!(
            count, 10,
            "Surviving on-disk entry count must equal exactly 10"
        );
    }

    for i in 1..=10u64 {
        let key = format!("k{:02}", i);
        let expected = format!("v{:02}", i);
        let val = storage.get(key.as_bytes()).await.unwrap();
        assert_eq!(val, Some(bytes::Bytes::from(expected)));
    }

    for i in 11..=15u64 {
        let key = format!("k{:02}", i);
        let val = storage.get(key.as_bytes()).await.unwrap();
        assert_eq!(val, None);
    }
}

#[tokio::test]
async fn test_rollback_drops_sstable_fully_stale_after_recompaction() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    for i in 10..=15u64 {
        let tx = TxId::new(i);
        let key = format!("k{:02}", i);
        let val = format!("v{:02}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
        storage.commit(tx).await.unwrap();
    }
    storage.force_flush().await.unwrap();

    storage
        .rollback_to_tx(TxId::new(5))
        .await
        .expect("rollback");

    {
        let sstables = storage.sstables.read().await;
        assert!(sstables.is_empty(), "Fully stale SSTable must be dropped");
    }
}

#[tokio::test]
async fn test_wal_survives_process_restart() {
    let tmp = TempDir::new().expect("temp dir");
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
        let storage = LsmStorage::new(config.clone())
            .await
            .expect("create storage");
        let tx = TxId::new(1);
        storage
            .put(tx, b"persistent_key", b"persistent_val")
            .await
            .expect("put");
        storage.commit(tx).await.expect("commit");
    }

    {
        let storage = LsmStorage::new(config).await.expect("reopen storage");
        let val = storage.get(b"persistent_key").await.expect("get");
        assert_eq!(val, Some(bytes::Bytes::from_static(b"persistent_val")));
    }
}

#[tokio::test]
async fn test_rollback_to_tx_edge_cases() {
    let (storage, _tmp) = test_storage().await;

    let res = storage.rollback_to_tx(TxId::new(999)).await;
    assert!(res.is_ok());

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.expect("put");
    storage.commit(tx1).await.expect("commit");

    storage
        .rollback_to_tx(TxId::new(0))
        .await
        .expect("rollback to 0");
    assert_eq!(storage.get(b"key1").await.expect("get"), None);
}

#[tokio::test]
async fn test_rollback_tombstone_sstable() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"key2", b"val2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    let tx3 = TxId::new(3);
    storage.delete(tx3, b"key2").await.unwrap();
    storage.commit(tx3).await.unwrap();
    storage.force_flush().await.unwrap();

    storage.rollback_to_tx(tx3).await.unwrap();

    let tx4 = TxId::new(4);
    storage.put(tx4, b"key3", b"val3").await.unwrap();
    storage.commit(tx4).await.unwrap();

    let current_max_seq = storage.next_seq_no.load(Ordering::Acquire);
    let val = storage.get_at_seq(b"key3", current_max_seq).await.unwrap();
    assert_eq!(val, Some(bytes::Bytes::from_static(b"val3")));

    let last_seq = storage.last_seq_no().await.unwrap();
    assert_eq!(
        last_seq & TOMBSTONE_BIT,
        0,
        "Sequence number of new insert must not have TOMBSTONE_BIT set"
    );
}

#[tokio::test]
async fn test_rollback_tombstone_wal() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"k2", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    let tx3 = TxId::new(3);
    storage.delete(tx3, b"k2").await.unwrap();
    storage.commit(tx3).await.unwrap();

    storage.rollback_to_tx(tx3).await.unwrap();

    let tx4 = TxId::new(4);
    storage.put(tx4, b"k3", b"v3").await.unwrap();
    storage.commit(tx4).await.unwrap();

    let current_max_seq = storage.next_seq_no.load(Ordering::Acquire);
    let val = storage.get_at_seq(b"k3", current_max_seq).await.unwrap();
    assert_eq!(val, Some(bytes::Bytes::from_static(b"v3")));

    let last_seq = storage.last_seq_no().await.unwrap();
    assert_eq!(
        last_seq & TOMBSTONE_BIT,
        0,
        "Sequence number of new insert must not have TOMBSTONE_BIT set"
    );
}

#[tokio::test]
async fn test_rollback_tombstone_subsequent_ops() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    storage.delete(tx2, b"key1").await.unwrap();
    storage.commit(tx2).await.unwrap();

    storage.rollback_to_tx(tx2).await.unwrap();

    let tx3 = TxId::new(3);
    storage.put(tx3, b"key2", b"val2").await.unwrap();
    storage.commit(tx3).await.unwrap();

    let tx4 = TxId::new(4);
    storage.delete(tx4, b"key2").await.unwrap();
    storage.commit(tx4).await.unwrap();

    let tx5 = TxId::new(5);
    storage.put(tx5, b"key3", b"val3").await.unwrap();
    storage.commit(tx5).await.unwrap();

    let state = storage.state.read().await;
    for (k, _v, seq, _tx) in state.memtable.iter_latest() {
        if k.as_ref() == b"key1" || k.as_ref() == b"key2" {
            assert_ne!(
                seq & TOMBSTONE_BIT,
                0,
                "Latest entry for deleted key {:?} must have TOMBSTONE_BIT set",
                String::from_utf8_lossy(&k)
            );
        } else if k.as_ref() == b"key3" {
            assert_eq!(
                seq & TOMBSTONE_BIT,
                0,
                "Latest entry for inserted key {:?} must NOT have TOMBSTONE_BIT set",
                String::from_utf8_lossy(&k)
            );
        }
    }
    drop(state);

    assert_eq!(storage.get(b"key1").await.unwrap(), None);
    assert_eq!(storage.get(b"key2").await.unwrap(), None);
    assert_eq!(
        storage.get(b"key3").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val3"))
    );
}

#[tokio::test]
async fn test_recovery_scan_ignores_and_removes_tmp_files() {
    let tmp = tempfile::TempDir::new().unwrap();
    let corrupt_tmp_path = tmp.path().join("sst-compact-corrupt.sst.tmp");
    tokio::fs::write(&corrupt_tmp_path, b"invalid sst data from crash")
        .await
        .unwrap();

    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: std::time::Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };

    let storage = LsmStorage::new(config)
        .await
        .expect("LsmStorage startup must succeed");
    assert_eq!(storage.sstables.read().await.len(), 0);

    assert!(
        !corrupt_tmp_path.exists(),
        "Leftover .tmp file must be removed during startup recovery scan"
    );
}

#[tokio::test]
async fn test_startup_flush_before_wal_cleanup() {
    let tmp = TempDir::new().expect("temp dir");
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
        let storage = LsmStorage::new(config.clone())
            .await
            .expect("create initial storage");
        let tx1 = TxId::new(1);
        storage.put(tx1, b"key1", b"val1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"key2", b"val2").await.unwrap();
        storage.commit(tx2).await.unwrap();
    }

    {
        let storage = LsmStorage::new(config.clone())
            .await
            .expect("reopen storage after crash/restart");

        assert_eq!(
            storage.get(b"key1").await.unwrap(),
            Some(bytes::Bytes::from_static(b"val1"))
        );
        assert_eq!(
            storage.get(b"key2").await.unwrap(),
            Some(bytes::Bytes::from_static(b"val2"))
        );

        let stats = storage.stats().await.unwrap();
        assert!(
            stats.num_segments >= 1,
            "Startup flush must persist replayed WAL entries into SSTable"
        );
    }

    {
        let storage = LsmStorage::new(config)
            .await
            .expect("reopen storage after second crash");
        assert_eq!(
            storage.get(b"key1").await.unwrap(),
            Some(bytes::Bytes::from_static(b"val1"))
        );
        assert_eq!(
            storage.get(b"key2").await.unwrap(),
            Some(bytes::Bytes::from_static(b"val2"))
        );
    }
}

#[tokio::test]
async fn test_rollback_small_tx_inline_no_sstable() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        group_commit_window_micros: 0,
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

    let sst_count_before = storage.sstables.read().await.len();
    assert_eq!(sst_count_before, 1);

    storage.rollback_to_tx(tx1).await.unwrap();

    assert_eq!(
        storage.get(b"key1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val1"))
    );
    assert_eq!(storage.get(b"key2").await.unwrap(), None);

    let sst_count_after = storage.sstables.read().await.len();
    assert!(
        sst_count_after <= sst_count_before,
        "Small transaction rollback must not produce new SSTables"
    );
}

#[tokio::test]
async fn test_wal_uuid_sidecar_cleaned_up_on_startup() {
    let tmp = TempDir::new().expect("temp dir");
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
    }

    let uuid_path = tmp.path().join("wal-00000000000000000000.log.uuid");
    tokio::fs::write(&uuid_path, b"test-uuid-content")
        .await
        .unwrap();
    let wal1_path = tmp.path().join("wal-00000000000000000001.log");
    tokio::fs::write(&wal1_path, b"").await.unwrap();
    assert!(
        uuid_path.exists(),
        "Dummy .uuid file must exist before startup cleanup"
    );

    {
        let _storage = LsmStorage::new(config).await.expect("reopen storage");

        assert!(
            !uuid_path.exists(),
            "WAL .uuid sidecar file must be cleaned up during startup recovery"
        );
    }
}

#[tokio::test]
async fn test_rollback_crash_recovery_startup() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);
    let tx3 = TxId::new(3);

    {
        let storage = LsmStorage::new(config.clone())
            .await
            .expect("create storage");

        storage.put(tx1, b"key1", b"val1").await.unwrap();
        storage.commit(tx1).await.unwrap();
        storage.force_flush().await.unwrap();

        storage.put(tx2, b"key2", b"val2").await.unwrap();
        storage.commit(tx2).await.unwrap();
        storage.force_flush().await.unwrap();

        storage.put(tx3, b"key3", b"val3").await.unwrap();
        storage.commit(tx3).await.unwrap();
        storage.close().await.unwrap();
    }

    let intent_path = tmp
        .path()
        .join(format!("rollback-{:016x}.intent", tx1.inner()));
    const INTENT_MAGIC: &[u8] = b"MFRLBK\0\0";
    let mut intent_bytes = Vec::with_capacity(16);
    intent_bytes.extend_from_slice(INTENT_MAGIC);
    intent_bytes.extend_from_slice(&tx1.inner().to_le_bytes());
    tokio::fs::write(&intent_path, &intent_bytes).await.unwrap();

    assert!(
        intent_path.exists(),
        "Rollback intent file must exist before startup recovery"
    );

    let storage = LsmStorage::new(config.clone())
        .await
        .expect("reopen storage after simulated rollback crash");

    assert_eq!(
        storage.get(b"key1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val1")),
        "Data committed at target_tx must remain visible"
    );
    assert_eq!(
        storage.get(b"key2").await.unwrap(),
        None,
        "Data committed after target_tx (tx2) must be rolled back"
    );
    assert_eq!(
        storage.get(b"key3").await.unwrap(),
        None,
        "Data committed after target_tx (tx3) must be rolled back"
    );
    assert!(
        !intent_path.exists(),
        "Rollback intent file must be deleted after successful startup recovery"
    );
}

#[tokio::test]
async fn test_wal_discovery_mixed_filenames() {
    let tmp = TempDir::new().expect("temp dir");

    let legacy_wal_path = tmp.path().join("wal.log");
    let wal = Wal::open_with_key_manager(&legacy_wal_path, None)
        .await
        .unwrap();
    let tx1 = TxId::new(1);
    let (entries1, _) = wal
        .prepare_batch(vec![
            (
                WalOp::Put {
                    tx_id: tx1,
                    key: b"k1".to_vec(),
                    value: b"v1".to_vec(),
                },
                1,
            ),
            (
                WalOp::TxEnd {
                    tx_id: tx1,
                    committed: true,
                },
                2,
            ),
        ])
        .await
        .unwrap();
    wal.append_batch(entries1).await.unwrap();
    drop(wal);

    let wal5_path = tmp.path().join("wal-5.log");
    let wal5 = Wal::open_with_key_manager(&wal5_path, None).await.unwrap();
    let tx2 = TxId::new(2);
    let (entries2, _) = wal5
        .prepare_batch(vec![
            (
                WalOp::Put {
                    tx_id: tx2,
                    key: b"k2".to_vec(),
                    value: b"v2".to_vec(),
                },
                3,
            ),
            (
                WalOp::TxEnd {
                    tx_id: tx2,
                    committed: true,
                },
                4,
            ),
        ])
        .await
        .unwrap();
    wal5.append_batch(entries2).await.unwrap();
    drop(wal5);

    let overflow_wal_path = tmp.path().join(format!("wal-{}.log", u128::MAX));
    let wal_overflow = Wal::open_with_key_manager(&overflow_wal_path, None)
        .await
        .unwrap();
    let tx3 = TxId::new(3);
    let (entries3, _) = wal_overflow
        .prepare_batch(vec![
            (
                WalOp::Put {
                    tx_id: tx3,
                    key: b"k3".to_vec(),
                    value: b"v3".to_vec(),
                },
                5,
            ),
            (
                WalOp::TxEnd {
                    tx_id: tx3,
                    committed: true,
                },
                6,
            ),
        ])
        .await
        .unwrap();
    wal_overflow.append_batch(entries3).await.unwrap();
    drop(wal_overflow);

    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config)
        .await
        .expect("LsmStorage startup must succeed with mixed WAL files");

    assert_eq!(
        storage.flush_counter.load(Ordering::Relaxed),
        7,
        "flush_counter should be 7 (initialized to 6 + 1 for startup flush)"
    );
    assert_eq!(
        storage.get(b"k1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
    assert_eq!(
        storage.get(b"k2").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v2"))
    );
    assert_eq!(
        storage.get(b"k3").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v3"))
    );
}

#[tokio::test]
async fn test_uncommitted_transaction_discarded_on_recovery() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };

    let wal_path = tmp.path().join("wal.log");
    {
        let wal = Wal::open_with_key_manager(&wal_path, None)
            .await
            .expect("open wal");

        // Transaction 1: Fully committed with Put + TxEnd
        let tx1 = TxId::new(1);
        let (batch1, _) = wal
            .prepare_batch(vec![
                (
                    WalOp::Put {
                        tx_id: tx1,
                        key: b"committed_key".to_vec(),
                        value: b"committed_val".to_vec(),
                    },
                    1,
                ),
                (
                    WalOp::TxEnd {
                        tx_id: tx1,
                        committed: true,
                    },
                    1,
                ),
            ])
            .await
            .unwrap();
        wal.append_batch(batch1).await.unwrap();

        // Transaction 2: Uncommitted (Put without TxEnd marker)
        let tx2 = TxId::new(2);
        let (batch2, _) = wal
            .prepare_batch(vec![(
                WalOp::Put {
                    tx_id: tx2,
                    key: b"uncommitted_key".to_vec(),
                    value: b"uncommitted_val".to_vec(),
                },
                2,
            )])
            .await
            .unwrap();
        wal.append_batch(batch2).await.unwrap();
    }

    let storage = LsmStorage::new(config)
        .await
        .expect("startup storage recovery");

    assert_eq!(
        storage.get(b"committed_key").await.unwrap(),
        Some(bytes::Bytes::from_static(b"committed_val")),
        "Committed transaction MUST be restored on startup"
    );
    assert_eq!(
        storage.get(b"uncommitted_key").await.unwrap(),
        None,
        "Uncommitted transaction missing TxEnd marker MUST be discarded on startup"
    );
}

#[tokio::test]
async fn test_rollback_spanning_sstable_below_min_entries_threshold() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"k2", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    storage.force_flush().await.unwrap();

    {
        let ssts = storage.sstables.read().await;
        assert_eq!(
            ssts.len(),
            1,
            "Should have 1 spanning SSTable before rollback"
        );
    }

    storage.rollback_to_tx(tx1).await.unwrap();

    {
        let ssts = storage.sstables.read().await;
        assert_eq!(
            ssts.len(),
            0,
            "No new SSTable should be created when surviving entries < 8"
        );
    }

    assert_eq!(
        storage.get(b"k1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
    assert_eq!(storage.get(b"k2").await.unwrap(), None);
}
