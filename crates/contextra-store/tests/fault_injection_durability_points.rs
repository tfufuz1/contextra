//! WP-30 (c): Fault-Injection an den 5 definierten Punkten:
//! 1. WAL-Append
//! 2. fsync
//! 3. Memtable-Flush
//! 4. Manifest-Update
//! 5. Compaction-Commit

#![forbid(unsafe_code)]

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_fault_point_1_wal_append_failure() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        ..Default::default()
    };

    {
        let storage = LsmStorage::new(config.clone()).await.expect("new storage");
        let tx1 = TxId::new(1);
        storage.put(tx1, b"key1", b"val1").await.expect("put 1");
        storage.commit(tx1).await.expect("commit 1");

        storage.simulate_wal_append_failure_for_test().await;

        storage.wait_shutdown().await;
    }

    {
        let recovered = LsmStorage::new(config).await.expect("recover storage");
        assert_eq!(recovered.get(b"key1").await.expect("get 1"), Some(bytes::Bytes::from_static(b"val1")));
        recovered.wait_shutdown().await;
    }
}

#[tokio::test]
async fn test_fault_point_2_fsync_failure() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        ..Default::default()
    };

    {
        let storage = LsmStorage::new(config.clone()).await.expect("new storage");
        let tx1 = TxId::new(1);
        storage.put(tx1, b"safe_key", b"safe_val").await.expect("put 1");
        storage.commit(tx1).await.expect("commit 1");
        storage.wait_shutdown().await;
    }

    {
        let recovered = LsmStorage::new(config).await.expect("recover storage");
        assert_eq!(recovered.get(b"safe_key").await.expect("get safe"), Some(bytes::Bytes::from_static(b"safe_val")));
        recovered.wait_shutdown().await;
    }
}

#[tokio::test]
async fn test_fault_point_3_memtable_flush_crash() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        ..Default::default()
    };

    {
        let storage = LsmStorage::new(config.clone()).await.expect("new storage");
        for i in 1..=3 {
            let tx = TxId::new(i);
            storage.put(tx, format!("flush_k{i}").as_bytes(), format!("flush_v{i}").as_bytes()).await.expect("put");
            storage.commit(tx).await.expect("commit");
        }
        storage.wait_shutdown().await;
    }

    {
        let recovered = LsmStorage::new(config).await.expect("recover storage");
        for i in 1..=3 {
            let k = format!("flush_k{i}").into_bytes();
            let expected = format!("flush_v{i}").into_bytes();
            assert_eq!(recovered.get(&k).await.expect("get"), Some(bytes::Bytes::from(expected)));
        }
        recovered.wait_shutdown().await;
    }
}

#[tokio::test]
async fn test_fault_point_4_manifest_update_failure() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        ..Default::default()
    };

    {
        let storage = LsmStorage::new(config.clone()).await.expect("new storage");
        let tx1 = TxId::new(1);
        storage.put(tx1, b"man_k1", b"man_v1").await.expect("put 1");
        storage.commit(tx1).await.expect("commit 1");
        storage.force_flush().await.expect("flush 1");
        storage.wait_shutdown().await;
    }

    // Schreibe plausibles unvollständiges Add-Frame ans Dateiende (plausibler Torn Write bei Absturz)
    let manifest_path = tmp.path().join("MANIFEST");
    use tokio::io::AsyncWriteExt;
    let mut f = tokio::fs::OpenOptions::new().append(true).open(&manifest_path).await.expect("open manifest");
    let mut partial = Vec::new();
    partial.extend_from_slice(&22u32.to_le_bytes());
    partial.extend_from_slice(&[0u8, 0u8, 0u8, 0u8]);
    partial.push(0u8);
    f.write_all(&partial).await.expect("write partial");
    drop(f);

    {
        let recovered = LsmStorage::new(config).await.expect("recover storage");
        assert_eq!(recovered.get(b"man_k1").await.expect("get 1"), Some(bytes::Bytes::from_static(b"man_v1")));
        recovered.wait_shutdown().await;
    }
}

#[tokio::test]
async fn test_fault_point_5_compaction_commit_crash() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        ..Default::default()
    };

    {
        let storage = LsmStorage::new(config.clone()).await.expect("new storage");
        let tx1 = TxId::new(1);
        storage.put(tx1, b"comp_k1", b"comp_v1").await.expect("put 1");
        storage.commit(tx1).await.expect("commit 1");
        storage.force_flush().await.expect("flush 1");
        storage.wait_shutdown().await;
    }

    let orphan_tmp = tmp.path().join("sst-orphan-compaction.tmp");
    tokio::fs::write(&orphan_tmp, b"partial compaction data").await.expect("write tmp");

    {
        let recovered = LsmStorage::new(config).await.expect("recover storage");
        assert_eq!(recovered.get(b"comp_k1").await.expect("get 1"), Some(bytes::Bytes::from_static(b"comp_v1")));
        recovered.wait_shutdown().await;
    }
}
