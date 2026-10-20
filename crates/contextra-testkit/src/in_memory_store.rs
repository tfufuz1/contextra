// FILE-CONTEXT
// STAND: 2026-09-19T20:12:00Z (SESSION: 01c5be8b)
// ZWECK: In-Memory StorageEngine implementation for fast, lightweight testing.
// INVARIANTEN: Zero disk I/O, transactional staging & commit/rollback, zero unsafe.

use bytes::Bytes;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{Result, TxId};

#[derive(Debug, Clone)]
enum StagedOp {
    Put(Vec<u8>),
    Delete,
}

type StoreDataMap = BTreeMap<Vec<u8>, (Vec<u8>, u64)>;
type StagedTxMap = BTreeMap<TxId, BTreeMap<Vec<u8>, StagedOp>>;

/// An in-memory, thread-safe implementation of `StorageEngine` for tests.
#[derive(Debug, Clone)]
pub struct InMemoryStorageEngine {
    data: Arc<RwLock<StoreDataMap>>,
    staged: Arc<RwLock<StagedTxMap>>,
    seq: Arc<AtomicU64>,
    last_tx: Arc<RwLock<TxId>>,
}

impl Default for InMemoryStorageEngine {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(BTreeMap::new())),
            staged: Arc::new(RwLock::new(BTreeMap::new())),
            seq: Arc::new(AtomicU64::new(0)),
            last_tx: Arc::new(RwLock::new(TxId::INVALID)),
        }
    }
}

impl InMemoryStorageEngine {
    /// Creates a new, empty in-memory storage engine.
    pub fn new() -> Self {
        Self::default()
    }
}

impl StorageEngine for InMemoryStorageEngine {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let guard = self.data.read();
            Ok(guard.get(key).map(|(val, _)| Bytes::copy_from_slice(val)))
        })
    }

    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let guard = self.data.read();
            if let Some((val, val_seq)) = guard.get(key) {
                if *val_seq <= seq {
                    return Ok(Some(Bytes::copy_from_slice(val)));
                }
            }
            Ok(None)
        })
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staged = self.staged.write();
            staged
                .entry(tx_id)
                .or_default()
                .insert(key.to_vec(), StagedOp::Put(value.to_vec()));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staged = self.staged.write();
            staged
                .entry(tx_id)
                .or_default()
                .insert(key.to_vec(), StagedOp::Delete);
            Ok(())
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let ops = {
                let mut staged = self.staged.write();
                staged.remove(&tx_id)
            };

            if let Some(ops) = ops {
                let new_seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
                let mut data = self.data.write();
                for (key, op) in ops {
                    match op {
                        StagedOp::Put(val) => {
                            data.insert(key, (val, new_seq));
                        }
                        StagedOp::Delete => {
                            data.remove(&key);
                        }
                    }
                }
                *self.last_tx.write() = tx_id;
            }
            Ok(())
        })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staged = self.staged.write();
            staged.remove(&tx_id);
            Ok(())
        })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            // Simplified in-memory rollback: reset staging
            self.staged.write().clear();
            Ok(())
        })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            let data = self.data.read();
            let total_size: usize = data.iter().map(|(k, (v, _))| k.len() + v.len()).sum();
            Ok(StorageStats {
                num_segments: 1,
                total_size_bytes: total_size as u64,
                memtable_size_bytes: total_size as u64,
            })
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(self.seq.load(Ordering::SeqCst)) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(*self.last_tx.read()) })
    }

    fn pin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.read();
            let mut results = Vec::new();
            for (key, (val, _)) in data.range(prefix.to_vec()..) {
                if key.starts_with(prefix) {
                    results.push((key.clone(), val.clone()));
                } else {
                    break;
                }
            }
            Ok(results)
        })
    }

    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.read();
            let start_bound = match start {
                std::ops::Bound::Included(b) => std::ops::Bound::Included(b.to_vec()),
                std::ops::Bound::Excluded(b) => std::ops::Bound::Excluded(b.to_vec()),
                std::ops::Bound::Unbounded => std::ops::Bound::Unbounded,
            };
            let end_bound = match end {
                std::ops::Bound::Included(b) => std::ops::Bound::Included(b.to_vec()),
                std::ops::Bound::Excluded(b) => std::ops::Bound::Excluded(b.to_vec()),
                std::ops::Bound::Unbounded => std::ops::Bound::Unbounded,
            };

            let mut results = Vec::new();
            for (key, (val, _)) in data.range((start_bound, end_bound)) {
                results.push((key.clone(), val.clone()));
                if let Some(lim) = limit {
                    if results.len() >= lim {
                        break;
                    }
                }
            }
            Ok(results)
        })
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Barrier;

    #[tokio::test]
    async fn test_in_memory_store_put_get_commit() {
        let store = InMemoryStorageEngine::new();
        let tx = TxId::try_new(1).expect("valid tx");

        assert!(store.get(b"key1").await.unwrap().is_none());

        store.put(tx, b"key1", b"value1").await.unwrap();
        // Not visible before commit
        assert!(store.get(b"key1").await.unwrap().is_none());

        store.commit(tx).await.unwrap();
        let val = store.get(b"key1").await.unwrap();
        assert_eq!(val, Some(Bytes::from_static(b"value1")));
        assert_eq!(store.last_tx_id().await.unwrap(), tx);
        assert_eq!(store.last_seq_no().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_store_rollback() {
        let store = InMemoryStorageEngine::new();
        let tx = TxId::try_new(2).expect("valid tx");

        store.put(tx, b"key_rb", b"val").await.unwrap();
        store.rollback(tx).await.unwrap();
        store.commit(tx).await.unwrap();

        assert!(store.get(b"key_rb").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_in_memory_store_edge_empty_and_large_keys_values() {
        let store = InMemoryStorageEngine::new();
        let tx = TxId::try_new(10).expect("valid tx");

        // Empty key and empty value
        store.put(tx, b"", b"").await.unwrap();
        store.commit(tx).await.unwrap();

        let empty_val = store.get(b"").await.unwrap();
        assert_eq!(empty_val, Some(Bytes::from_static(b"")));

        // Large key (100KB) and large value (500KB)
        let large_key = vec![0xAB; 100 * 1024];
        let large_val = vec![0xCD; 500 * 1024];
        let tx_large = TxId::try_new(11).expect("valid tx");

        store.put(tx_large, &large_key, &large_val).await.unwrap();
        store.commit(tx_large).await.unwrap();

        let fetched_large = store.get(&large_key).await.unwrap().unwrap();
        assert_eq!(fetched_large.len(), large_val.len());
        assert_eq!(&fetched_large[..], &large_val[..]);

        let stats = store.stats().await.unwrap();
        assert_eq!(stats.total_size_bytes, 600 * 1024);
    }

    #[tokio::test]
    async fn test_in_memory_store_edge_scans_and_prefixes() {
        let store = InMemoryStorageEngine::new();

        // Scan empty store
        let empty_scan = store
            .scan(std::ops::Bound::Unbounded, std::ops::Bound::Unbounded, None)
            .await
            .unwrap();
        assert!(empty_scan.is_empty());

        let empty_prefix = store.scan_prefix(b"pref").await.unwrap();
        assert!(empty_prefix.is_empty());

        // Insert items
        let tx = TxId::try_new(20).expect("valid tx");
        store.put(tx, b"a/1", b"val_a1").await.unwrap();
        store.put(tx, b"a/2", b"val_a2").await.unwrap();
        store.put(tx, b"b/1", b"val_b1").await.unwrap();
        store.commit(tx).await.unwrap();

        // Prefix scan
        let a_prefix = store.scan_prefix(b"a/").await.unwrap();
        assert_eq!(a_prefix.len(), 2);
        assert_eq!(a_prefix[0], (b"a/1".to_vec(), b"val_a1".to_vec()));
        assert_eq!(a_prefix[1], (b"a/2".to_vec(), b"val_a2".to_vec()));

        // Range scan with limit
        let limited = store
            .scan(
                std::ops::Bound::Included(b"a/1".as_slice()),
                std::ops::Bound::Unbounded,
                Some(2),
            )
            .await
            .unwrap();
        assert_eq!(limited.len(), 2);

        // Excluded bound scan
        let excluded = store
            .scan(
                std::ops::Bound::Excluded(b"a/1".as_slice()),
                std::ops::Bound::Included(b"b/1".as_slice()),
                None,
            )
            .await
            .unwrap();
        assert_eq!(excluded.len(), 2);
        assert_eq!(excluded[0].0, b"a/2".to_vec());
        assert_eq!(excluded[1].0, b"b/1".to_vec());
    }

    #[tokio::test]
    async fn test_in_memory_store_edge_duplicate_delete_and_empty_commit() {
        let store = InMemoryStorageEngine::new();
        let tx1 = TxId::try_new(30).expect("valid tx");

        // Commit with 0 ops (no staged writes/deletes for tx1)
        store.commit(tx1).await.unwrap();
        assert_eq!(store.last_tx_id().await.unwrap(), TxId::INVALID);
        assert_eq!(store.last_seq_no().await.unwrap(), 0);

        // Stage put then double delete in same transaction
        let tx2 = TxId::try_new(31).expect("valid tx");
        store.put(tx2, b"key_dup", b"val").await.unwrap();
        store.delete(tx2, b"key_dup").await.unwrap();
        store.commit(tx2).await.unwrap();

        assert!(store.get(b"key_dup").await.unwrap().is_none());

        // Delete key that does not exist
        let tx3 = TxId::try_new(32).expect("valid tx");
        store.delete(tx3, b"non_existent").await.unwrap();
        store.commit(tx3).await.unwrap();

        assert!(store.get(b"non_existent").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_in_memory_store_get_at_seq() {
        let store = InMemoryStorageEngine::new();

        let tx1 = TxId::try_new(40).expect("valid tx");
        store.put(tx1, b"key_seq1", b"val_seq1").await.unwrap();
        store.commit(tx1).await.unwrap(); // seq 1

        // For key_seq1 (committed at seq 1), sequence lower than 1 returns None
        assert!(store.get_at_seq(b"key_seq1", 0).await.unwrap().is_none());
        // Sequence >= 1 returns val_seq1
        assert_eq!(
            store.get_at_seq(b"key_seq1", 1).await.unwrap(),
            Some(Bytes::from_static(b"val_seq1"))
        );
        assert_eq!(
            store.get_at_seq(b"key_seq1", 10).await.unwrap(),
            Some(Bytes::from_static(b"val_seq1"))
        );

        let tx2 = TxId::try_new(41).expect("valid tx");
        store.put(tx2, b"key_seq2", b"val_seq2").await.unwrap();
        store.commit(tx2).await.unwrap(); // seq 2

        assert!(store.get_at_seq(b"key_seq2", 1).await.unwrap().is_none());
        assert_eq!(
            store.get_at_seq(b"key_seq2", 2).await.unwrap(),
            Some(Bytes::from_static(b"val_seq2"))
        );
    }

    #[tokio::test]
    async fn test_in_memory_store_rollback_to_tx_and_checkpoints() {
        let store = InMemoryStorageEngine::new();
        let tx = TxId::try_new(50).expect("valid tx");

        store.put(tx, b"k_rb_to", b"v_rb_to").await.unwrap();
        store
            .rollback_to_tx(TxId::try_new(10).unwrap())
            .await
            .unwrap();

        // Staging map should be cleared by rollback_to_tx
        store.commit(tx).await.unwrap();
        assert!(store.get(b"k_rb_to").await.unwrap().is_none());

        // Checkpoints & flush
        store.pin_checkpoint(1).await.unwrap();
        store.unpin_checkpoint(1).await.unwrap();
        store.flush().await.unwrap();
    }

    #[tokio::test]
    async fn test_in_memory_store_concurrency_parallel_tx_barrier() {
        const NUM_THREADS: usize = 10;
        const OPS_PER_THREAD: usize = 20;

        let store = Arc::new(InMemoryStorageEngine::new());
        let barrier = Arc::new(Barrier::new(NUM_THREADS));
        let seed: u64 = 0xCAFEE220;

        let mut handles = Vec::new();

        for t in 0..NUM_THREADS {
            let store = store.clone();
            let barrier = barrier.clone();

            handles.push(tokio::spawn(async move {
                // Wait for all worker tasks to reach starting line
                barrier.wait().await;

                for i in 0..OPS_PER_THREAD {
                    let raw_tx_id = (t * OPS_PER_THREAD + i + 1) as u64;
                    let tx_id =
                        TxId::try_new(raw_tx_id).expect("valid tx id in barrier concurrency test");

                    let key = format!("thread_{}_key_{}", t, i).into_bytes();
                    let val = format!("thread_{}_val_{}_seed_{}", t, i, seed).into_bytes();

                    store.put(tx_id, &key, &val).await.unwrap();
                    store.commit(tx_id).await.unwrap();

                    // Read-your-own-writes check immediately after commit
                    let fetched = store.get(&key).await.unwrap().expect("value exists");
                    assert_eq!(&fetched[..], &val[..], "seed {}", seed);
                }
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        // Verify total committed sequences and records
        let final_seq = store.last_seq_no().await.unwrap();
        assert_eq!(final_seq, (NUM_THREADS * OPS_PER_THREAD) as u64);

        let all_records = store
            .scan(std::ops::Bound::Unbounded, std::ops::Bound::Unbounded, None)
            .await
            .unwrap();
        assert_eq!(all_records.len(), NUM_THREADS * OPS_PER_THREAD);
    }

    #[tokio::test]
    async fn test_in_memory_store_concurrency_race_same_keys() {
        const NUM_WORKERS: usize = 8;
        const ITERATIONS: usize = 25;

        let store = Arc::new(InMemoryStorageEngine::new());
        let barrier = Arc::new(Barrier::new(NUM_WORKERS));
        let seed: u64 = 0xDEADBEEF;

        let mut handles = Vec::new();

        for w in 0..NUM_WORKERS {
            let store = store.clone();
            let barrier = barrier.clone();

            handles.push(tokio::spawn(async move {
                barrier.wait().await;

                for i in 0..ITERATIONS {
                    let tx_id = TxId::try_new((w * ITERATIONS + i + 1000) as u64).unwrap();
                    let shared_key = format!("shared_key_{}", i % 5).into_bytes();
                    let shared_val = format!("worker_{}_iter_{}_seed_{}", w, i, seed).into_bytes();

                    // Alternating put and delete
                    if (w + i) % 2 == 0 {
                        store.put(tx_id, &shared_key, &shared_val).await.unwrap();
                    } else {
                        store.delete(tx_id, &shared_key).await.unwrap();
                    }

                    store.commit(tx_id).await.unwrap();

                    // Perform a concurrent prefix scan while other workers mutate shared keys
                    let _prefix_res = store.scan_prefix(b"shared_key_").await.unwrap();
                }
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        // Verify store remains consistent without panics or lock corruption
        let last_tx = store.last_tx_id().await.unwrap();
        assert!(last_tx.inner() >= 1000, "seed {}", seed);
    }
}
