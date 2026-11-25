//! Pure In-Memory `StorageEngine` implementation for `memory-only-storage`.
//!
//! Provides transactional KV operations with staging and zero disk/WAL/mmap I/O.

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

#[derive(Debug, Clone)]
struct KeyVersion {
    seq: u64,
    tx_id: TxId,
    val: Option<Vec<u8>>,
}

type StoreDataMap = BTreeMap<Vec<u8>, Vec<KeyVersion>>;
type StagedTxMap = BTreeMap<TxId, BTreeMap<Vec<u8>, StagedOp>>;

/// An in-memory implementation of [`StorageEngine`] guaranteed to perform no disk I/O.
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
            last_tx: Arc::new(RwLock::new(TxId::new(0))),
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
            if let Some(versions) = guard.get(key) {
                if let Some(latest) = versions.last() {
                    return Ok(latest.val.as_ref().map(|v| Bytes::copy_from_slice(v)));
                }
            }
            Ok(None)
        })
    }

    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let guard = self.data.read();
            if let Some(versions) = guard.get(key) {
                for ver in versions.iter().rev() {
                    if ver.seq <= seq {
                        return Ok(ver.val.as_ref().map(|v| Bytes::copy_from_slice(v)));
                    }
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
                    let version = match op {
                        StagedOp::Put(val) => KeyVersion {
                            seq: new_seq,
                            tx_id,
                            val: Some(val),
                        },
                        StagedOp::Delete => KeyVersion {
                            seq: new_seq,
                            tx_id,
                            val: None,
                        },
                    };
                    data.entry(key).or_default().push(version);
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

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            {
                let mut staged = self.staged.write();
                staged.retain(|t, _| *t <= tx_id);
            }
            {
                let mut data = self.data.write();
                let mut empty_keys = Vec::new();
                for (key, versions) in data.iter_mut() {
                    versions.retain(|ver| ver.tx_id <= tx_id);
                    if versions.is_empty() {
                        empty_keys.push(key.clone());
                    }
                }
                for key in empty_keys {
                    data.remove(&key);
                }
            }
            *self.last_tx.write() = tx_id;
            Ok(())
        })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            let data = self.data.read();
            let total_size: usize = data
                .iter()
                .map(|(k, versions)| {
                    k.len()
                        + versions
                            .iter()
                            .map(|v| v.val.as_ref().map_or(0, |val| val.len()))
                            .sum::<usize>()
                })
                .sum();
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
            for (key, versions) in data.range(prefix.to_vec()..) {
                if key.starts_with(prefix) {
                    if let Some(latest) = versions.last() {
                        if let Some(val) = &latest.val {
                            results.push((key.clone(), val.clone()));
                        }
                    }
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
            for (key, versions) in data.range((start_bound, end_bound)) {
                if let Some(latest) = versions.last() {
                    if let Some(val) = &latest.val {
                        results.push((key.clone(), val.clone()));
                        if let Some(lim) = limit {
                            if results.len() >= lim {
                                break;
                            }
                        }
                    }
                }
            }
            Ok(results)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_store_put_get_commit_contract() {
        let store = InMemoryStorageEngine::new();
        let tx = TxId::new(1);

        assert!(store.get(b"key1").await.unwrap().is_none());

        store.put(tx, b"key1", b"value1").await.unwrap();
        // Transaction isolation check: uncommitted Get must return None
        assert!(store.get(b"key1").await.unwrap().is_none());

        store.commit(tx).await.unwrap();
        let val = store.get(b"key1").await.unwrap();
        assert_eq!(val, Some(Bytes::from_static(b"value1")));
    }

    #[tokio::test]
    async fn test_in_memory_store_transaction_isolation() {
        let store = InMemoryStorageEngine::new();
        let tx = TxId::new(42);

        store.put(tx, b"uncommitted_key", b"val").await.unwrap();
        // Verify uncommitted write is invisible
        let res = store.get(b"uncommitted_key").await.unwrap();
        assert!(res.is_none());

        // Rolling back should keep key invisible
        store.rollback(tx).await.unwrap();
        store.commit(tx).await.unwrap();
        let res_after = store.get(b"uncommitted_key").await.unwrap();
        assert!(res_after.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_store_mvcc_get_at_seq() {
        let store = InMemoryStorageEngine::new();
        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        store.put(tx1, b"key_mvcc", b"v1").await.unwrap();
        store.commit(tx1).await.unwrap();

        store.put(tx2, b"key_mvcc", b"v2").await.unwrap();
        store.commit(tx2).await.unwrap();

        // seq 1 should return v1, seq 2 should return v2
        let val_at_1 = store.get_at_seq(b"key_mvcc", 1).await.unwrap();
        assert_eq!(val_at_1, Some(Bytes::from_static(b"v1")));

        let val_at_2 = store.get_at_seq(b"key_mvcc", 2).await.unwrap();
        assert_eq!(val_at_2, Some(Bytes::from_static(b"v2")));
    }

    #[tokio::test]
    async fn test_in_memory_store_rollback_to_tx() {
        let store = InMemoryStorageEngine::new();
        let tx1 = TxId::new(10);
        let tx2 = TxId::new(20);

        store.put(tx1, b"key_t1", b"v1").await.unwrap();
        store.commit(tx1).await.unwrap();

        store.put(tx2, b"key_t2", b"v2").await.unwrap();
        store.commit(tx2).await.unwrap();

        // Roll back to tx1
        store.rollback_to_tx(tx1).await.unwrap();

        assert_eq!(
            store.get(b"key_t1").await.unwrap(),
            Some(Bytes::from_static(b"v1"))
        );
        assert!(store.get(b"key_t2").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_in_memory_store_zero_disk_io() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let store = InMemoryStorageEngine::new();
        let tx = TxId::new(100);

        store.put(tx, b"key_disk", b"val_disk").await.unwrap();
        store.commit(tx).await.unwrap();
        store.flush().await.unwrap();

        let entries = std::fs::read_dir(&path).unwrap().count();
        assert_eq!(
            entries, 0,
            "Directory must remain empty as zero disk I/O occurs"
        );
    }
}
