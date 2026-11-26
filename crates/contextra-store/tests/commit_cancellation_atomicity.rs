// FILE-CONTEXT: Integration tests verifying atomicity, HMAC chain integrity, and recovery after commit cancellation across various timeout points.
// INVARIANTEN:
// - Pro Transaktion nach Abbruch und Neustart: Entweder alle Schlüssel oder keiner (All-or-Nothing).
// - Baseline-Daten bleiben unbeschädigt intakt.
// - HMAC-Kette ist nach Replay vollständig verifiziert (keine Kettenbrüche).
// - Folge-Commits nach dem Neustart gelingen fehlerfrei.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::timeout;

async fn run_cancellation_atomicity_suite(group_commit_window_micros: u64) {
    let dir = tempdir().expect("tempdir creation failed");
    let path = dir.path().to_path_buf();

    let config = LsmConfig {
        path: path.clone(),
        group_commit_window_micros,
        ..Default::default()
    };

    let mut storage = LsmStorage::new(config.clone())
        .await
        .expect("Initial LsmStorage creation failed");

    // 1. Commit baseline transaction
    let tx_base = TxId::new(1);
    storage
        .put(tx_base, b"baseline_key", b"baseline_val")
        .await
        .expect("put baseline failed");
    storage
        .commit(tx_base)
        .await
        .expect("commit baseline failed");

    // 2. Loop over multiple timeout points from 0 to 3000 microseconds
    let timeout_micros = [0, 10, 50, 100, 250, 500, 1000, 1500, 2000, 3000];
    let mut tx_counter = 10u64;

    for &micros in &timeout_micros {
        tx_counter += 1;
        let tx_cancel = TxId::new(tx_counter);

        let k1 = format!("cancel_tx_{}_k1", tx_counter).into_bytes();
        let k2 = format!("cancel_tx_{}_k2", tx_counter).into_bytes();
        let k3 = format!("cancel_tx_{}_k3", tx_counter).into_bytes();
        let val = format!("val_{}", tx_counter).into_bytes();

        storage
            .put(tx_cancel, &k1, &val)
            .await
            .expect("put k1 failed");
        storage
            .put(tx_cancel, &k2, &val)
            .await
            .expect("put k2 failed");
        storage
            .put(tx_cancel, &k3, &val)
            .await
            .expect("put k3 failed");

        // Attempt commit under timeout
        let _ = timeout(Duration::from_micros(micros), storage.commit(tx_cancel)).await;

        // Simulate crash / restart: drop existing storage handle and reopen
        drop(storage);

        let storage_reopen = LsmStorage::new(config.clone()).await.unwrap_or_else(|e| {
            panic!("Reopen after timeout {micros}µs failed (HMAC or recovery error): {e}")
        });

        // Verify baseline key is intact
        let base_val = storage_reopen
            .get(b"baseline_key")
            .await
            .expect("get baseline key failed");
        assert_eq!(
            base_val,
            Some(bytes::Bytes::from_static(b"baseline_val")),
            "Baseline data corrupted after restart at timeout {}µs",
            micros
        );

        // Verify All-or-Nothing atomicity: either all 3 keys are present, or none
        let has_k1 = storage_reopen
            .get(&k1)
            .await
            .expect("get k1 failed")
            .is_some();
        let has_k2 = storage_reopen
            .get(&k2)
            .await
            .expect("get k2 failed")
            .is_some();
        let has_k3 = storage_reopen
            .get(&k3)
            .await
            .expect("get k3 failed")
            .is_some();

        assert_eq!(
            has_k1, has_k2,
            "Atomicity violation at timeout {}µs: k1 present ({}), k2 present ({})",
            micros, has_k1, has_k2
        );
        assert_eq!(
            has_k2, has_k3,
            "Atomicity violation at timeout {}µs: k2 present ({}), k3 present ({})",
            micros, has_k2, has_k3
        );

        // Verify that subsequent commits succeed after recovery
        tx_counter += 1;
        let tx_post = TxId::new(tx_counter);
        let post_k = format!("post_tx_{}_k", tx_counter).into_bytes();
        let post_v = format!("post_tx_{}_v", tx_counter).into_bytes();

        storage_reopen
            .put(tx_post, &post_k, &post_v)
            .await
            .expect("put post key failed");
        storage_reopen
            .commit(tx_post)
            .await
            .expect("post commit failed after cancellation restart");

        let fetched_post = storage_reopen
            .get(&post_k)
            .await
            .expect("get post key failed");
        assert_eq!(
            fetched_post,
            Some(bytes::Bytes::from(post_v)),
            "Post-commit key must be present after successful commit"
        );

        storage = storage_reopen;
    }

    // Final reopen check
    drop(storage);
    let final_storage = LsmStorage::new(config.clone())
        .await
        .expect("Final reopen failed");
    let base_val = final_storage
        .get(b"baseline_key")
        .await
        .expect("final get baseline failed");
    assert_eq!(
        base_val,
        Some(bytes::Bytes::from_static(b"baseline_val")),
        "Baseline key must remain intact at end of test suite"
    );
}

#[tokio::test]
async fn test_commit_cancellation_atomicity_single_commit() {
    run_cancellation_atomicity_suite(0).await;
}

#[tokio::test]
async fn test_commit_cancellation_atomicity_group_commit() {
    run_cancellation_atomicity_suite(500).await;
}
