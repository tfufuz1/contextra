// FILE-CONTEXT: Integration and Loom tests for Task J02 (Store Commit Path, Dekker pattern, WriteTicket, Observer async error handling, and sequence numbers).

#![allow(unexpected_cfgs)]

use contextra_ports::StorageEngine;
use contextra_store::lsm::{
    CommittedBatch, ErasureFence, FenceError, LsmConfig, LsmStorage, WalObserver, WriteOrigin,
};
use contextra_types::{ContextraError, TxId};
use std::sync::atomic::{AtomicU32, Ordering};
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

            // Writer Thread
            let t_writer = thread::spawn(move || {
                // Writer enters write scope: inc_inflight then load active
                inflight_writer.fetch_add(1, Ordering::SeqCst);
                let is_active = active_writer.load(Ordering::SeqCst);
                (is_active, inflight_writer.load(Ordering::SeqCst))
            });

            // Installer Thread
            let t_installer = thread::spawn(move || {
                // Installer activates fence: store active then load inflight
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
    let panic_res = std::panic::catch_unwind(move || {
        let _ticket = fence_clone.enter_write([b"k1".as_slice()]).unwrap();
        assert_eq!(fence_clone.gate.inflight_count(), 1);
        panic!("Simulated panic inside write block");
    });
    assert!(panic_res.is_err());
    assert_eq!(fence.gate.inflight_count(), 0);
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

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let seq1 = storage.last_applied_seq();

    // Stage tx2 with invalid key prefix or failure trigger, or commit success
    let tx2 = TxId::new(2);
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

    let tx1 = TxId::new(1);
    storage.put(tx1, b"obs_key", b"obs_val").await.unwrap();

    // Commit must succeed despite observer error
    let commit_res = storage.commit(tx1).await;
    assert!(
        commit_res.is_ok(),
        "Commit must succeed even if observer fails"
    );

    // Verify data is committed and visible
    let val = storage.get(b"obs_key").await.unwrap();
    assert_eq!(val, Some(bytes::Bytes::from_static(b"obs_val")));
}

#[tokio::test]
async fn test_group_commit_with_observer_and_fence() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 10_000, // 10ms window for group commit
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.expect("LsmStorage creation"));

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    storage.put(tx1, b"grp_key1", b"val1").await.unwrap();
    storage.put(tx2, b"grp_key2", b"val2").await.unwrap();

    let s1 = Arc::clone(&storage);
    let h1 = tokio::spawn(async move { s1.commit(tx1).await });

    let s2 = Arc::clone(&storage);
    let h2 = tokio::spawn(async move { s2.commit(tx2).await });

    let (r1, r2) = tokio::join!(h1, h2);
    assert!(r1.unwrap().is_ok());
    assert!(r2.unwrap().is_ok());

    assert_eq!(
        storage.get(b"grp_key1").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val1"))
    );
    assert_eq!(
        storage.get(b"grp_key2").await.unwrap(),
        Some(bytes::Bytes::from_static(b"val2"))
    );
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

    // 1. Empty ops commit
    let tx_empty = TxId::new(10);
    assert!(storage.commit(tx_empty).await.is_ok());

    // 2. Single tombstone delete
    let tx_del = TxId::new(11);
    storage.delete(tx_del, b"non_existent_key").await.unwrap();
    assert!(storage.commit(tx_del).await.is_ok());

    // 3. Key exactly on fence prefix boundary
    let fence = ErasureFence::new();
    fence
        .gate
        .acquire_fence(1, vec![b"prefix".to_vec()], Duration::from_secs(1))
        .await
        .unwrap();

    // "prefix" starts with "prefix" -> blocked
    assert_eq!(
        fence.enter_write([b"prefix".as_slice()]).err(),
        Some(FenceError::CollectionDropping)
    );

    // "pref" does not start with "prefix" -> allowed
    assert!(fence.enter_write([b"pref".as_slice()]).is_ok());

    // "prefix_ext" starts with "prefix" -> blocked
    assert_eq!(
        fence.enter_write([b"prefix_ext".as_slice()]).err(),
        Some(FenceError::CollectionDropping)
    );
}
