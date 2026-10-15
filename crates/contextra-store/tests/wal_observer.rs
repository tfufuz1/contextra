#![cfg(not(loom))]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{CommittedBatch, LsmConfig, LsmStorage, WalObserver, WriteOrigin};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

struct TestObserver {
    id: usize,
    invocations: Arc<parking_lot::Mutex<Vec<(usize, u64, TxId, WriteOrigin, usize)>>>,
    delay: Option<Duration>,
}

impl TestObserver {
    fn new(
        id: usize,
        invocations: Arc<parking_lot::Mutex<Vec<(usize, u64, TxId, WriteOrigin, usize)>>>,
    ) -> Self {
        Self {
            id,
            invocations,
            delay: None,
        }
    }

    fn with_delay(
        id: usize,
        invocations: Arc<parking_lot::Mutex<Vec<(usize, u64, TxId, WriteOrigin, usize)>>>,
        delay: Duration,
    ) -> Self {
        Self {
            id,
            invocations,
            delay: Some(delay),
        }
    }
}

impl WalObserver for TestObserver {
    fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId) {
        if let Some(delay) = self.delay {
            std::thread::sleep(delay);
        }
        self.invocations
            .lock()
            .push((self.id, seq_no, tx_id, batch.origin, batch.entries.len()));
    }
}

#[tokio::test]
async fn test_observer_single_commit_and_properties() {
    let dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let invocations = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let obs = Arc::new(TestObserver::new(1, Arc::clone(&invocations)));

    storage.register_observer(obs);

    let tx = TxId::new(10);
    storage.put(tx, b"k1", b"v1").await.expect("put");
    storage.commit(tx).await.expect("commit");

    let invs = invocations.lock().clone();
    assert_eq!(invs.len(), 1);
    let (obs_id, seq_no, committed_tx, origin, entry_count) = invs[0];
    assert_eq!(obs_id, 1);
    assert_eq!(committed_tx, tx);
    assert_eq!(origin, WriteOrigin::UserWrite);
    assert!(seq_no > 0);
    assert_eq!(entry_count, 2); // Put + TxEnd
}

#[tokio::test]
async fn test_multiple_observers_order() {
    let dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let invocations = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let obs1 = Arc::new(TestObserver::new(1, Arc::clone(&invocations)));
    let obs2 = Arc::new(TestObserver::new(2, Arc::clone(&invocations)));
    let obs3 = Arc::new(TestObserver::new(3, Arc::clone(&invocations)));

    storage.register_observer(obs1);
    storage.register_observer(obs2);
    storage.register_observer(obs3);

    let tx = TxId::new(20);
    storage.put(tx, b"k2", b"v2").await.expect("put");
    storage.commit(tx).await.expect("commit");

    let invs = invocations.lock().clone();
    assert_eq!(invs.len(), 3);
    assert_eq!(invs[0].0, 1);
    assert_eq!(invs[1].0, 2);
    assert_eq!(invs[2].0, 3);
}

#[tokio::test]
async fn test_observer_fail_open_timeout_deregistration() {
    let dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");
    storage.set_max_observer_latency(Duration::from_millis(1));

    let invocations = Arc::new(parking_lot::Mutex::new(Vec::new()));
    // Delay 10ms > max_observer_latency 1ms
    let slow_obs = Arc::new(TestObserver::with_delay(
        99,
        Arc::clone(&invocations),
        Duration::from_millis(10),
    ));

    storage.register_observer(slow_obs);

    let tx1 = TxId::new(30);
    storage.put(tx1, b"k3", b"v3").await.expect("put 1");
    storage
        .commit(tx1)
        .await
        .expect("commit 1 should succeed (fail-open)");

    // Wait for slow observer background execution to finish
    tokio::time::sleep(Duration::from_millis(20)).await;

    // The slow observer ran once
    assert_eq!(invocations.lock().len(), 1);

    // But now it should be deregistered!
    let tx2 = TxId::new(31);
    let start2 = Instant::now();
    storage.put(tx2, b"k4", b"v4").await.expect("put 2");
    storage.commit(tx2).await.expect("commit 2");
    let commit2_duration = start2.elapsed();

    // Invocations should still be 1 because slow observer was evicted on tx1
    assert_eq!(invocations.lock().len(), 1);
    assert!(commit2_duration < Duration::from_millis(50));
}

#[tokio::test]
async fn test_observer_group_commit_path() {
    let dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 5_000,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.expect("create storage"));

    let invocations = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let obs = Arc::new(TestObserver::new(1, Arc::clone(&invocations)));

    storage.register_observer(obs);

    let storage_clone = Arc::clone(&storage);
    let h1 = tokio::spawn(async move {
        let tx = TxId::new(40);
        storage_clone.put(tx, b"k_gc_1", b"v1").await.unwrap();
        storage_clone.commit(tx).await.unwrap();
    });

    let storage_clone2 = Arc::clone(&storage);
    let h2 = tokio::spawn(async move {
        let tx = TxId::new(41);
        storage_clone2.put(tx, b"k_gc_2", b"v2").await.unwrap();
        storage_clone2.commit(tx).await.unwrap();
    });

    h1.await.unwrap();
    h2.await.unwrap();

    let invs = invocations.lock().clone();
    assert_eq!(invs.len(), 2);
    let txs: Vec<TxId> = invs.iter().map(|i| i.2).collect();
    assert!(txs.contains(&TxId::new(40)));
    assert!(txs.contains(&TxId::new(41)));
}

#[tokio::test]
async fn test_zero_observers_regression() {
    let dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let tx = TxId::new(50);
    storage.put(tx, b"k_reg", b"v_reg").await.expect("put");
    storage.commit(tx).await.expect("commit");

    let val = storage.get(b"k_reg").await.expect("get");
    assert_eq!(val, Some(bytes::Bytes::from_static(b"v_reg")));
}
