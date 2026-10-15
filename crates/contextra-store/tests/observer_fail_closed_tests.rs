// ZWECK: Integrationstests für Fail-Closed WalObserver Verhalten.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_types::{ContextraError, Result, TxId};
use contextra_ports::StorageEngine;
use contextra_store::{CommittedBatch, LsmConfig, LsmStorage, WalObserver, WriteOrigin};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

/// Observer that returns an explicit error in `on_commit_fail_closed`.
struct FailingObserver;

impl WalObserver for FailingObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {}

    fn on_commit_fail_closed(
        &self,
        _batch: &CommittedBatch<'_>,
        _seq_no: u64,
        _tx_id: TxId,
    ) -> Result<()> {
        Err(ContextraError::Storage(
            "Observer intentionally failed".into(),
        ))
    }
}

/// Observer that panics inside callback.
struct PanickingObserver;

impl WalObserver for PanickingObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        panic!("Observer panic test");
    }
}

/// Observer that sleeps/blocks during callback.
struct SlowObserver {
    delay: Duration,
}

impl WalObserver for SlowObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        std::thread::sleep(self.delay);
    }
}

/// Observer that captures received `WriteOrigin`.
struct OriginCheckingObserver {
    last_origin: Arc<parking_lot::Mutex<Option<WriteOrigin>>>,
}

impl WalObserver for OriginCheckingObserver {
    fn on_commit(&self, batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        *self.last_origin.lock() = Some(batch.origin);
    }
}

async fn create_test_storage(dir: &TempDir) -> Arc<LsmStorage> {
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        memtable_size_limit: 4 * 1024 * 1024,
        ..Default::default()
    };
    Arc::new(LsmStorage::new(config).await.expect("LsmStorage::new"))
}

/// (a) Test: Observer returning `Err` in `on_commit_fail_closed` causes `storage.commit(tx)` to fail.
#[tokio::test]
async fn test_observer_error_fails_commit() {
    let dir = TempDir::new().expect("tempdir");
    let storage = create_test_storage(&dir).await;

    let failing_obs = Arc::new(FailingObserver);
    storage.register_observer(failing_obs);

    let tx = TxId::new(100);
    storage.put(tx, b"key1", b"val1").await.expect("put");

    let res = storage.commit(tx).await;
    assert!(
        res.is_err(),
        "Commit should fail when WalObserver returns an error"
    );

    match res {
        Err(ContextraError::Storage(msg)) => {
            assert!(
                msg.contains("Observer intentionally failed")
                    || msg.contains("WalObserver notification failed"),
                "Expected observer error message, got: {msg}"
            );
        }
        other => panic!("Expected ContextraError::Storage, got {:?}", other),
    }

    // Verify key was NOT committed to LSM storage visibility
    let val = storage.get(b"key1").await.expect("get");
    assert!(
        val.is_none(),
        "Uncommitted key should not be visible after failed commit"
    );

    storage.close().await.expect("close");
}

/// (b) Test: Panicking observer causes `storage.commit(tx)` to fail cleanly with `ContextraError::Storage`.
#[tokio::test]
async fn test_panicking_observer_fails_commit() {
    let dir = TempDir::new().expect("tempdir");
    let storage = create_test_storage(&dir).await;
    storage.set_max_observer_latency(Duration::from_millis(500));

    let panicking_obs = Arc::new(PanickingObserver);
    storage.register_observer(panicking_obs);

    let tx = TxId::new(200);
    storage.put(tx, b"panic_k", b"panic_v").await.expect("put");

    let res = storage.commit(tx).await;
    assert!(
        res.is_err(),
        "Commit should fail cleanly when WalObserver panics"
    );

    match res {
        Err(ContextraError::Storage(msg)) => {
            assert!(
                msg.contains("WalObserver") || msg.contains("panic"),
                "Expected panic or observer error message, got: {msg}"
            );
        }
        other => panic!("Expected ContextraError::Storage, got {:?}", other),
    }

    // Verify key was NOT committed to LSM storage visibility
    let val = storage.get(b"panic_k").await.expect("get");
    assert!(
        val.is_none(),
        "Key should not be visible when observer panic fails commit"
    );

    storage.close().await.expect("close");
}

/// (c) Test: Observer timeout causes `storage.commit(tx)` to fail with `ContextraError::Storage`.
#[tokio::test]
async fn test_slow_observer_timeout_fails_commit() {
    let dir = TempDir::new().expect("tempdir");
    let storage = create_test_storage(&dir).await;

    storage.set_max_observer_latency(Duration::from_millis(10));

    let slow_obs = Arc::new(SlowObserver {
        delay: Duration::from_millis(50),
    });
    storage.register_observer(slow_obs);

    let tx = TxId::new(300);
    storage.put(tx, b"slow_k", b"slow_v").await.expect("put");

    let res = storage.commit(tx).await;
    assert!(
        res.is_err(),
        "Commit should fail when WalObserver times out"
    );

    match res {
        Err(ContextraError::Storage(msg)) => {
            assert!(
                msg.contains("timed out")
                    || msg.contains("exceeded")
                    || msg.contains("WalObserver"),
                "Expected timeout message, got: {msg}"
            );
        }
        other => panic!("Expected ContextraError::Storage, got {:?}", other),
    }

    storage.close().await.expect("close");
}

/// (d) Test: `WriteOrigin` is correctly passed in `CommittedBatch.origin`.
#[tokio::test]
async fn test_write_origin_passed_to_observer() {
    let dir = TempDir::new().expect("tempdir");
    let storage = create_test_storage(&dir).await;

    let last_origin = Arc::new(parking_lot::Mutex::new(None));
    let obs = Arc::new(OriginCheckingObserver {
        last_origin: last_origin.clone(),
    });
    storage.register_observer(obs);

    let tx = TxId::new(400);
    storage.put(tx, b"key2", b"val2").await.expect("put");
    storage.commit(tx).await.expect("commit");

    let origin = last_origin.lock().clone();
    assert_eq!(origin, Some(WriteOrigin::UserWrite));

    storage.close().await.expect("close");
}
