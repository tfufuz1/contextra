//! # Startup & Replay Recovery Tests (Ring 1)
//!
//! ZWECK: Pruefung von WAL-Discovery, Crash-Recovery beim Startup und Bereinigung unvollstaendiger Transaktionen.

use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_recovery_scan_ignores_and_removes_tmp_files() {
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

    let tmp_file_1 = tmp.path().join("sst-00000000000000000001-000001.tmp");
    let tmp_file_2 = tmp.path().join("some_other_file.tmp");
    tokio::fs::write(&tmp_file_1, b"corrupted data")
        .await
        .unwrap();
    tokio::fs::write(&tmp_file_2, b"corrupted data")
        .await
        .unwrap();

    let storage = LsmStorage::new(config.clone()).await.unwrap();

    let ssts = storage.sstables.read().await;
    assert_eq!(ssts.len(), 0);
    drop(ssts);

    assert!(!tmp_file_1.exists(), "tmp_file_1 must be deleted");
    assert!(!tmp_file_2.exists(), "tmp_file_2 must be deleted");
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
        let storage = LsmStorage::new(config.clone()).await.unwrap();
        let tx = TxId::new(1);
        storage.put(tx, b"k_persist", b"v_persist").await.unwrap();
        storage.commit(tx).await.unwrap();
    }

    {
        let storage = LsmStorage::new(config.clone()).await.unwrap();
        let val = storage.get(b"k_persist").await.unwrap();
        assert_eq!(val, Some(bytes::Bytes::from_static(b"v_persist")));

        let ssts = storage.sstables.read().await;
        assert_eq!(ssts.len(), 1, "Startup flush must have created an SSTable");
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
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.unwrap();

    for i in 1..=4u64 {
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

    storage.rollback_to_tx(TxId::new(2)).await.unwrap();

    let ssts = storage.sstables.read().await;
    assert!(
        ssts.is_empty(),
        "No SSTable should be created when surviving entries < 8"
    );
    drop(ssts);

    assert_eq!(
        storage.get(b"k01").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v01"))
    );
    assert_eq!(
        storage.get(b"k02").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v02"))
    );
    assert_eq!(storage.get(b"k03").await.unwrap(), None);
    assert_eq!(storage.get(b"k04").await.unwrap(), None);
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

    let old_wal = tmp.path().join("wal-00000000000000000001.log");
    let old_sidecar = tmp.path().join("wal-00000000000000000001.log.uuid");
    tokio::fs::write(&old_wal, b"dummy").await.unwrap();
    tokio::fs::write(&old_sidecar, b"dummy-uuid").await.unwrap();

    let new_wal = tmp.path().join("wal-00000000000000000002.log");
    tokio::fs::write(&new_wal, b"").await.unwrap();

    let salt_path = tmp.path().join("SALT");
    let salt = crate::lsm::recovery::generate_crypto_salt().unwrap();
    crate::lsm::recovery::write_salt_atomically(&salt_path, &salt)
        .await
        .unwrap();

    let _storage = LsmStorage::new(config).await;

    assert!(!old_sidecar.exists(), "WAL UUID sidecar must be removed");
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
        ..Default::default()
    };

    {
        let storage = LsmStorage::new(config.clone()).await.unwrap();
        let tx1 = TxId::new(1);
        storage.put(tx1, b"k1", b"v1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"k2", b"v2").await.unwrap();
        storage.commit(tx2).await.unwrap();

        let intent_path = tmp.path().join(format!("rollback-{:016x}.intent", 1));
        let mut intent_bytes = Vec::with_capacity(16);
        intent_bytes.extend_from_slice(b"MFRLBK\0\0");
        intent_bytes.extend_from_slice(&1u64.to_le_bytes());
        tokio::fs::write(&intent_path, &intent_bytes).await.unwrap();
    }

    {
        let storage = LsmStorage::new(config).await.unwrap();
        assert_eq!(
            storage.get(b"k1").await.unwrap(),
            Some(bytes::Bytes::from_static(b"v1"))
        );
        assert_eq!(storage.get(b"k2").await.unwrap(), None);

        let intent_path = tmp.path().join(format!("rollback-{:016x}.intent", 1));
        assert!(
            !intent_path.exists(),
            "Rollback intent file must be removed after successful recovery"
        );
    }
}

#[tokio::test]
async fn test_wal_discovery_mixed_filenames() {
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

    let salt_path = tmp.path().join("SALT");
    let salt = crate::lsm::recovery::generate_crypto_salt().unwrap();
    crate::lsm::recovery::write_salt_atomically(&salt_path, &salt)
        .await
        .unwrap();

    let storage = LsmStorage::new(config).await.unwrap();
    assert_eq!(storage.health(), StorageHealth::Healthy);
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
    let storage = LsmStorage::new(config).await.unwrap();

    for i in 1..=5u64 {
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

    storage.rollback_to_tx(TxId::new(3)).await.unwrap();

    assert_eq!(
        storage.get(b"k01").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v01"))
    );
    assert_eq!(
        storage.get(b"k02").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v02"))
    );
    assert_eq!(
        storage.get(b"k03").await.unwrap(),
        Some(bytes::Bytes::from_static(b"v03"))
    );
    assert_eq!(storage.get(b"k04").await.unwrap(), None);
    assert_eq!(storage.get(b"k05").await.unwrap(), None);
}

#[tokio::test]
async fn test_uncommitted_transaction_discarded_on_open_recovery() {
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

    let salt_path = tmp.path().join("SALT");
    let salt = crate::lsm::recovery::generate_crypto_salt().unwrap();
    crate::lsm::recovery::write_salt_atomically(&salt_path, &salt)
        .await
        .unwrap();

    {
        let storage = LsmStorage::new(config.clone()).await.unwrap();
        let tx1 = TxId::new(1);
        storage.put(tx1, b"committed_k", b"committed_v").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"uncommitted_k", b"uncommitted_v").await.unwrap();
        // tx2 not committed
    }

    {
        let storage = LsmStorage::new(config).await.unwrap();
        assert_eq!(
            storage.get(b"committed_k").await.unwrap(),
            Some(bytes::Bytes::from_static(b"committed_v"))
        );
        assert_eq!(storage.get(b"uncommitted_k").await.unwrap(), None);
    }
}

#[tokio::test]
async fn test_aborted_transaction_discarded_on_open_recovery() {
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

    let salt_path = tmp.path().join("SALT");
    let salt = crate::lsm::recovery::generate_crypto_salt().unwrap();
    crate::lsm::recovery::write_salt_atomically(&salt_path, &salt)
        .await
        .unwrap();

    {
        let storage = LsmStorage::new(config.clone()).await.unwrap();
        let tx1 = TxId::new(1);
        storage.put(tx1, b"k_alive", b"v_alive").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"k_aborted", b"v_aborted").await.unwrap();
        storage.rollback(tx2).await.unwrap();
    }

    {
        let storage = LsmStorage::new(config).await.unwrap();
        assert_eq!(
            storage.get(b"k_alive").await.unwrap(),
            Some(bytes::Bytes::from_static(b"v_alive"))
        );
        assert_eq!(storage.get(b"k_aborted").await.unwrap(), None);
    }
}
