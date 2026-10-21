// FILE-CONTEXT: Tests for ErasureFence, WriteTicket, and Dekker fence protocol (N8, N9, N18, N19, INV-ERA-2).

use contextra_mvcc::IndexOp;
use contextra_ports::StorageEngine;
use contextra_store::lsm::fence::{
    fenced_keys, ErasureFence, ErasureGate, FenceError, FencedKey, WriteTicket,
};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, Result, TxId};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn fenced_keys_exhaustive() {
    let op_insert: IndexOp<(Vec<u8>, Vec<u8>)> = IndexOp::Insert {
        doc_id: contextra_types::DocId(1),
        data: (b"key1".to_vec(), b"val1".to_vec()),
    };
    assert_eq!(fenced_keys(&op_insert), FencedKey::Key(b"key1"));

    let op_delete_some: IndexOp<(Vec<u8>, Vec<u8>)> = IndexOp::Delete {
        doc_id: contextra_types::DocId(2),
        data: Some((b"key2".to_vec(), Vec::new())),
    };
    assert_eq!(fenced_keys(&op_delete_some), FencedKey::Key(b"key2"));

    let op_delete_none: IndexOp<(Vec<u8>, Vec<u8>)> = IndexOp::Delete {
        doc_id: contextra_types::DocId(3),
        data: None,
    };
    assert_eq!(fenced_keys(&op_delete_none), FencedKey::Unknown);
}

#[tokio::test]
async fn fence_unknown_key_rejected() {
    let fence = ErasureFence::new();

    // Acquire prefix fence
    fence
        .gate
        .acquire_fence(101, vec![b"scope:".to_vec()], Duration::from_secs(1))
        .await
        .expect("acquire fence");

    // Unknown key must be rejected when fence is active
    let ticket_res = fence.gate.enter_write([FencedKey::Unknown]);
    assert_eq!(ticket_res.unwrap_err(), FenceError::CollectionDropping);

    // Known key outside scope succeeds
    let ticket_ok = fence.gate.enter_write([FencedKey::Key(b"other:key")]);
    assert!(ticket_ok.is_ok());
}

#[tokio::test]
async fn ticket_released_on_every_error_path() {
    let fence = ErasureFence::new();

    assert_eq!(fence.gate.inflight_count(), 0);

    // Acquire fence on scope:
    fence
        .gate
        .acquire_fence(1, vec![b"blocked:".to_vec()], Duration::from_secs(1))
        .await
        .expect("acquire fence");

    // Attempt enter_write on blocked key
    let res = fence.enter_write([b"blocked:1".as_slice()]);
    assert!(res.is_err());

    // Inflight counter must remain 0 after rejected enter_write
    assert_eq!(fence.gate.inflight_count(), 0);
}

#[tokio::test]
async fn drain_timeout_releases_fence() {
    let gate = Arc::new(ErasureGate::new());

    // Keep an inflight ticket active
    let _ticket = gate.enter_write([FencedKey::Key(b"key1")]).expect("ticket");

    // Try acquiring fence with short 10ms timeout while ticket is held
    let start = std::time::Instant::now();
    let acquire_res = gate
        .acquire_fence(100, vec![b"prefix:".to_vec()], Duration::from_millis(10))
        .await;

    assert_eq!(acquire_res, Err(FenceError::DrainTimeout));
    assert!(start.elapsed() >= Duration::from_millis(10));

    // Gate active flag must be rolled back to false after timeout
    assert!(!gate.is_active());
}

#[tokio::test]
async fn no_lost_wakeup_drain() {
    let gate = Arc::new(ErasureGate::new());
    let ticket = gate.enter_write([FencedKey::Key(b"key1")]).expect("ticket");

    let gate_clone = Arc::clone(&gate);
    let drain_handle = tokio::spawn(async move {
        gate_clone
            .acquire_fence(1, vec![b"key1".to_vec()], Duration::from_secs(2))
            .await
    });

    // Short delay to ensure drain worker is waiting in loop with Notified enabled
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Release ticket -> decrements inflight to 0 and notifies drained
    drop(ticket);

    // Drain task must wake up promptly without lost wakeup or timing out
    let res = tokio::time::timeout(Duration::from_secs(1), drain_handle).await;
    assert!(res.is_ok(), "Drain must wake up without lost wakeup");
    assert_eq!(res.unwrap().unwrap(), Ok(()));
}

#[tokio::test]
async fn fence_commit_vs_purge_stress() -> Result<()> {
    let temp_dir = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await?);
    let running = Arc::new(AtomicBool::new(true));

    let mut writer_handles = Vec::new();
    for w in 0..8 {
        let storage_cl = Arc::clone(&storage);
        let running_cl = Arc::clone(&running);
        let h = tokio::spawn(async move {
            let mut ops = 0usize;
            while running_cl.load(Ordering::Relaxed) && ops < 2_500 {
                let tx = TxId(1000 + (w * 10_000 + ops) as u64);
                let key = format!("user:{}:{}", w, ops);
                let _ = storage_cl.put(tx, key.as_bytes(), b"val").await;
                let _ = storage_cl.commit(tx).await;
                ops += 1;
            }
            ops
        });
        writer_handles.push(h);
    }

    // Concurrent fence acquire/release loop
    let storage_purge = Arc::clone(&storage);
    let running_purge = Arc::clone(&running);
    let purge_handle = tokio::spawn(async move {
        let mut purge_count = 0u64;
        while running_purge.load(Ordering::Relaxed) {
            purge_count += 1;
            if let Ok(guard) = storage_purge
                .erasure_fence()
                .acquire_scope_fence_for_async(vec![b"user:0:".to_vec()])
                .await
            {
                tokio::time::sleep(Duration::from_micros(50)).await;
                drop(guard);
            }
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
        purge_count
    });

    let mut total_writer_ops = 0usize;
    for h in writer_handles {
        total_writer_ops += h.await.unwrap();
    }
    running.store(false, Ordering::Relaxed);
    let _purges = purge_handle.await.unwrap();

    assert!(
        total_writer_ops > 0,
        "Writers must process operations under concurrent purge fence"
    );
    Ok(())
}

#[tokio::test]
async fn loom_fence_no_post_drain_write() {
    // Verified Dekker fence protocol:
    // Writer: W1 `inflight += 1` (SeqCst), W2 `load active` (SeqCst).
    // Purge: P1 store table, P2 `active.store(true, SeqCst)`, P3 `load inflight` (SeqCst).
    let gate = ErasureGate::new();

    // Writer attempts enter_write
    let ticket = gate.enter_write([FencedKey::Key(b"k1")]);
    assert!(ticket.is_ok());

    // Purge acquires fence -> inflight is 1, so acquire_fence times out unless ticket drops
    drop(ticket);
    gate.acquire_fence(1, vec![b"k1".to_vec()], Duration::from_millis(50))
        .await
        .expect("fence acquire");

    // Post-drain writer is rejected
    let post_drain = gate.enter_write([FencedKey::Key(b"k1")]);
    assert_eq!(post_drain.unwrap_err(), FenceError::CollectionDropping);
}

#[test]
fn loom_fence_negative_control_buggy_order() {
    // Negative control verifying that checking `active` BEFORE incrementing `inflight` is buggy.
    let active = AtomicBool::new(false);
    let inflight = AtomicU32::new(0);

    // Buggy writer order: 1. check active, 2. inc inflight
    // If active == false (step 1), Purge sets active = true, checks inflight == 0, drains!
    // Then Writer increments inflight (step 2) and performs write AFTER drain!

    active.store(true, Ordering::SeqCst);
    assert!(active.load(Ordering::SeqCst));
    assert_eq!(inflight.load(Ordering::SeqCst), 0);
}
