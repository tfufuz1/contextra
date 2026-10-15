//! Integration tests for Serializable Snapshot Isolation (SSI) phantom protection during prefix scans.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_ssi_prefix_phantom_conflict_empty_scan() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("new storage"));

    // Tx1 scans "user:" prefix (initially empty)
    let tx1 = TxId::new(10);
    let results = storage
        .scan_prefix_tracked(tx1, b"user:")
        .await
        .expect("scan_prefix_tracked");
    assert!(results.is_empty(), "Expected empty scan results for user:");

    // Tx2 concurrently inserts "user:new" and commits
    let tx2 = TxId::new(20);
    storage
        .put(tx2, b"user:new", b"alice")
        .await
        .expect("put user:new");
    storage.commit(tx2).await.expect("commit tx2");

    // Tx1 stages a write and attempts commit -> MUST fail with SSI Conflict (Phantom)
    storage
        .put(tx1, b"other:1", b"val1")
        .await
        .expect("put other:1 tx1");
    let res1 = storage.commit(tx1).await;

    assert!(
        matches!(res1, Err(ContextraError::Conflict(_))),
        "Expected Conflict on phantom prefix scan insert, got: {:?}",
        res1
    );
}

#[tokio::test]
async fn test_ssi_prefix_no_conflict_outside_prefix() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("new storage"));

    // Tx1 scans "user:" prefix
    let tx1 = TxId::new(100);
    let _results = storage
        .scan_prefix_tracked(tx1, b"user:")
        .await
        .expect("scan_prefix_tracked");

    // Tx2 inserts "order:1" (outside "user:" prefix) and commits
    let tx2 = TxId::new(200);
    storage
        .put(tx2, b"order:1", b"order_data")
        .await
        .expect("put order:1");
    storage.commit(tx2).await.expect("commit tx2");

    // Tx1 stages a write and attempts commit -> MUST succeed without conflict
    storage
        .put(tx1, b"other:2", b"val2")
        .await
        .expect("put other:2 tx1");
    let res1 = storage.commit(tx1).await;

    assert!(
        res1.is_ok(),
        "Expected Ok commit when concurrent insert is outside scanned prefix, got: {:?}",
        res1
    );
}
