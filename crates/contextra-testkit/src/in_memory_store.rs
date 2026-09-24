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
use contextra_types::TxId;
use contextra_types::Result;

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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_store_put_get_commit() {
        let store = InMemoryStorageEngine::new();
        let tx = TxId::new(1);

        assert!(store.get(b"key1").await.unwrap().is_none());

        store.put(tx, b"key1", b"value1").await.unwrap();
        // Not visible before commit
        assert!(store.get(b"key1").await.unwrap().is_none());

        store.commit(tx).await.unwrap();
        let val = store.get(b"key1").await.unwrap();
        assert_eq!(val, Some(Bytes::from_static(b"value1")));
    }

    #[tokio::test]
    async fn test_in_memory_store_rollback() {
        let store = InMemoryStorageEngine::new();
        let tx = TxId::new(2);

        store.put(tx, b"key_rb", b"val").await.unwrap();
        store.rollback(tx).await.unwrap();
        store.commit(tx).await.unwrap();

        assert!(store.get(b"key_rb").await.unwrap().is_none());
    }
}
