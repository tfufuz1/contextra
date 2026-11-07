#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::indexing_slicing,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::await_holding_lock
)]
//! # Integrationstest: Systematische Crash- und Recovery-Simulation
//!
//! Simuliert plötzliche Abstürze vor, während und nach dem Schreiben von Checkpoints.

use contextra_checkpoint::PersistentCheckpointStore;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_testkit::InMemoryStorageEngine;
use contextra_types::{ContextraError, Result, TxId};
use parking_lot::Mutex;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Wrapping StorageEngine mit kontrollierter Fehlerinjektion für Crash- und Abbruchtests.
struct FaultInjectingStorage<S: StorageEngine> {
    inner: Arc<S>,
    fail_put: AtomicBool,
    fail_commit: AtomicBool,
    fail_delete: AtomicBool,
    fail_pin: AtomicBool,
    rolled_back: Mutex<Vec<TxId>>,
}

impl<S: StorageEngine> FaultInjectingStorage<S> {
    fn new(inner: Arc<S>) -> Self {
        Self {
            inner,
            fail_put: AtomicBool::new(false),
            fail_commit: AtomicBool::new(false),
            fail_delete: AtomicBool::new(false),
            fail_pin: AtomicBool::new(false),
            rolled_back: Mutex::new(Vec::new()),
        }
    }
}

impl<S: StorageEngine> StorageEngine for FaultInjectingStorage<S> {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.inner.get(key)
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.inner.get_at_seq(key, seq)
    }

    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if self.fail_put.load(Ordering::SeqCst) {
                return Err(ContextraError::Io(std::io::Error::other(
                    "Injected Put Failure",
                )));
            }
            self.inner.put(tx_id, key, value).await
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if self.fail_delete.load(Ordering::SeqCst) {
                return Err(ContextraError::Io(std::io::Error::other(
                    "Injected Delete Failure",
                )));
            }
            self.inner.delete(tx_id, key).await
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if self.fail_commit.load(Ordering::SeqCst) {
                return Err(ContextraError::Io(std::io::Error::other(
                    "Injected Commit Failure",
                )));
            }
            self.inner.commit(tx_id).await
        })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.rolled_back.lock().push(tx_id);
            self.inner.rollback(tx_id).await
        })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.rollback_to_tx(tx_id)
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        self.inner.last_seq_no()
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        self.inner.last_tx_id()
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        self.inner.flush()
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        self.inner.stats()
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if self.fail_pin.load(Ordering::SeqCst) {
                return Err(ContextraError::Io(std::io::Error::other(
                    "Injected Pin Failure",
                )));
            }
            self.inner.pin_checkpoint(seq_no).await
        })
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.inner.unpin_checkpoint(seq_no)
    }

    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan(start, end, limit)
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan_prefix(prefix)
    }
}

// Testet: systematic crash at every checkpoint io point via fault-injected storage.
#[tokio::test]
async fn systematic_crash_at_every_checkpoint_io_point() {
    for crash_mode in 0..4 {
        let mem = Arc::new(InMemoryStorageEngine::new());
        let fault_storage = Arc::new(FaultInjectingStorage::new(mem.clone()));

        // Put initial test data in storage
        let tx1 = TxId::new(1);
        fault_storage.put(tx1, b"doc1", b"content_v1").await.unwrap();
        fault_storage.commit(tx1).await.unwrap();

        let store = PersistentCheckpointStore::open(fault_storage.clone(), "test_ns")
            .await
            .unwrap();

        match crash_mode {
            0 => fault_storage.fail_pin.store(true, Ordering::SeqCst),
            1 => fault_storage.fail_put.store(true, Ordering::SeqCst),
            2 => fault_storage.fail_commit.store(true, Ordering::SeqCst),
            _ => {} // Clean run
        }

        let tx_cp = TxId::new(TxId::INTERNAL_BASE + 1000);
        let cp_res = store
            .create_checkpoint(
                "chk_systematic",
                "col_main",
                1,
                tx_cp,
                json!({"version": 1}),
            )
            .await;

        if crash_mode < 3 {
            assert!(cp_res.is_err(), "Crash mode {crash_mode} must cause create_checkpoint to fail");
            assert!(
                store.get_checkpoint("chk_systematic").await.unwrap().is_none(),
                "Failed checkpoint creation must not be queryable"
            );
        } else {
            assert!(cp_res.is_ok(), "Clean run must succeed");
            let meta = store.get_checkpoint("chk_systematic").await.unwrap().expect("Checkpoint must exist");
            assert_eq!(meta.name, "chk_systematic");
            assert_eq!(meta.seq_no, 1);
        }
    }
}

// Testet: crash during checkpoint deletion leaves clean state.
#[tokio::test]
async fn crash_during_checkpoint_deletion_leaves_clean_state() {
    let mem = Arc::new(InMemoryStorageEngine::new());
    let fault_storage = Arc::new(FaultInjectingStorage::new(mem.clone()));
    let store = PersistentCheckpointStore::open(fault_storage.clone(), "test_drop_ns")
        .await
        .unwrap();

    let tx_cp = TxId::new(TxId::INTERNAL_BASE + 2000);
    store
        .create_checkpoint("chk_to_drop", "col_drop", 10, tx_cp, json!({}))
        .await
        .unwrap();

    // Verify created
    assert!(store.get_checkpoint("chk_to_drop").await.unwrap().is_some());

    // Inject delete failure on drop
    fault_storage.fail_delete.store(true, Ordering::SeqCst);

    let drop_res = store.drop_checkpoint("chk_to_drop").await;
    assert!(drop_res.is_err(), "Drop with failing storage delete must return error");

    // Clear fault and verify drop works cleanly
    fault_storage.fail_delete.store(false, Ordering::SeqCst);
    let retry_drop = store.drop_checkpoint("chk_to_drop").await;
    assert!(retry_drop.is_ok(), "Retry of drop_checkpoint must succeed cleanly");

    let check = store.get_checkpoint("chk_to_drop").await.unwrap();
    assert!(check.is_none(), "Checkpoint must be removed after successful drop");
}
