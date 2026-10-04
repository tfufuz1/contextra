// FILE-CONTEXT: Campaign stress tests for contextra-store concurrency, cancel-safety, interleaving, and observers.
// TS: 2026-10-03

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use bytes::Bytes;
use contextra_core::{StorageEngine, TxId};
use contextra_store::{CommittedBatch, LsmConfig, LsmStorage, WalObserver};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

/// Ground truth oracle tracking committed state independently. (R4: Anti-Mirroring Oracle)
#[derive(Clone, Default)]
struct GroundTruthOracle {
    inner: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,
    tx_counter: Arc<AtomicU64>,
}

impl GroundTruthOracle {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BTreeMap::new())),
            tx_counter: Arc::new(AtomicU64::new(100)),
        }
    }

    fn next_tx(&self) -> TxId {
        TxId::new(self.tx_counter.fetch_add(1, Ordering::SeqCst))
    }

    fn put(&self, key: Vec<u8>, val: Vec<u8>) {
        let mut map = self.inner.lock().unwrap();
        map.insert(key, val);
    }

    fn delete(&self, key: &[u8]) {
        let mut map = self.inner.lock().unwrap();
        map.remove(key);
    }

    fn snapshot(&self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        let map = self.inner.lock().unwrap();
        map.clone()
    }
}

/// Slow observer that blocks for a specified duration inside `on_commit`.
struct BlockingObserver {
    block_duration: Duration,
    received_count: Arc<AtomicUsize>,
}

impl BlockingObserver {
    fn new(block_duration: Duration) -> (Self, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));
        (
            Self {
                block_duration,
                received_count: Arc::clone(&counter),
            },
            counter,
        )
    }
}

impl WalObserver for BlockingObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        self.received_count.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(self.block_duration);
    }
}

/// a) Group-Commit Stress Test under high concurrency & artificial delay
#[tokio::test]
async fn test_campaign_group_commit_stress_200_tasks() {
    let res = timeout(Duration::from_secs(30), async {
        let tmp = TempDir::new().expect("tempdir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            group_commit_window_micros: 200,
            memtable_size_limit: 64 * 1024,
            ..Default::default()
        };

        let storage = Arc::new(
            LsmStorage::new(config.clone())
                .await
                .expect("LsmStorage::new"),
        );
        let oracle = GroundTruthOracle::new();

        let num_tasks = 200;
        let batches_per_task = 10; // Total 2000 transactions
        let mut handles = Vec::new();

        for task_id in 0..num_tasks {
            let s_clone = Arc::clone(&storage);
            let o_clone = oracle.clone();
            let handle = tokio::spawn(async move {
                for b in 0..batches_per_task {
                    let tx = o_clone.next_tx();
                    let key = format!("k_{:04}_{:02}", task_id, b).into_bytes();
                    let val = format!("v_{:04}_{:02}", task_id, b).into_bytes();

                    s_clone.put(tx, &key, &val).await.expect("put");
                    s_clone.commit(tx).await.expect("commit");
                    o_clone.put(key, val);
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.await.expect("task join");
        }

        // Verify confirmed commits match ground truth oracle exactly
        let expected = oracle.snapshot();
        assert_eq!(expected.len(), num_tasks * batches_per_task);

        for (k, expected_v) in &expected {
            let actual = s_clone_get(&storage, k).await;
            assert_eq!(actual, Some(Bytes::from(expected_v.clone())));
        }

        // Verify WAL completeness / replay consistency post reopen
        drop(storage);

        let storage_reopened = LsmStorage::new(config).await.expect("reopen");
        for (k, expected_v) in &expected {
            let actual = s_clone_get(&storage_reopened, k).await;
            assert_eq!(actual, Some(Bytes::from(expected_v.clone())));
        }
    })
    .await;

    assert!(
        res.is_ok(),
        "Test timed out due to possible deadlock or starvation!"
    );
}

/// b) Commit / Rollback / Flush Interleaving against Ground Truth Oracle
#[tokio::test]
async fn test_campaign_commit_rollback_flush_interleaving_32_tasks() {
    let res = timeout(Duration::from_secs(30), async {
        let seed = 9988776655u64;
        let mut rng = StdRng::seed_from_u64(seed);

        let tmp = TempDir::new().expect("tempdir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 8 * 1024, // 8 KB limit to force flushes
            ..Default::default()
        };

        let storage = Arc::new(LsmStorage::new(config).await.expect("LsmStorage::new"));
        let oracle = GroundTruthOracle::new();

        let num_tasks = 32;
        let mut handles = Vec::new();

        for task_id in 0..num_tasks {
            let s_clone = Arc::clone(&storage);
            let o_clone = oracle.clone();
            let task_seed = rng.gen::<u64>();

            let handle = tokio::spawn(async move {
                let mut task_rng = StdRng::seed_from_u64(task_seed);
                for op_i in 0..15 {
                    let tx = o_clone.next_tx();
                    let key = format!("interleave_k_{}", task_id % 10).into_bytes();
                    let val = format!("val_{}_{}", task_id, op_i).into_bytes();

                    let op_type = task_rng.gen_range(0..5);
                    match op_type {
                        0 => {
                            // Put + Commit
                            if s_clone.put(tx, &key, &val).await.is_ok() {
                                if s_clone.commit(tx).await.is_ok() {
                                    o_clone.put(key, val);
                                }
                            }
                        }
                        1 => {
                            // Put + Rollback
                            if s_clone.put(tx, &key, &val).await.is_ok() {
                                let _ = s_clone.rollback(tx).await;
                            }
                        }
                        2 => {
                            // PutIfAbsent + Commit
                            if matches!(s_clone.put_if_absent(tx, &key, &val).await, Ok(true)) {
                                if s_clone.commit(tx).await.is_ok() {
                                    o_clone.put(key, val);
                                }
                            } else {
                                let _ = s_clone.rollback(tx).await;
                            }
                        }
                        3 => {
                            // Delete + Commit
                            if s_clone.delete(tx, &key).await.is_ok() {
                                if s_clone.commit(tx).await.is_ok() {
                                    o_clone.delete(&key);
                                }
                            }
                        }
                        _ => {
                            // Flush trigger
                            let _ = s_clone.force_flush().await;
                        }
                    }
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.await.expect("join");
        }

        // Quiescence check against oracle
        let expected = oracle.snapshot();
        for (k, expected_v) in &expected {
            let actual = s_clone_get(&storage, k).await;
            assert_eq!(actual, Some(Bytes::from(expected_v.clone())));
        }
    })
    .await;

    assert!(res.is_ok(), "Test timed out during interleaving stress!");
}

/// c) Cancel-Safety & Intent-Lock Cleanup Test
#[tokio::test]
async fn test_campaign_cancel_safety_intent_locks() {
    let res = timeout(Duration::from_secs(15), async {
        let tmp = TempDir::new().expect("tempdir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            ..Default::default()
        };

        let storage = Arc::new(LsmStorage::new(config).await.expect("LsmStorage::new"));

        // Phase 1: Abort task during put_if_absent
        for i in 0..50 {
            let tx = TxId::new(1000 + i);
            let key = format!("cancel_k_{}", i).into_bytes();
            let val = format!("cancel_v_{}", i).into_bytes();

            let s_clone = Arc::clone(&storage);
            let handle = tokio::spawn(async move {
                let _ = s_clone.put_if_absent(tx, &key, &val).await;
                tokio::time::sleep(Duration::from_millis(50)).await;
                let _ = s_clone.commit(tx).await;
            });

            // Immediately abort the task mid-flight
            tokio::task::yield_now().await;
            handle.abort();
            let _ = handle.await;
        }

        // Phase 2: Verify that keys can still be written to without hanging/being locked permanently
        for i in 0..50 {
            let tx = TxId::new(2000 + i);
            let key = format!("cancel_k_{}", i).into_bytes();
            let val = format!("fresh_v_{}", i).into_bytes();

            let res = storage.put_if_absent(tx, &key, &val).await;
            assert!(
                res.is_ok(),
                "Key cancel_k_{} remained locked after task abort!",
                i
            );
            storage.commit(tx).await.expect("commit fresh key");
        }
    })
    .await;

    assert!(res.is_ok(), "Cancel safety test timed out!");
}

/// d) Observer Stress & Fail-Open Protection Test
#[tokio::test]
async fn test_campaign_observer_blocking_fail_open_stress() {
    let res = timeout(Duration::from_secs(15), async {
        let tmp = TempDir::new().expect("tempdir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            ..Default::default()
        };

        let storage = Arc::new(LsmStorage::new(config).await.expect("LsmStorage::new"));
        storage.set_max_observer_latency(Duration::from_millis(10));

        // Register a blocking observer that takes 500ms per commit
        let (blocking_obs, received_counter) = BlockingObserver::new(Duration::from_millis(500));
        let obs_arc: Arc<dyn WalObserver> = Arc::new(blocking_obs);
        storage.register_observer(obs_arc);

        let start = std::time::Instant::now();
        let num_commits = 100;

        for i in 1..=num_commits {
            let tx = TxId::new(i);
            let key = format!("obs_k_{}", i).into_bytes();
            let val = format!("obs_v_{}", i).into_bytes();

            storage.put(tx, &key, &val).await.expect("put");
            storage.commit(tx).await.expect("commit");
        }

        let elapsed = start.elapsed();

        // 100 commits x 500ms would take 50 seconds without fail-open!
        assert!(
            elapsed < Duration::from_secs(2),
            "Commit throughput collapsed due to blocking observer! Elapsed: {:?}",
            elapsed
        );

        // Verify observer received count
        assert!(
            received_counter.load(Ordering::Relaxed) > 0,
            "Observer was never notified"
        );
    })
    .await;

    assert!(res.is_ok(), "Observer stress test timed out!");
}

async fn s_clone_get(storage: &LsmStorage, key: &[u8]) -> Option<Bytes> {
    storage.get(key).await.expect("get")
}
