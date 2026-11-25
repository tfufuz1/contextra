#![allow(unexpected_cfgs)]
#![cfg(not(loom))]

use contextra_core::{StorageEngine, TxId};
use contextra_store::{CommittedBatch, LsmConfig, LsmStorage, WalObserver, WriteOrigin};
use contextra_testkit::ManualClock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

struct RecordingObserver {
    _id: usize,
    records: Arc<parking_lot::Mutex<Vec<(u64, TxId, WriteOrigin, usize)>>>,
    delay: Option<Duration>,
    panic_on_call: bool,
    clock_advance: Option<(Arc<ManualClock>, Duration)>,
}

impl RecordingObserver {
    fn new(
        id: usize,
        records: Arc<parking_lot::Mutex<Vec<(u64, TxId, WriteOrigin, usize)>>>,
    ) -> Self {
        Self {
            _id: id,
            records,
            delay: None,
            panic_on_call: false,
            clock_advance: None,
        }
    }

    fn with_delay(
        id: usize,
        records: Arc<parking_lot::Mutex<Vec<(u64, TxId, WriteOrigin, usize)>>>,
        delay: Duration,
    ) -> Self {
        Self {
            _id: id,
            records,
            delay: Some(delay),
            panic_on_call: false,
            clock_advance: None,
        }
    }

    fn panicking(
        id: usize,
        records: Arc<parking_lot::Mutex<Vec<(u64, TxId, WriteOrigin, usize)>>>,
    ) -> Self {
        Self {
            _id: id,
            records,
            delay: None,
            panic_on_call: true,
            clock_advance: None,
        }
    }

    fn with_clock_advance(
        id: usize,
        records: Arc<parking_lot::Mutex<Vec<(u64, TxId, WriteOrigin, usize)>>>,
        clock: Arc<ManualClock>,
        advance: Duration,
    ) -> Self {
        Self {
            _id: id,
            records,
            delay: None,
            panic_on_call: false,
            clock_advance: Some((clock, advance)),
        }
    }
}

impl WalObserver for RecordingObserver {
    fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId) {
        if self.panic_on_call {
            panic!("Simulated WalObserver panic inside on_commit");
        }
        if let Some(delay) = self.delay {
            std::thread::sleep(delay);
        }
        if let Some((ref clock, advance)) = self.clock_advance {
            clock.advance(advance);
        }
        self.records
            .lock()
            .push((seq_no, tx_id, batch.origin, batch.entries.len()));
    }
}

async fn create_test_storage(dir: &TempDir, group_commit_window_micros: u64) -> Arc<LsmStorage> {
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros,
        memtable_size_limit: 4 * 1024 * 1024,
        ..Default::default()
    };
    Arc::new(LsmStorage::new(config).await.expect("LsmStorage::new"))
}

#[tokio::test]
async fn test_observer_basic_notification() {
    let dir = TempDir::new().unwrap();
    let storage = create_test_storage(&dir, 0).await;

    let records = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let obs = Arc::new(RecordingObserver::new(1, Arc::clone(&records)));
    storage.register_observer(obs);

    let tx = TxId::new(1001);
    storage.put(tx, b"test_key", b"test_val").await.unwrap();
    storage.commit(tx).await.expect("commit failed");

    let recs = records.lock().clone();
    assert_eq!(recs.len(), 1);
    let (seq_no, committed_tx, origin, entry_count) = recs[0];
    assert!(seq_no > 0);
    assert_eq!(committed_tx, tx);
    assert_eq!(origin, WriteOrigin::UserWrite);
    assert_eq!(entry_count, 2); // Put + TxEnd

    storage.close().await.unwrap();
}

#[tokio::test]
async fn test_observer_ordering_single_and_group_commit() {
    // 1. Single commit path
    let dir1 = TempDir::new().unwrap();
    let storage1 = create_test_storage(&dir1, 0).await;

    let records1 = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let obs1 = Arc::new(RecordingObserver::new(1, Arc::clone(&records1)));
    storage1.register_observer(obs1);

    for i in 1..=5 {
        let tx = TxId::new(i);
        let key = format!("k_{i}");
        let val = format!("v_{i}");
        storage1
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
        storage1.commit(tx).await.unwrap();
    }

    let recs1 = records1.lock().clone();
    assert_eq!(recs1.len(), 5);
    for idx in 0..5 {
        let expected_tx = TxId::new((idx + 1) as u64);
        assert_eq!(recs1[idx].1, expected_tx);
        if idx > 0 {
            assert!(
                recs1[idx].0 > recs1[idx - 1].0,
                "seq numbers must strictly increase"
            );
        }
    }
    storage1.close().await.unwrap();

    // 2. Group commit path
    let dir2 = TempDir::new().unwrap();
    let storage2 = create_test_storage(&dir2, 2_000).await;

    let records2 = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let obs2 = Arc::new(RecordingObserver::new(2, Arc::clone(&records2)));
    storage2.register_observer(obs2);

    let mut handles = Vec::new();
    for i in 100..110 {
        let st = Arc::clone(&storage2);
        handles.push(tokio::spawn(async move {
            let tx = TxId::new(i);
            let key = format!("gc_k_{i}");
            let val = format!("gc_v_{i}");
            st.put(tx, key.as_bytes(), val.as_bytes()).await.unwrap();
            st.commit(tx).await.unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let recs2 = records2.lock().clone();
    assert_eq!(recs2.len(), 10);
    let notified_txs: Vec<u64> = recs2.iter().map(|r| r.1.inner()).collect();
    for i in 100..110 {
        assert!(
            notified_txs.contains(&i),
            "Missing notification for TxId({i})"
        );
    }

    storage2.close().await.unwrap();
}

#[tokio::test]
async fn test_observer_fail_open_timeout_and_latency_unaffected() {
    let dir = TempDir::new().unwrap();
    let storage = create_test_storage(&dir, 0).await;
    storage.set_max_observer_latency(Duration::from_millis(1));

    let records_slow = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let records_fast = Arc::new(parking_lot::Mutex::new(Vec::new()));

    // Slow observer delays 10ms (> 1ms max latency)
    let slow_obs = Arc::new(RecordingObserver::with_delay(
        100,
        Arc::clone(&records_slow),
        Duration::from_millis(10),
    ));
    let fast_obs = Arc::new(RecordingObserver::new(200, Arc::clone(&records_fast)));

    storage.register_observer(slow_obs);
    storage.register_observer(fast_obs);

    // First commit: slow observer runs once, exceeds threshold, gets evicted
    let tx1 = TxId::new(2001);
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    assert_eq!(records_slow.lock().len(), 1);
    assert_eq!(records_fast.lock().len(), 1);

    // Second commit: slow observer was evicted; commit latency is completely unhindered
    let start2 = Instant::now();
    let tx2 = TxId::new(2002);
    storage.put(tx2, b"key2", b"val2").await.unwrap();
    storage.commit(tx2).await.unwrap();
    let commit2_duration = start2.elapsed();

    // Slow observer count remains 1 (deregistered), fast observer count is 2
    assert_eq!(records_slow.lock().len(), 1);
    assert_eq!(records_fast.lock().len(), 2);

    // Commit latency after eviction must be fast (< 50ms tolerance)
    assert!(
        commit2_duration < Duration::from_millis(50),
        "Commit latency must remain unhindered after slow observer eviction: took {:?}",
        commit2_duration
    );

    storage.close().await.unwrap();
}

#[tokio::test]
async fn test_observer_panic_isolation() {
    let dir = TempDir::new().unwrap();
    let storage = create_test_storage(&dir, 0).await;

    let records_panic = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let records_healthy = Arc::new(parking_lot::Mutex::new(Vec::new()));

    let panicking_obs = Arc::new(RecordingObserver::panicking(1, Arc::clone(&records_panic)));
    let healthy_obs = Arc::new(RecordingObserver::new(2, Arc::clone(&records_healthy)));

    storage.register_observer(panicking_obs);
    storage.register_observer(healthy_obs);

    let tx1 = TxId::new(3001);
    storage.put(tx1, b"p_key1", b"p_val1").await.unwrap();
    // Commit MUST NOT panic or fail when an observer panics
    let commit_res = storage.commit(tx1).await;
    assert!(
        commit_res.is_ok(),
        "Commit must succeed despite panicking observer"
    );

    assert_eq!(records_healthy.lock().len(), 1);
    assert_eq!(records_panic.lock().len(), 0);

    // On second commit, panicking observer was deregistered
    let tx2 = TxId::new(3002);
    storage.put(tx2, b"p_key2", b"p_val2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    assert_eq!(records_healthy.lock().len(), 2);
    assert_eq!(records_panic.lock().len(), 0);

    storage.close().await.unwrap();
}

#[tokio::test]
async fn test_observer_determinism_with_manual_clock() {
    let dir = TempDir::new().unwrap();
    let storage = create_test_storage(&dir, 0).await;

    let manual_clock = Arc::new(ManualClock::new(1_000_000_000));
    storage.set_clock(manual_clock.clone());
    storage.set_max_observer_latency(Duration::from_millis(1));

    let records = Arc::new(parking_lot::Mutex::new(Vec::new()));
    // Observer advances the manual clock by 2ms inside on_commit (> 1ms max latency)
    let obs = Arc::new(RecordingObserver::with_clock_advance(
        1,
        Arc::clone(&records),
        manual_clock.clone(),
        Duration::from_millis(2),
    ));

    storage.register_observer(obs);

    let tx1 = TxId::new(4001);
    storage.put(tx1, b"m_key1", b"m_val1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    assert_eq!(records.lock().len(), 1);

    // Next commit: observer should be deregistered based on ManualClock calculation
    let tx2 = TxId::new(4002);
    storage.put(tx2, b"m_key2", b"m_val2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    assert_eq!(records.lock().len(), 1);

    storage.close().await.unwrap();
}
