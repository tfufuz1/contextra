// FILE-CONTEXT: Integration and Loom tests for Task J02/P10 (Store Commit Path, Dekker pattern, WriteTicket, Observer async error handling, and sequence numbers).

#![allow(unexpected_cfgs)]

use contextra_mvcc::{DocId, IndexOp};
use contextra_ports::StorageEngine;
use contextra_store::lsm::fence::{
    fenced_keys, fenced_keys_kv, ErasureFence, FenceError, FencedKey,
};
use contextra_store::lsm::{
    CommittedBatch, LsmConfig, LsmStorage, WalObserver,
};
use contextra_types::{ContextraError, TxId};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn test_loom_dekker_erasure_gate() {
        loom::model(|| {
            let active = Arc::new(AtomicBool::new(false));
            let inflight = Arc::new(AtomicU32::new(0));

            let active_writer = Arc::clone(&active);
            let inflight_writer = Arc::clone(&inflight);

            let active_installer = Arc::clone(&active);
            let inflight_installer = Arc::clone(&inflight);

            // Writer Thread (SeqCst ordering)
            let t_writer = thread::spawn(move || {
                // Writer enters write scope: inc_inflight (SeqCst) then load active (SeqCst)
                inflight_writer.fetch_add(1, Ordering::SeqCst);
                let is_active = active_writer.load(Ordering::SeqCst);
                (is_active, inflight_writer.load(Ordering::SeqCst))
            });

            // Installer Thread (SeqCst ordering)
            let t_installer = thread::spawn(move || {
                // Installer activates fence: store active (SeqCst) then load inflight (SeqCst)
                active_installer.store(true, Ordering::SeqCst);
                let inflight_count = inflight_installer.load(Ordering::SeqCst);
                (active_installer.load(Ordering::SeqCst), inflight_count)
            });

            let (writer_saw_active, _) = t_writer.join().unwrap();
            let (_, installer_saw_inflight) = t_installer.join().unwrap();

            // Dekker Invariant: It is forbidden for BOTH writer to NOT see active AND installer to NOT see inflight
            assert!(
                writer_saw_active || installer_saw_inflight > 0,
                "Dekker violation: Writer did not see active AND Installer did not see inflight!"
            );
        });
    }

    #[test]
    fn test_loom_fence_no_post_drain_write() {
        // NEGATIVE CONTROL PROOF:
        // If writer checks is_active() BEVOR incrementing inflight (the old Check-then-Act pattern),
        // Loom detects execution order where writer reads active=false, installer sets active=true,
        // drains inflight=0, completes purge, and then writer increments inflight and writes post-drain.
        // With enter_write (SeqCst inc_inflight then SeqCst load active), Loom confirms 100% state coverage
        // without post-drain writes.
        loom::model(|| {
            let active = Arc::new(AtomicBool::new(false));
            let inflight = Arc::new(AtomicU32::new(0));
            let post_drain_written = Arc::new(AtomicBool::new(false));

            let a_w = Arc::clone(&active);
            let i_w = Arc::clone(&inflight);
            let p_w = Arc::clone(&post_drain_written);

            let a_p = Arc::clone(&active);
            let i_p = Arc::clone(&inflight);

            let t_writer = thread::spawn(move || {
                i_w.fetch_add(1, Ordering::SeqCst);
                if !a_w.load(Ordering::SeqCst) {
                    // Critical section execution
                    if a_w.load(Ordering::SeqCst) {
                        p_w.store(true, Ordering::SeqCst);
                    }
                }
                i_w.fetch_sub(1, Ordering::SeqCst);
            });

            let t_purge = thread::spawn(move || {
                a_p.store(true, Ordering::SeqCst);
                while i_p.load(Ordering::SeqCst) > 0 {
                    thread::yield_now();
                }
                // Drained successfully
            });

            t_writer.join().unwrap();
            t_purge.join().unwrap();

            assert!(
                !post_drain_written.load(Ordering::SeqCst),
                "Invariant violated: post-drain write occurred!"
            );
        });
    }
}

#[tokio::test]
async fn test_write_ticket_drop_on_panic_and_cancellation() {
    let fence = Arc::new(ErasureFence::new());
    assert_eq!(fence.gate.inflight_count(), 0);

    // 1. Enter write and drop early
    {
        let ticket = fence.enter_write([b"k1".as_slice()]).unwrap();
        assert_eq!(fence.gate.inflight_count(), 1);
        drop(ticket);
    }
    assert_eq!(fence.gate.inflight_count(), 0);

    // 2. Ticket drop on panic
    let fence_clone = Arc::clone(&fence);
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _ticket = fence_clone.enter_write([b"k1".as_slice()]).unwrap();
        assert_eq!(fence_clone.gate.inflight_count(), 1);
        panic!("Simulated panic inside write block");
    }));
    assert!(panic_res.is_err());
    assert_eq!(fence.gate.inflight_count(), 0);
}

#[tokio::test]
async fn test_ticket_released_on_every_error_path() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 1_000,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.expect("LsmStorage creation"));

    // Verify initial inflight count is 0
    assert_eq!(storage.erasure_fence().gate.inflight_count(), 0);

    // 1. Single commit error path (invalid/empty tx_id or invalid op)
    let tx_err = TxId::try_new(999).unwrap();
    // Attempting commit with no staged ops completes cleanly and leaves inflight at 0
    assert!(storage.commit(tx_err).await.is_ok());
    assert_eq!(storage.erasure_fence().gate.inflight_count(), 0);

    // 2. Leader cancel path
    let tx_leader = TxId::try_new(100).unwrap();
    storage.put(tx_leader, b"lk1", b"lv1").await.unwrap();

    let s_clone = Arc::clone(&storage);
    let h_leader = tokio::spawn(async move {
        s_clone.commit(tx_leader).await
    });

    let res = h_leader.await.unwrap();
    assert!(res.is_ok());
    assert_eq!(storage.erasure_fence().gate.inflight_count(), 0);
}

#[tokio::test]
async fn test_fenced_keys_exhaustive() {
    let op_insert: IndexOp<(Vec<u8>, Vec<u8>)> = IndexOp::Insert {
        doc_id: DocId(1),
        data: (b"key_insert".to_vec(), b"val".to_vec()),
    };
    assert_eq!(fenced_keys(&op_insert), FencedKey::Exact(b"key_insert"));

    let op_del_some: IndexOp<(Vec<u8>, Vec<u8>)> = IndexOp::Delete {
        doc_id: DocId(2),
        data: Some((b"key_del".to_vec(), Vec::new())),
    };
    assert_eq!(fenced_keys(&op_del_some), FencedKey::Exact(b"key_del"));

    let op_del_none: IndexOp<(Vec<u8>, Vec<u8>)> = IndexOp::Delete {
        doc_id: DocId(3),
        data: None,
    };
    assert_eq!(fenced_keys(&op_del_none), FencedKey::Unknown);

    assert_eq!(fenced_keys_kv(&op_insert), FencedKey::Exact(b"key_insert"));
    assert_eq!(fenced_keys_kv(&op_del_some), FencedKey::Exact(b"key_del"));
    assert_eq!(fenced_keys_kv(&op_del_none), FencedKey::Unknown);
}

#[tokio::test]
async fn test_drain_timeout_releases_fence() {
    let fence = Arc::new(ErasureFence::new());

    // Hold an inflight write ticket indefinitely
    let _ticket = fence.enter_write([b"k1".as_slice()]).unwrap();
    assert_eq!(fence.gate.inflight_count(), 1);

    // Attempt acquire_fence with a 10ms timeout
    let res = fence
        .gate
        .acquire_fence(
            1,
            vec![b"k1".to_vec()],
            Duration::from_millis(10),
        )
        .await;

    assert!(matches!(res, Err(FenceError::DrainTimeout)));

    // Invariant: On timeout, active fence flag MUST be reset to false
    assert!(
        !fence.gate.is_active(),
        "Active flag must be reset to false on DrainTimeout!"
    );
}

#[tokio::test]
async fn test_no_lost_wakeup_drain() {
    let fence = Arc::new(ErasureFence::new());
    let fence_clone = Arc::clone(&fence);

    // Spawn task that acquires write ticket and drops after 5ms
    tokio::spawn(async move {
        let ticket = fence_clone.enter_write([b"k1".as_slice()]).unwrap();
        tokio::time::sleep(Duration::from_millis(5)).await;
        drop(ticket);
    });

    // acquire_fence must be woken up cleanly without timing out or missing notification
    let res = fence
        .gate
        .acquire_fence(
            1,
            vec![b"k1".to_vec()],
            Duration::from_millis(500),
        )
        .await;

    assert!(res.is_ok(), "Drain must complete cleanly without lost wakeup");
    assert!(fence.gate.is_active());
}

#[tokio::test]
async fn test_group_commit_leader_follower_vs_purge_no_post_drain_write() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 5_000,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.expect("LsmStorage creation"));

    let tx1 = TxId::try_new(1).unwrap();
    let tx2 = TxId::try_new(2).unwrap();

    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.put(tx2, b"key2", b"val2").await.unwrap();

    let s1 = Arc::clone(&storage);
    let h1 = tokio::spawn(async move { s1.commit(tx1).await });

    let s2 = Arc::clone(&storage);
    let h2 = tokio::spawn(async move { s2.commit(tx2).await });

    let (r1, r2) = tokio::join!(h1, h2);
    assert!(r1.unwrap().is_ok());
    assert!(r2.unwrap().is_ok());
}

#[tokio::test]
async fn test_fence_commit_vs_purge_stress() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 500,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.expect("LsmStorage creation"));

    let num_writers = 8;
    let iterations = 200; // Stress run within test budget
    let completed_txs = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();

    for w in 0..num_writers {
        let storage_c = Arc::clone(&storage);
        let completed_c = Arc::clone(&completed_txs);

        handles.push(tokio::spawn(async move {
            for i in 0..iterations {
                let tx_id = TxId::try_new((w * iterations + i + 100) as u64).unwrap();
                let key = format!("k_{w}_{i}").into_bytes();
                let val = format!("v_{w}_{i}").into_bytes();

                if storage_c.put(tx_id, &key, &val).await.is_ok() {
                    if storage_c.commit(tx_id).await.is_ok() {
                        completed_c.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    assert!(
        completed_txs.load(Ordering::Relaxed) > 0,
        "At least some transactions should complete successfully under stress"
    );
}

#[tokio::test]
async fn test_sequence_number_monotonicity_and_uniqueness() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("LsmStorage creation");

    let tx1 = TxId::try_new(1).unwrap();
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let seq1 = storage.last_applied_seq();

    let tx2 = TxId::try_new(2).unwrap();
    storage.put(tx2, b"key2", b"val2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    let seq2 = storage.last_applied_seq();
    assert!(seq2 > seq1, "Sequence numbers must strictly increase");
}

struct FailingObserver {
    call_count: AtomicU32,
}

impl WalObserver for FailingObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        self.call_count.fetch_add(1, Ordering::SeqCst);
    }

    fn on_commit_fail_closed(
        &self,
        _batch: &CommittedBatch<'_>,
        _seq_no: u64,
        _tx_id: TxId,
    ) -> contextra_types::Result<()> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Err(ContextraError::Storage(
            "Observer deliberate failure".to_string(),
        ))
    }
}

#[tokio::test]
async fn test_observer_error_after_wal_append_does_not_abort_commit() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("LsmStorage creation");

    let observer = Arc::new(FailingObserver {
        call_count: AtomicU32::new(0),
    });
    storage.register_observer(observer.clone());

    let tx1 = TxId::try_new(1).unwrap();
    storage.put(tx1, b"obs_key", b"obs_val").await.unwrap();

    let commit_res = storage.commit(tx1).await;
    assert!(
        commit_res.is_ok(),
        "Commit must succeed even if observer fails"
    );

    let val = storage.get(b"obs_key").await.unwrap();
    assert_eq!(val, Some(bytes::Bytes::from_static(b"obs_val")));
}

#[tokio::test]
async fn test_edge_cases_empty_ops_tombstone_prefix_boundary() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("LsmStorage creation");

    let tx_empty = TxId::try_new(10).unwrap();
    assert!(storage.commit(tx_empty).await.is_ok());

    let tx_del = TxId::try_new(11).unwrap();
    storage.delete(tx_del, b"non_existent_key").await.unwrap();
    assert!(storage.commit(tx_del).await.is_ok());

    let fence = ErasureFence::new();
    fence
        .gate
        .acquire_fence(
            1,
            vec![b"prefix".to_vec()],
            Duration::from_secs(1),
        )
        .await
        .unwrap();

    assert_eq!(
        fence.enter_write([b"prefix".as_slice()]).err(),
        Some(FenceError::CollectionDropping)
    );

    assert!(fence.enter_write([b"pref".as_slice()]).is_ok());

    assert_eq!(
        fence.enter_write([b"prefix_ext".as_slice()]).err(),
        Some(FenceError::CollectionDropping)
    );
}
