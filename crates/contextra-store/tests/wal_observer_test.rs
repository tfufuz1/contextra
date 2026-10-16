use contextra_core::{StorageEngine, TxId};
use contextra_store::{CommittedBatch, LsmConfig, LsmStorage, WalObserver, WriteOrigin};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct TestObserver {
    id: usize,
    commit_count: AtomicUsize,
    last_tx_id: Mutex<Option<TxId>>,
    last_max_seq: Mutex<Option<u64>>,
    last_origin: Mutex<Option<WriteOrigin>>,
    delay: Option<Duration>,
    call_order: Option<Arc<Mutex<Vec<usize>>>>,
}

impl TestObserver {
    fn new(id: usize) -> Self {
        Self {
            id,
            commit_count: AtomicUsize::new(0),
            last_tx_id: Mutex::new(None),
            last_max_seq: Mutex::new(None),
            last_origin: Mutex::new(None),
            delay: None,
            call_order: None,
        }
    }

    fn with_delay(id: usize, delay: Duration) -> Self {
        Self {
            id,
            commit_count: AtomicUsize::new(0),
            last_tx_id: Mutex::new(None),
            last_max_seq: Mutex::new(None),
            last_origin: Mutex::new(None),
            delay: Some(delay),
            call_order: None,
        }
    }

    fn with_order(id: usize, call_order: Arc<Mutex<Vec<usize>>>) -> Self {
        Self {
            id,
            commit_count: AtomicUsize::new(0),
            last_tx_id: Mutex::new(None),
            last_max_seq: Mutex::new(None),
            last_origin: Mutex::new(None),
            delay: None,
            call_order: Some(call_order),
        }
    }
}

impl WalObserver for TestObserver {
    fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId) {
        self.commit_count.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut guard) = self.last_tx_id.lock() {
            *guard = Some(tx_id);
        }
        if let Ok(mut guard) = self.last_max_seq.lock() {
            *guard = Some(seq_no);
        }
        if let Ok(mut guard) = self.last_origin.lock() {
            *guard = Some(batch.origin);
        }
        if let Some(ref order) = self.call_order {
            if let Ok(mut guard) = order.lock() {
                guard.push(self.id);
            }
        }
        if let Some(delay) = self.delay {
            std::thread::sleep(delay);
        }
    }
}

async fn create_test_storage(dir: &std::path::Path, group_commit_micros: u64) -> Arc<LsmStorage> {
    let config = LsmConfig {
        path: dir.to_path_buf(),
        group_commit_window_micros: group_commit_micros,
        memtable_size_limit: 4 * 1024 * 1024,
        ..Default::default()
    };
    Arc::new(LsmStorage::new(config).await.expect("LsmStorage::new"))
}

#[tokio::test]
async fn test_observer_single_commit_called_once() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let storage = create_test_storage(temp_dir.path(), 0).await;

    let observer = Arc::new(TestObserver::new(1));
    storage.register_observer(observer.clone());

    let tx = TxId::new(100);
    storage.put(tx, b"key1", b"val1").await.expect("put");
    storage.commit(tx).await.expect("commit");

    assert_eq!(observer.commit_count.load(Ordering::SeqCst), 1);
    assert_eq!(observer.last_tx_id.lock().unwrap().expect("last_tx_id"), tx);
    assert_eq!(
        observer.last_origin.lock().unwrap().expect("last_origin"),
        WriteOrigin::UserWrite
    );
    assert!(observer.last_max_seq.lock().unwrap().expect("max_seq") > 0);

    storage.close().await.expect("close");
}

#[tokio::test]
async fn test_multiple_observers_called_in_registration_order() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let storage = create_test_storage(temp_dir.path(), 0).await;

    let call_order = Arc::new(Mutex::new(Vec::new()));
    let obs1 = Arc::new(TestObserver::with_order(1, call_order.clone()));
    let obs2 = Arc::new(TestObserver::with_order(2, call_order.clone()));
    let obs3 = Arc::new(TestObserver::with_order(3, call_order.clone()));

    storage.register_observer(obs1.clone());
    storage.register_observer(obs2.clone());
    storage.register_observer(obs3.clone());

    let tx = TxId::new(101);
    storage.put(tx, b"k", b"v").await.expect("put");
    storage.commit(tx).await.expect("commit");

    let order = call_order.lock().unwrap().clone();
    assert_eq!(order, vec![1, 2, 3]);

    storage.close().await.expect("close");
}

#[tokio::test]
async fn test_slow_observer_deregistered_fail_open() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let storage = create_test_storage(temp_dir.path(), 0).await;

    // Slow observer blocks for 10ms (> 1ms latency limit)
    let slow_obs = Arc::new(TestObserver::with_delay(1, Duration::from_millis(10)));
    let fast_obs = Arc::new(TestObserver::new(2));

    storage.register_observer(slow_obs.clone());
    storage.register_observer(fast_obs.clone());

    let start = Instant::now();
    let tx1 = TxId::new(201);
    storage.put(tx1, b"k1", b"v1").await.expect("put");
    storage.commit(tx1).await.expect("commit");
    let commit_elapsed = start.elapsed();

    // Commit should succeed without failing (Fail-Open)
    assert_eq!(slow_obs.commit_count.load(Ordering::SeqCst), 1);
    assert_eq!(fast_obs.commit_count.load(Ordering::SeqCst), 1);

    // On the second commit, slow_obs should have been deregistered
    let tx2 = TxId::new(202);
    storage.put(tx2, b"k2", b"v2").await.expect("put");
    storage.commit(tx2).await.expect("commit");

    // slow_obs still count 1 (deregistered), fast_obs count 2
    assert_eq!(slow_obs.commit_count.load(Ordering::SeqCst), 1);
    assert_eq!(fast_obs.commit_count.load(Ordering::SeqCst), 2);

    // Ensure commit didn't panic or block excessively beyond the single slow call
    assert!(commit_elapsed < Duration::from_secs(1));

    storage.close().await.expect("close");
}

#[tokio::test]
async fn test_observer_single_and_group_commit_paths() {
    // 1. Single-commit path (group_commit_window_micros == 0)
    let temp_dir1 = tempfile::tempdir().expect("tempdir");
    let storage1 = create_test_storage(temp_dir1.path(), 0).await;

    let obs1 = Arc::new(TestObserver::new(1));
    storage1.register_observer(obs1.clone());

    let tx1 = TxId::new(301);
    storage1.put(tx1, b"k1", b"v1").await.expect("put");
    storage1.commit(tx1).await.expect("commit");

    assert_eq!(obs1.commit_count.load(Ordering::SeqCst), 1);
    assert_eq!(obs1.last_tx_id.lock().unwrap().expect("tx"), tx1);
    storage1.close().await.expect("close");

    // 2. Group-commit path (group_commit_window_micros > 0)
    let temp_dir2 = tempfile::tempdir().expect("tempdir");
    let storage2 = create_test_storage(temp_dir2.path(), 1000).await;

    let obs2 = Arc::new(TestObserver::new(2));
    storage2.register_observer(obs2.clone());

    let tx2 = TxId::new(302);
    storage2.put(tx2, b"k2", b"v2").await.expect("put");
    storage2.commit(tx2).await.expect("commit");

    assert_eq!(obs2.commit_count.load(Ordering::SeqCst), 1);
    assert_eq!(obs2.last_tx_id.lock().unwrap().expect("tx"), tx2);
    storage2.close().await.expect("close");
}

#[tokio::test]
async fn test_zero_observers_regression() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let storage = create_test_storage(temp_dir.path(), 0).await;

    let tx = TxId::new(401);
    storage.put(tx, b"key_reg", b"val_reg").await.expect("put");
    storage.commit(tx).await.expect("commit");

    let val = storage.get(b"key_reg").await.expect("get").expect("found");
    assert_eq!(&val[..], b"val_reg");

    storage.close().await.expect("close");
}
