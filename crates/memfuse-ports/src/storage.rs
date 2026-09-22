//! Storage engine subsystem trait definitions and statistics.

use super::BoxFuture;
use crate::types::*;
use crate::Result;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Statistics for the storage engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageStats {
    /// Number of SSTable segments.
    pub num_segments: usize,
    /// Total size of all SSTables in bytes.
    pub total_size_bytes: u64,
    /// Total size of memtables in bytes.
    pub memtable_size_bytes: u64,
}

// INVARIANT: Implementor: LsmStorage (memfuse-store/src/lsm.rs)
// Lifecycle: put/delete → commit/rollback → flush(background).

/// Harte Obergrenze für die Anzahl distinkter Keys, die eine StorageEngine-
/// Implementierung während eines einzelnen scan()/scan_prefix_bounded()-Aufrufs
/// intern akkumulieren darf, BEVOR limit/cursor angewendet wird. Verhindert
/// unbegrenztes Speicherwachstum bei sehr breiten Scans, unabhängig vom vom
/// Aufrufer angeforderten `limit`. Muss größer als jedes sinnvolle `limit` sein,
/// um normale paginierte Nutzung nicht zu beeinträchtigen.
pub const MAX_SCAN_MERGE_ACCUMULATOR: usize = 100_000;

/// Synchronous storage read trait — abstracts sync read access to storage (Spec §20.2 / ADR-N02).
///
/// # Dyn-Kompatibilität
/// Dieser Trait ist durch synchrone Rückgabetypen vtable-kompatibel (dyn-safe).
///
/// # Invarianten
/// - Implementierungen DÜRFEN NICHT paniken (Zero-Panic Doctrine)
/// - Alle Fehler werden über `crate::Result<T>` propagiert
pub trait StorageRead: Send + Sync + 'static {
    /// Retrieves a value by key synchronously.
    fn get(&self, key: &[u8]) -> Result<Option<Bytes>>;

    /// Retrieves a value by key at a specific sequence number (MVCC) synchronously.
    fn get_at_seq(&self, key: &[u8], seq: u64) -> Result<Option<Bytes>>;

    /// Scans a range of keys with the given prefix synchronously.
    fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

    /// Scans a range of keys with the given prefix, limit, and optional cursor pagination synchronously.
    fn scan_prefix_bounded(
        &self,
        prefix: &[u8],
        limit: usize,
        cursor: Option<&[u8]>,
    ) -> Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)> {
        let all = self.scan_prefix(prefix)?;
        let mut results = Vec::new();

        for (k, v) in all {
            if let Some(cur_bytes) = cursor {
                if k.as_slice() <= cur_bytes {
                    continue;
                }
            }
            results.push((k, v));
            if results.len() == limit {
                break;
            }
        }

        let next_cursor = if results.len() == limit {
            results.last().map(|(k, _)| k.clone())
        } else {
            None
        };

        Ok((results, next_cursor))
    }

    /// Scans keys with a prefix, returning only entries visible at or before `seq_no` synchronously.
    fn scan_prefix_at(&self, _prefix: &[u8], _seq_no: u64) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        Err(crate::error::MemFuseError::capability_unsupported(
            "snapshot_read_at",
            "Storage-level snapshot-isolated prefix scan (scan_prefix_at) is not supported by default",
        ))
    }

    /// Scans a range of keys between `start` and `end` bounds, optionally capped at `limit` synchronously.
    fn scan(
        &self,
        start: std::ops::Bound<&[u8]>,
        end: std::ops::Bound<&[u8]>,
        limit: Option<usize>,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

    /// Scans a range of keys with bounds, limit, and cursor synchronously.
    fn scan_bounded(
        &self,
        _start: std::ops::Bound<&[u8]>,
        _end: std::ops::Bound<&[u8]>,
        _limit: usize,
        _cursor: Option<&[u8]>,
    ) -> Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)> {
        Err(crate::error::MemFuseError::capability_unsupported(
            "scan_bounded",
            "Bounded range scan (scan_bounded) is not supported by default",
        ))
    }

    /// Returns storage statistics synchronously.
    fn stats(&self) -> Result<StorageStats>;

    /// Returns the last sequence number committed to storage synchronously.
    fn last_seq_no(&self) -> Result<u64>;

    /// Returns the last transaction ID committed to storage synchronously.
    fn last_tx_id(&self) -> Result<TxId>;
}

/// Asynchronous storage write trait — abstracts async write, commit, and lifecycle operations (Spec §20.2 / ADR-N02).
///
/// # Dyn-Kompatibilität
/// Dieser Trait ist durch explizite `BoxFuture`-Rückgabetypen vtable-kompatibel (dyn-safe).
///
/// # Invarianten
/// - Implementierungen DÜRFEN NICHT paniken (Zero-Panic Doctrine)
/// - Alle Fehler werden über `crate::Result<T>` propagiert
pub trait StorageWrite: Send + Sync + 'static {
    /// Stores a key-value pair as part of a transaction.
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Atomically writes `value` under `key` within transaction ONLY IF no value exists.
    fn put_if_absent<'a>(
        &'a self,
        _tx_id: TxId,
        _key: &'a [u8],
        _value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            Err(crate::error::MemFuseError::capability_unsupported(
                "put_if_absent",
                "Atomic put_if_absent is not supported by default — concrete storage engines must implement it atomically.",
            ))
        })
    }

    /// Stores multiple key-value pairs as part of a transaction.
    fn put_batch<'a>(
        &'a self,
        tx_id: TxId,
        entries: &'a [(Vec<u8>, Vec<u8>)],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            for (key, value) in entries {
                self.put(tx_id, key, value).await?;
            }
            Ok(())
        })
    }

    /// Deletes a key as part of a transaction.
    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Deletes multiple keys as a single logical batch operation.
    fn delete_many<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            let mut deleted = 0u64;
            for key in keys {
                self.delete(tx_id, &key).await?;
                deleted += 1;
            }
            Ok(deleted)
        })
    }

    /// Deletes all key-value pairs starting with `prefix` as part of a transaction.
    fn delete_prefix<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>>;

    /// Commits a transaction — makes writes visible.
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back a transaction.
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back the entire storage state to a specific transaction ID.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Flushes memtable/buffer to disk.
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>;

    /// Pins a checkpoint for sequence number.
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Unpins a checkpoint for sequence number.
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;
}

/// Unified Storage Engine trait — retained for backward compatibility across higher layers.
///
/// # Dyn-Kompatibilität
/// Dieser Trait ist durch explizite `BoxFuture`-Rückgabetypen vtable-kompatibel (dyn-safe).
pub trait StorageEngine: Send + Sync + 'static {
    /// Retrieves a value by key.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Retrieves a value by key at a specific sequence number (MVCC).
    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Stores a key-value pair as part of a transaction.
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Atomically writes `value` under `key` within transaction ONLY IF no value exists.
    fn put_if_absent<'a>(
        &'a self,
        _tx_id: TxId,
        _key: &'a [u8],
        _value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            Err(crate::error::MemFuseError::capability_unsupported(
                "put_if_absent",
                "Atomic put_if_absent is not supported by default — concrete storage engines must implement it atomically.",
            ))
        })
    }

    /// Stores multiple key-value pairs as part of a transaction.
    fn put_batch<'a>(
        &'a self,
        tx_id: TxId,
        entries: &'a [(Vec<u8>, Vec<u8>)],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            for (key, value) in entries {
                self.put(tx_id, key, value).await?;
            }
            Ok(())
        })
    }

    /// Deletes a key as part of a transaction.
    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Deletes multiple keys as a single logical batch operation.
    fn delete_many<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            let mut deleted = 0u64;
            for key in keys {
                self.delete(tx_id, &key).await?;
                deleted += 1;
            }
            Ok(deleted)
        })
    }

    /// Deletes all key-value pairs starting with `prefix` as part of a transaction.
    fn delete_prefix<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            let matching_keys: Vec<Vec<u8>> = self
                .scan_prefix(prefix)
                .await?
                .into_iter()
                .map(|(key, _)| key)
                .collect();
            self.delete_many(tx_id, matching_keys).await
        })
    }

    /// Commits a transaction.
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back a transaction.
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back the entire storage state to a specific transaction ID.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Flushes memtable to disk.
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>;

    /// Returns storage statistics.
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>;

    /// Returns the last sequence number committed to storage.
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>>;

    /// Returns the last transaction ID committed to storage.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;

    /// Pins a checkpoint for sequence number.
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Unpins a checkpoint for sequence number.
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Scans a range of keys with given prefix.
    #[allow(clippy::type_complexity)]
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;

    /// Scans prefix bounded.
    #[allow(clippy::type_complexity)]
    fn scan_prefix_bounded<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        Box::pin(async move {
            let all = self.scan_prefix(prefix).await?;
            let mut results = Vec::new();

            for (k, v) in all {
                if let Some(cur_bytes) = cursor {
                    if k.as_slice() <= cur_bytes {
                        continue;
                    }
                }
                results.push((k, v));
                if results.len() == limit {
                    break;
                }
            }

            let next_cursor = if results.len() == limit {
                results.last().map(|(k, _)| k.clone())
            } else {
                None
            };

            Ok((results, next_cursor))
        })
    }

    /// Scans prefix visible at sequence number.
    #[allow(clippy::type_complexity)]
    fn scan_prefix_at<'a>(
        &'a self,
        _prefix: &'a [u8],
        _seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            Err(crate::error::MemFuseError::capability_unsupported(
                "snapshot_read_at",
                "Storage-level snapshot-isolated prefix scan (scan_prefix_at) is not supported by default — implementors must override this method to guarantee MVCC snapshot isolation.",
            ))
        })
    }

    /// Scans range between start and end bounds.
    #[allow(clippy::type_complexity)]
    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;

    /// Scans range bounded.
    #[allow(clippy::type_complexity)]
    fn scan_bounded<'a>(
        &'a self,
        _start: std::ops::Bound<&'a [u8]>,
        _end: std::ops::Bound<&'a [u8]>,
        _limit: usize,
        _cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        Box::pin(async move {
            Err(crate::error::MemFuseError::capability_unsupported(
                "scan_bounded",
                "Bounded range scan (scan_bounded) is not supported by default",
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_stats_serialization() {
        let s_stats = StorageStats {
            num_segments: 2,
            total_size_bytes: 2048,
            memtable_size_bytes: 512,
        };
        let ser = serde_json::to_string(&s_stats).unwrap();
        let deser: StorageStats = serde_json::from_str(&ser).unwrap();
        assert_eq!(s_stats.total_size_bytes, deser.total_size_bytes);
    }

    struct MockReadStore {
        data: std::collections::BTreeMap<Vec<u8>, Vec<u8>>,
    }

    impl StorageRead for MockReadStore {
        fn get(&self, key: &[u8]) -> Result<Option<Bytes>> {
            Ok(self.data.get(key).cloned().map(Bytes::from))
        }

        fn get_at_seq(&self, key: &[u8], _seq: u64) -> Result<Option<Bytes>> {
            Ok(self.data.get(key).cloned().map(Bytes::from))
        }

        fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
            Ok(self
                .data
                .iter()
                .filter(|(k, _)| k.starts_with(prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect())
        }

        fn scan(
            &self,
            start: std::ops::Bound<&[u8]>,
            end: std::ops::Bound<&[u8]>,
            limit: Option<usize>,
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
            let mut res = Vec::new();
            for (k, v) in self.data.range::<[u8], _>((start, end)) {
                res.push((k.clone(), v.clone()));
                if let Some(lim) = limit {
                    if res.len() >= lim {
                        break;
                    }
                }
            }
            Ok(res)
        }

        fn stats(&self) -> Result<StorageStats> {
            Ok(StorageStats {
                num_segments: 1,
                total_size_bytes: 100,
                memtable_size_bytes: 50,
            })
        }

        fn last_seq_no(&self) -> Result<u64> {
            Ok(42)
        }

        fn last_tx_id(&self) -> Result<TxId> {
            Ok(TxId(1))
        }
    }

    #[test]
    fn test_sync_storage_read_trait() {
        let mut data = std::collections::BTreeMap::new();
        data.insert(b"pfx:1".to_vec(), b"val1".to_vec());
        data.insert(b"pfx:2".to_vec(), b"val2".to_vec());

        let store = MockReadStore { data };
        let val = store.get(b"pfx:1").unwrap().unwrap();
        assert_eq!(val.as_ref(), b"val1");

        let (batch, next_cur) = store.scan_prefix_bounded(b"pfx:", 1, None).unwrap();
        assert_eq!(batch.len(), 1);
        assert_eq!(next_cur, Some(b"pfx:1".to_vec()));

        assert_eq!(store.last_seq_no().unwrap(), 42);
        assert_eq!(store.last_tx_id().unwrap(), TxId(1));
    }
}
