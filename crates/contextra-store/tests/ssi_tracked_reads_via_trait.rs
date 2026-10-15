//! Integration tests for SSI tracked reads via trait objects and TenantScopedStorage.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{ContextraError, TenantId, TxId};
use contextra_ports::StorageEngine;
use contextra_store::{tenant_codec::TenantScopedStorage, LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_ssi_conflict_via_storage_engine_trait() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let lsm = Arc::new(LsmStorage::new(config).await.expect("lsm init"));

    assert!(
        lsm.supports_ssi_tracking(),
        "LsmStorage supports ssi tracking"
    );

    let tenant = TenantId::try_new(42).unwrap();
    let store: Arc<dyn StorageEngine> = Arc::new(TenantScopedStorage::new(lsm, tenant));

    assert!(
        store.supports_ssi_tracking(),
        "TenantScopedStorage supports ssi tracking"
    );

    // Init doc
    let tx0 = TxId::new(1);
    store.put(tx0, b"k1", b"v0").await.expect("put k1");
    store.commit(tx0).await.expect("commit tx0");

    // Tx1 performs tracked read via StorageEngine trait object
    let tx1 = TxId::new(10);
    let val1 = store.get_tracked(tx1, b"k1").await.expect("get_tracked");
    assert_eq!(val1, Some(bytes::Bytes::from_static(b"v0")));

    // Tx2 concurrently mutates k1
    let tx2 = TxId::new(20);
    store.put(tx2, b"k1", b"v2").await.expect("put k1 tx2");
    store.commit(tx2).await.expect("commit tx2");

    // Tx1 attempts to stage write and commit -> must trigger ContextraError::Conflict
    store.put(tx1, b"k2", b"v1_tx1").await.expect("put k2 tx1");
    let res1 = store.commit(tx1).await;

    assert!(
        matches!(res1, Err(ContextraError::Conflict(_))),
        "Expected Conflict on trait object tracked read, got: {:?}",
        res1
    );
}

#[tokio::test]
async fn test_ssi_scan_prefix_tracked_via_tenant_scoped_storage() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let lsm = Arc::new(LsmStorage::new(config).await.expect("lsm init"));
    let tenant = TenantId::try_new(99).unwrap();
    let store: Arc<dyn StorageEngine> = Arc::new(TenantScopedStorage::new(lsm, tenant));

    // Init tenant entries
    let tx0 = TxId::new(1);
    store.put(tx0, b"pfx:1", b"v1").await.expect("put pfx:1");
    store.commit(tx0).await.expect("commit tx0");

    // Tx1 scans prefix tracked via trait object
    let tx1 = TxId::new(100);
    let scanned = store
        .scan_prefix_tracked(tx1, b"pfx:")
        .await
        .expect("scan_prefix_tracked");
    assert_eq!(scanned.len(), 1);
    assert_eq!(scanned[0].0, b"pfx:1".to_vec());

    // Tx2 writes to pfx:1
    let tx2 = TxId::new(200);
    store
        .put(tx2, b"pfx:1", b"v2")
        .await
        .expect("put pfx:1 tx2");
    store.commit(tx2).await.expect("commit tx2");

    // Tx1 attempts commit -> Conflict
    store
        .put(tx1, b"pfx:2", b"v1_tx1")
        .await
        .expect("put pfx:2 tx1");
    let res = store.commit(tx1).await;

    assert!(
        matches!(res, Err(ContextraError::Conflict(_))),
        "Expected Conflict on TenantScopedStorage scan_prefix_tracked, got: {:?}",
        res
    );
}
