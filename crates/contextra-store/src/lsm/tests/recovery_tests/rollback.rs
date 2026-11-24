//! # Rollback & SSTable Recovery Tests (Ring 1)
//!
//! ZWECK: Pruefung von Rollbacks auf Transaktionen und selektiver SSTable-Rekompaktierung.

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

        let res_rb = storage.rollback_to_tx(tx1).await;
        eprintln!("ROLLBACK RESULT: {:?}", res_rb);
        res_rb.expect("rollback");

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
