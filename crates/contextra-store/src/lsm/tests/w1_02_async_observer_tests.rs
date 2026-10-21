// FILE-CONTEXT: Test verifying commit latency independence from observer execution in current_thread runtime (S2, INV-ASYNC-1).

use contextra_ports::StorageEngine;
use contextra_store::lsm::observer::{CommittedBatch, WalObserver};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::{Result, TxId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

struct SlowObserver {
    commits_received: AtomicU64,
}

impl WalObserver for SlowObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        // Simulate heavy/slow processing in observer
        std::thread::sleep(Duration::from_millis(50));
        self.commits_received.fetch_add(1, Ordering::Relaxed);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn commit_latency_independent_of_observer_current_thread() -> Result<()> {
    let temp_dir = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await?;

    let slow_obs = Arc::new(SlowObserver {
        commits_received: AtomicU64::new(0),
    });
    storage.register_observer(slow_obs.clone());

    let tx = TxId(101);
    storage.put(tx, b"key-slow", b"val-slow").await?;

    let start = Instant::now();
    storage.commit(tx).await?;
    let commit_duration = start.elapsed();

    // Commit must complete fast without blocking for the 50ms slow observer
    assert!(
        commit_duration < Duration::from_millis(20),
        "Commit took {:?}, expected < 20ms despite 50ms slow observer",
        commit_duration
    );

    // Give background observer worker task a moment to finish processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(slow_obs.commits_received.load(Ordering::Relaxed), 1);

    Ok(())
}
