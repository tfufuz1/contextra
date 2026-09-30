// ZWECK: Integrationstests für WalObserver Fail-Open Deadline, Circuit Breaker, Drop Counter und Non-blocking Shutdown.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::{CommittedBatch, LsmConfig, LsmStorage, ObserverRegistry, WalObserver};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Test observer that sleeps for a configurable duration inside `on_commit`.
struct SleepingObserver {
    sleep_duration: Duration,
    call_count: AtomicUsize,
}

impl SleepingObserver {
    fn new(sleep_duration: Duration) -> Self {
        Self {
            sleep_duration,
            call_count: AtomicUsize::new(0),
        }
    }
}

impl WalObserver for SleepingObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(self.sleep_duration);
    }
}

/// Test observer that records received transaction IDs and sequence numbers.
struct RecordingObserver {
    received: Mutex<Vec<(u64, TxId)>>,
}

impl RecordingObserver {
    fn new() -> Self {
        Self {
            received: Mutex::new(Vec::new()),
        }
    }
}

impl WalObserver for RecordingObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId) {
        if let Ok(mut guard) = self.received.lock() {
            guard.push((seq_no, tx_id));
        }
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

/// (1) Test: Observer sleeping for 5s does not measurably delay 100 commits.
#[tokio::test]
async fn test_sleeping_observer_does_not_delay_commits() {
    let dir = TempDir::new().expect("tempdir");
    let storage = create_test_storage(&dir).await;

    let sleeping_obs = Arc::new(SleepingObserver::new(Duration::from_secs(5)));
    storage.register_observer(sleeping_obs.clone());

    let start = Instant::now();

    for i in 1..=100u64 {
        let tx = TxId::new(i);
        let key = format!("k_{}", i).into_bytes();
        let val = format!("v_{}", i).into_bytes();
        storage.put(tx, &key, &val).await.expect("put");
        storage.commit(tx).await.expect("commit");
    }

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(2000),
        "100 commits took {:?}, expected < 2000ms due to Fail-Open deadline protection",
        elapsed
    );

    storage.close().await.expect("close");
}

/// (2) Test: Drop counters and Circuit Breaker status are queryable.
#[test]
fn test_drop_counter_and_circuit_breaker_status_queryable() {
    let registry = ObserverRegistry::new();
    registry.set_max_observer_latency(Duration::from_millis(1));

    let sleeping_obs: Arc<dyn WalObserver> =
        Arc::new(SleepingObserver::new(Duration::from_millis(50)));
    registry.register_observer(sleeping_obs.clone());

    assert_eq!(registry.dropped_count(), 0);
    assert_eq!(registry.dropped_count_for(&sleeping_obs), 0);
    assert!(!registry.is_circuit_breaker_open(&sleeping_obs));
    assert!(!registry.is_any_circuit_breaker_open());

    registry.notify(
        &[],
        1,
        TxId::new(10),
        contextra_store::WriteOrigin::UserWrite,
    );

    assert!(
        registry.dropped_count() >= 1,
        "Dropped count should be at least 1 after timeout, got {}",
        registry.dropped_count()
    );
    assert!(
        registry.is_circuit_breaker_open(&sleeping_obs),
        "Circuit breaker should be open after timeout"
    );
    assert!(
        registry.is_any_circuit_breaker_open(),
        "is_any_circuit_breaker_open should return true"
    );

    registry.notify(
        &[],
        2,
        TxId::new(11),
        contextra_store::WriteOrigin::UserWrite,
    );

    assert!(
        registry.dropped_count() >= 2,
        "Dropped count should be at least 2 after fast-skip, got {}",
        registry.dropped_count()
    );

    registry.clear_circuit_breaker(&sleeping_obs);
    assert!(
        !registry.is_circuit_breaker_open(&sleeping_obs),
        "Circuit breaker should be closed after clear_circuit_breaker"
    );
}

/// (3) Test: Normal healthy observer receives all events in strict sequence order.
#[tokio::test]
async fn test_normal_observer_receives_all_events_in_order() {
    let dir = TempDir::new().expect("tempdir");
    let storage = create_test_storage(&dir).await;
    storage.set_max_observer_latency(Duration::from_millis(50));

    let rec_obs = Arc::new(RecordingObserver::new());
    storage.register_observer(rec_obs.clone());

    for i in 1..=50u64 {
        let tx = TxId::new(i * 10);
        let key = format!("seq_k_{}", i).into_bytes();
        let val = format!("seq_v_{}", i).into_bytes();
        storage.put(tx, &key, &val).await.expect("put");
        storage.commit(tx).await.expect("commit");
    }

    let recorded = rec_obs.received.lock().unwrap().clone();
    assert_eq!(recorded.len(), 50, "Observer should receive all 50 events");

    for (idx, (seq_no, tx_id)) in recorded.into_iter().enumerate() {
        let expected_i = (idx + 1) as u64;
        assert_eq!(tx_id, TxId::new(expected_i * 10));
        assert!(seq_no > 0, "Sequence number should be positive");
    }

    storage.close().await.expect("close");
}

/// (4) Test: Shutdown/drop of ObserverRegistry or LsmStorage does not block indefinitely.
#[tokio::test]
async fn test_shutdown_drop_does_not_block() {
    let dir = TempDir::new().expect("tempdir");
    let storage = create_test_storage(&dir).await;

    let slow_obs = Arc::new(SleepingObserver::new(Duration::from_secs(10)));
    storage.register_observer(slow_obs.clone());

    let tx = TxId::new(1001);
    storage.put(tx, b"k_slow", b"v_slow").await.expect("put");
    storage.commit(tx).await.expect("commit");

    let start_drop = Instant::now();
    storage.close().await.expect("close");
    let drop_elapsed = start_drop.elapsed();

    assert!(
        drop_elapsed < Duration::from_millis(500),
        "Storage close/drop took {:?}, expected < 500ms without blocking on sleeping observer",
        drop_elapsed
    );
}
