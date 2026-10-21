// FILE-CONTEXT: Test verifying cancel safety of commit execution (S4, INV-COM-3).

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::{Result, TxId};
use std::future::Future;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn commit_future_dropped_after_append_is_consistent() -> Result<()> {
    let temp_dir = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await?;

    let tx = TxId(202);
    let key = b"key-cancel-safety";
    let val = b"val-cancel-safety";

    storage.put(tx, key, val).await?;

    // Poll the commit future past WAL append task spawn, then cancel/drop the outer caller future
    let commit_fut = storage.commit(tx);
    let mut pinned_commit = Box::pin(commit_fut);

    // Poll once to enter commit_internal, drain ops, prepare batch, and spawn append task
    let waker = futures::task::noop_waker();
    let mut cx = std::task::Context::from_waker(&waker);
    let _ = Future::poll(pinned_commit.as_mut(), &mut cx);

    // Drop caller future mid-commit (simulating client cancellation / timeout after append spawn)
    drop(pinned_commit);

    // Give background commit task time to complete Append -> Apply -> Visibility
    tokio::time::sleep(Duration::from_millis(50)).await;

    // State must be consistent and value readable
    let read_val = storage.get(key).await?;
    assert_eq!(read_val.as_deref(), Some(val.as_slice()));

    Ok(())
}
