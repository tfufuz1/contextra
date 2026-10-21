// FILE-CONTEXT: Test verifying phantom commit prevention (S1, INV-COM-1).

use contextra_ports::StorageEngine;
use contextra_store::lsm::observer::{CommittedBatch, WalObserver};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, Result, TxId};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

struct FailingObserver {
    should_fail: AtomicBool,
}

impl WalObserver for FailingObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {}

    fn on_commit_fail_closed(
        &self,
        _batch: &CommittedBatch<'_>,
        _seq_no: u64,
        _tx_id: TxId,
    ) -> Result<()> {
        if self.should_fail.load(Ordering::Relaxed) {
            Err(ContextraError::Storage(
                "Observer intentional error for test".into(),
            ))
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn observer_error_after_wal_append_returns_ok_and_value_visible() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await?;

    let observer = Arc::new(FailingObserver {
        should_fail: AtomicBool::new(true),
    });
    storage.register_observer(observer);

    let tx = TxId(100);
    let key = b"key-observer-test";
    let val = b"val-observer-test";

    storage.put(tx, key, val).await?;

    // Commit should succeed despite observer error after WAL append
    let commit_res = storage.commit(tx).await;
    assert!(
        commit_res.is_ok(),
        "Commit must return Ok even if observer fails after WAL append"
    );

    // Value must be visible immediately
    let read_val = storage.get(key).await?;
    assert_eq!(read_val.as_deref(), Some(val.as_slice()));

    // Verify recovery after re-opening storage
    drop(storage);
    let reopen_config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let reopened_storage = LsmStorage::new(reopen_config).await?;
    let recovered_val = reopened_storage.get(key).await?;
    assert_eq!(recovered_val.as_deref(), Some(val.as_slice()));

    Ok(())
}
