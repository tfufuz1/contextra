// FILE-CONTEXT: ZWECK/INVARIANTEN
// STAND: 2026-10-21
// ZWECK: Hardening & Edge-Case Integrationstests für contextra-store (J03).
// INVARIANTEN: Zero-Panic (P7), P28 Determinismus, Monotone Sichtbarkeit & Rollback-Grenzen.

use contextra_ports::StorageEngine;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, Result, TxId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Barrier;

#[tokio::test]
async fn recovery_rollback_boundary_test() -> Result<()> {
    let tmp = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await?;

    // Boundary 1: target_tx = 0 (TxId::INVALID)
    let tx_zero = TxId::INVALID;
    let res0 = storage.rollback_to_tx(tx_zero).await;
    assert!(
        res0.is_ok(),
        "rollback_to_tx(TxId::INVALID / 0) must succeed without panic"
    );

    // Boundary 2: target_tx = MAX_COLLECTION_SEQUENCE
    let tx_max_seq = TxId::try_new(TxId::MAX_COLLECTION_SEQUENCE).unwrap();
    let res_max_seq = storage.rollback_to_tx(tx_max_seq).await;
    assert!(
        res_max_seq.is_ok(),
        "rollback_to_tx(MAX_COLLECTION_SEQUENCE) must succeed without panic"
    );

    // Boundary 3: target_tx = MAX_COLLECTION_SEQUENCE + 1 (In invalid gap per AGT-GRAPH-001)
    let gap_tx_val = TxId::MAX_COLLECTION_SEQUENCE + 1;
    let try_gap = TxId::try_new(gap_tx_val);
    assert!(
        try_gap.is_err(),
        "TxId::try_new(MAX_COLLECTION_SEQUENCE + 1) must return Err"
    );

    // Boundary 4: target_tx = INTERNAL_BASE - 1 (In invalid gap per AGT-GRAPH-001)
    let gap_tx_val2 = TxId::INTERNAL_BASE - 1;
    let try_gap2 = TxId::try_new(gap_tx_val2);
    assert!(
        try_gap2.is_err(),
        "TxId::try_new(INTERNAL_BASE - 1) must return Err"
    );

    // Boundary 5: target_tx = INTERNAL_BASE
    let tx_internal_base = TxId::internal();
    let res_internal = storage.rollback_to_tx(tx_internal_base).await;
    assert!(
        res_internal.is_ok(),
        "rollback_to_tx(TxId::internal()) must succeed without panic"
    );

    Ok(())
}

#[tokio::test]
async fn compaction_advance_visibility_monotonic_test() -> Result<()> {
    let tmp = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await?;

    // Put initial data and commit at tx 10
    let tx10 = TxId::try_new(10).unwrap();
    storage.put(tx10, b"key1", b"val1").await?;
    storage.commit(tx10).await?;

    let last_tx_1 = storage.last_tx_id().await?;
    assert_eq!(last_tx_1, tx10);

    // Write newer data at tx 20 and flush
    let tx20 = TxId::try_new(20).unwrap();
    storage.put(tx20, b"key2", b"val2").await?;
    storage.commit(tx20).await?;

    storage.flush().await?;

    let last_tx_2 = storage.last_tx_id().await?;
    assert_eq!(last_tx_2, tx20);

    // Visibility must strictly not decrease
    assert!(
        last_tx_2 >= last_tx_1,
        "Visibility must be monotonic non-decreasing: {:?} >= {:?}",
        last_tx_2,
        last_tx_1
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn maintenance_last_tx_concurrent_readers_test() -> Result<()> {
    let tmp = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await?);
    let barrier = Arc::new(Barrier::new(4));
    let observed_max = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();

    let done = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // 3 parallel reader tasks
    for _ in 0..3 {
        let storage_clone = Arc::clone(&storage);
        let barrier_clone = Arc::clone(&barrier);
        let observed_max_clone = Arc::clone(&observed_max);
        let done_clone = Arc::clone(&done);

        handles.push(tokio::spawn(async move {
            barrier_clone.wait().await;
            let mut local_prev = 0u64;
            while !done_clone.load(Ordering::Acquire)
                || observed_max_clone.load(Ordering::Acquire) < 100
            {
                if let Ok(tx) = storage_clone.last_tx_id().await {
                    let val = tx.inner();
                    if val < local_prev {
                        panic!(
                            "Non-monotonic transaction ID observed by reader thread: {} < {}",
                            val, local_prev
                        );
                    }
                    local_prev = val;
                    observed_max_clone.fetch_max(val, Ordering::AcqRel);
                }
                tokio::task::yield_now().await;
            }
        }));
    }

    // 1 writer task
    let storage_writer = Arc::clone(&storage);
    let barrier_writer = Arc::clone(&barrier);
    let done_writer = Arc::clone(&done);

    handles.push(tokio::spawn(async move {
        barrier_writer.wait().await;
        for i in 1..=100 {
            let tx = TxId::try_new(i).unwrap();
            let _ = storage_writer.put(tx, b"key_concurrent", b"val").await;
            let _ = storage_writer.commit(tx).await;
            tokio::task::yield_now().await;
        }
        done_writer.store(true, Ordering::Release);
    }));

    for h in handles {
        h.await.unwrap();
    }

    assert_eq!(
        storage.last_tx_id().await.unwrap().inner(),
        100,
        "Final transaction ID must equal 100"
    );
    assert!(
        observed_max.load(Ordering::Acquire) > 0,
        "Concurrent readers must have observed committed transaction IDs"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hmac_concurrent_key_creation_and_cleanup_test() -> Result<()> {
    let tmp = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let db_path = tmp.path().join("wal_test.log");

    let barrier = Arc::new(Barrier::new(5));
    let mut handles = Vec::new();

    // 5 concurrent tasks accessing load_or_create_integrity_key on the same uninitialized path
    for _ in 0..5 {
        let path = db_path.clone();
        let barrier_clone = Arc::clone(&barrier);

        handles.push(tokio::spawn(async move {
            barrier_clone.wait().await;
            contextra_store::wal::Wal::load_or_create_integrity_key(&path).await
        }));
    }

    let mut keys = Vec::new();
    for h in handles {
        let key_res = h.await.unwrap();
        assert!(key_res.is_ok(), "Concurrent key creation must succeed");
        keys.push(key_res.unwrap());
    }

    // Every task must observe the exact same 32-byte key
    let first_key = keys[0];
    for k in &keys[1..] {
        assert_eq!(
            *k, first_key,
            "Concurrent integrity key access must produce exactly one key"
        );
    }

    Ok(())
}
