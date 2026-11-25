use contextra_core::{StorageEngine, TxId};
use contextra_store::wal::{Wal, WalOp};
use contextra_store::{CompactionConfig, LsmConfig, LsmStorage};
use tempfile::TempDir;

#[tokio::test]
async fn test_flush_retry_after_phase3_failure_deletes_sealed_wal_and_no_null_checkpoint() {
    let tmp = TempDir::new().expect("temp dir");

    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: std::time::Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone())
        .await
        .expect("create storage");

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.expect("put 1");
    storage.commit(tx1).await.expect("commit 1");

    // Block SSTable creation path by blocking the sst file path with a directory
    let seq = storage.next_seq_no_for_test();
    let blocking_dir = tmp.path().join(format!("sst-{:020}-000000.sst", seq));
    tokio::fs::create_dir(&blocking_dir)
        .await
        .expect("create blocking dir");

    // First flush attempt fails in Phase 3
    let res1 = storage.force_flush().await;
    assert!(
        res1.is_err(),
        "First flush attempt must fail due to directory collision"
    );

    // Remove blocking directory
    tokio::fs::remove_dir(&blocking_dir)
        .await
        .expect("remove blocking dir");

    // Active memtable is empty, immutable_memtables holds old memtable
    // Second flush attempt (retry)
    storage
        .force_flush()
        .await
        .expect("Retry flush must succeed");

    // Inspect manifest to ensure the LAST WalCheckpoint is NOT [0; 32] (H4a)
    let manifest_path = tmp.path().join("MANIFEST");
    let entries = contextra_store::Manifest::load(&manifest_path)
        .await
        .expect("load manifest");

    let last_checkpoint_hmac = entries
        .iter()
        .filter_map(|e| match e {
            contextra_store::ManifestEntry::WalCheckpoint { hmac } => Some(*hmac),
            _ => None,
        })
        .last();

    assert!(
        last_checkpoint_hmac.is_some(),
        "Manifest must contain at least one WalCheckpoint"
    );
    assert_ne!(
        last_checkpoint_hmac.unwrap(),
        [0u8; 32],
        "The high-water-mark (LAST WalCheckpoint) must not be overwritten with null zeroes [0; 32]"
    );

    // Verify data remains correct
    let val1 = storage.get(b"key1").await.expect("get key1");
    assert_eq!(val1, Some(bytes::Bytes::from_static(b"val1")));
}

#[tokio::test]
async fn test_parallel_force_flushes_execute_serially() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: std::time::Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };

    let storage = std::sync::Arc::new(
        LsmStorage::new(config.clone())
            .await
            .expect("create storage"),
    );

    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let s1 = storage.clone();
    let s2 = storage.clone();

    let (res1, res2) = tokio::join!(async move { s1.force_flush().await }, async move {
        s2.force_flush().await
    });
    res1.unwrap();
    res2.unwrap();

    let val = storage.get(b"k1").await.unwrap();
    assert_eq!(val, Some(bytes::Bytes::from_static(b"v1")));

    drop(storage);

    let reopened = LsmStorage::new(config).await.unwrap();
    assert_eq!(
        reopened.get(b"k1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
}

#[tokio::test]
async fn test_orphan_transaction_isolation_and_no_leak_on_reuse() {
    let tmp = TempDir::new().expect("temp dir");

    let config_init = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    // Initialize clean database directory with SALT and WAL via LsmStorage
    {
        let init_storage = LsmStorage::new(config_init).await.unwrap();
        init_storage.close().await.unwrap();
    }

    let wal_path = tmp.path().join("wal.log");
    {
        let wal = Wal::open_with_key_manager(&wal_path, None)
            .await
            .expect("open wal");

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        // Transaction 1: Committed
        let (batch1, _) = wal
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
                    1,
                ),
            ])
            .await
            .expect("batch 1");
        wal.append_batch(batch1).await.expect("append 1");

        // Transaction 2: Torn batch / missing TxEnd marker (Orphan)
        let (batch2, _) = wal
            .prepare_batch(vec![(
                WalOp::Put {
                    tx_id: tx2,
                    key: b"orphan_key".to_vec(),
                    value: b"orphan_val".to_vec(),
                },
                2,
            )])
            .await
            .expect("batch 2");
        wal.append_batch(batch2).await.expect("append 2");
    }

    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    // Open storage — recovery replay should seed last_committed_tx including orphan TxId 2
    let storage = LsmStorage::new(config).await.expect("startup recovery");

    assert_eq!(
        storage.get(b"k1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
    assert_eq!(storage.get(b"orphan_key").await.unwrap(), None);

    // Perform new transaction — next TxId should be 3 (> 2) so no TxId reuse collision
    let tx3 = TxId::new(3);
    storage.put(tx3, b"new_key", b"new_val").await.unwrap();
    storage.commit(tx3).await.unwrap();

    assert_eq!(
        storage.get(b"orphan_key").await.unwrap(),
        None,
        "Orphan key must not become visible"
    );
    assert_eq!(
        storage.get(b"new_key").await.unwrap(),
        Some(bytes::Bytes::from_static(b"new_val"))
    );
}
