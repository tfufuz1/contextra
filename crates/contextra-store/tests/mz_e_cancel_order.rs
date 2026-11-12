//! # Guard-Aufräumreihenfolge & Cancel-Safety Integrationstest (MZ-E)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Prüft die deterministische Bereinigung von Intent-Locks und WAL-HMAC bei Abbruch
//! // INVARIANTEN: ADR-0XX-P0A, INV-STORE-1, INV-LOCK-ORDER-1

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_mz_e_cancel_order_intent_cleanup() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("storage init");

    // Table of test cases for cancel cleanup
    let test_cases = vec![
        ("key_alpha", "val_alpha", 1u64),
        ("key_beta", "val_beta", 2u64),
        ("key_gamma", "val_gamma", 3u64),
    ];

    for (k, v, tx_num) in test_cases {
        let tx = TxId::new(tx_num);
        let key = k.as_bytes();
        let val = v.as_bytes();

        let inserted = storage.put_if_absent(tx, key, val).await.expect("put_if_absent");
        assert!(inserted, "First put_if_absent must succeed");

        // Intent lock must be registered
        let is_second_staged = storage.put_if_absent(TxId::new(tx_num + 100), key, val).await.expect("second");
        assert!(!is_second_staged, "Second transaction must see active intent lock");

        // Rollback explicitly cancels and cleans up intent lock
        storage.rollback(tx).await.expect("rollback");

        // After rollback, key must be available again
        let can_insert_after = storage.put_if_absent(TxId::new(tx_num + 200), key, val).await.expect("after rollback");
        assert!(can_insert_after, "Key must be acquirable after cancel/rollback");
    }

    storage.close().await.expect("close storage");
}

#[tokio::test]
async fn test_mz_e_future_drop_cancels_cleanly() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 20_000,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.expect("storage init"));

    let tx = TxId::new(999);
    storage.put(tx, b"abandoned_key", b"abandoned_val").await.expect("put");

    // Spawn a commit future but drop it early before completion
    let storage_clone = Arc::clone(&storage);
    let handle = tokio::spawn(async move {
        storage_clone.commit(tx).await
    });

    // Abort task immediately
    handle.abort();
    let _ = handle.await;

    // Subsequent transaction must not hang or see corruption
    let tx_next = TxId::new(1000);
    storage.put(tx_next, b"clean_key", b"clean_val").await.expect("put next");
    let res = storage.commit(tx_next).await;
    assert!(res.is_ok(), "Subsequent commit must succeed after aborted task");

    storage.close().await.expect("close storage");
}
