#![allow(unexpected_cfgs)]
#![cfg(not(loom))]

use contextra_core::{StorageEngine, TxId};
use contextra_store::{CommittedBatch, LsmConfig, LsmStorage, WalObserver};
use contextra_testkit::ManualClock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Controlled blocking observer using synchronization primitives.
///
/// Holds `unblock_signal` until released by the test,
/// proving that `commit()` blocks until `on_commit()` returns, demonstrating no preemptive timeout.
struct ControlledBlockingObserver {
    unblock_signal: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}

impl WalObserver for ControlledBlockingObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        // Spin/wait until the test sets unblock_signal to true
        while !self.unblock_signal.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

/// Panicking observer to verify panic isolation and eviction.
struct PanickingObserver {
    calls: Arc<AtomicUsize>,
}

impl WalObserver for PanickingObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        panic!("Intentional test panic in WalObserver::on_commit");
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

/// §3.2 Test (i) & (ii):
/// (i) Proves `commit()` blocks until `on_commit()` returns (no preemptive cancellation/timeout thread).
/// (ii) Proves that once `on_commit()` returns after exceeding `max_observer_latency`, the observer is deregistered.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_blocking_observer_no_preemptive_timeout_and_eviction_on_return() {
    let dir = TempDir::new().unwrap();
    let storage = create_test_storage(&dir).await;

    // Inject ManualClock so latency measurement is completely deterministic and fast in test time
    let clock = Arc::new(ManualClock::new(1_000_000_000));
    storage.set_clock(clock.clone());
    storage.set_max_observer_latency(Duration::from_millis(1));

    let unblock_signal = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicUsize::new(0));

    let observer = Arc::new(ControlledBlockingObserver {
        unblock_signal: Arc::clone(&unblock_signal),
        calls: Arc::clone(&calls),
    });

    storage.register_observer(observer);

    let storage_clone = Arc::clone(&storage);
    let tx1 = TxId::new(1001);
    storage_clone.put(tx1, b"key1", b"val1").await.unwrap();

    // Spawn commit on a separate thread via std::thread or spawn_blocking
    let unblock_signal_clone = Arc::clone(&unblock_signal);
    let clock_clone = Arc::clone(&clock);

    let (tx_done, rx_done) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            storage_clone.commit(tx1).await.expect("commit tx1");
        });
        let _ = tx_done.send(());
    });

    // Wait until on_commit is actively being executed
    let start_wait = Instant::now();
    while calls.load(Ordering::SeqCst) == 0 {
        tokio::time::sleep(Duration::from_millis(1)).await;
        if start_wait.elapsed() > Duration::from_secs(5) {
            panic!("Observer on_commit was not called within 5 seconds");
        }
    }

    // Give it 50ms while unblock_signal is false. Verify thread has NOT finished (proving no preemptive timeout)
    tokio::time::sleep(Duration::from_millis(50)).await;
    let mut rx_done = rx_done;
    assert!(
        rx_done.try_recv().is_err(),
        "commit() returned early! Preemptive timeout guarantee does not exist."
    );

    // Advance clock by 5ms (> 1ms max latency) while observer is blocked
    clock_clone.advance(Duration::from_millis(5));

    // Signal observer to unblock and return
    unblock_signal_clone.store(true, Ordering::SeqCst);

    // Now commit MUST finish cleanly
    tokio::time::timeout(Duration::from_secs(2), rx_done)
        .await
        .expect("commit task timed out after unblocking")
        .expect("commit task channel error");

    assert_eq!(calls.load(Ordering::SeqCst), 1);

    // On second commit, observer should have been evicted due to elapsed latency > max_observer_latency
    let tx2 = TxId::new(1002);
    storage.put(tx2, b"key2", b"val2").await.unwrap();
    storage.commit(tx2).await.expect("commit tx2");

    // Call count remains 1, proving eviction
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "Observer should have been evicted after exceeding max_observer_latency"
    );

    storage.close().await.unwrap();
}

/// §3.2 Panicking observer control test:
/// Verifies panicking observer is caught via catch_unwind and deregistered without failing commit.
#[tokio::test]
async fn test_panicking_observer_deregistered_without_failing_commit() {
    let dir = TempDir::new().unwrap();
    let storage = create_test_storage(&dir).await;

    let calls = Arc::new(AtomicUsize::new(0));
    let panicking_observer = Arc::new(PanickingObserver {
        calls: Arc::clone(&calls),
    });

    storage.register_observer(panicking_observer);

    let tx1 = TxId::new(2001);
    storage.put(tx1, b"key1", b"val1").await.unwrap();

    // Commit must succeed despite observer panic
    let res = storage.commit(tx1).await;
    assert!(
        res.is_ok(),
        "Commit failed despite Fail-Open panic isolation"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    // Second commit: panicking observer should have been deregistered
    let tx2 = TxId::new(2002);
    storage.put(tx2, b"key2", b"val2").await.unwrap();
    storage.commit(tx2).await.expect("commit tx2");

    // Call count remains 1
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "Panicking observer should have been deregistered after panic"
    );

    storage.close().await.unwrap();
}
