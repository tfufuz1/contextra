//! Storage engine subsystem trait definitions and statistics.

use super::BoxFuture;
use crate::types::*;
use crate::Result;
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

/// Storage Engine trait — abstrahiert die LSM-Tree-Persistenz.
///
/// # Dyn-Kompatibilität
/// Dieser Trait ist durch explizite `BoxFuture`-Rückgabetypen vtable-kompatibel (dyn-safe).
///
/// # Invarianten
/// - Implementierungen DÜRFEN NICHT paniken (Zero-Panic Doctrine)
/// - Alle Fehler werden über `crate::Result<T>` propagiert
pub trait StorageEngine: Send + Sync + 'static {
    /// Retrieves a value by key.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>>;

    /// Retrieves a value by key at a specific sequence number (MVCC).
    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Vec<u8>>>>;

    /// Stores a key-value pair as part of a transaction.
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Atomically writes `value` under `key` within the given transaction ONLY IF no value
    /// currently exists for `key` (as observed within the same transaction's read view and uncommitted staged state).
    ///
    /// # Semantics & MVCC Guarantees
    /// Checks key existence against the storage engine's current committed snapshot and any uncommitted
    /// staged writes for `tx_id`. If the key exists and is non-tombstoned, no write is staged and `Ok(false)`
    /// is returned. The transaction is NOT automatically rolled back by this call.
    /// If the key does not exist (or is tombstoned), `value` is staged for `tx_id` and `Ok(true)` is returned.
    fn put_if_absent<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            if self.get(key).await?.is_some() {
                Ok(false)
            } else {
                self.put(tx_id, key, value).await?;
                Ok(true)
            }
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
    ///
    /// # Performance
    /// Default implementation delegates to sequential `delete()` calls.
    /// Implementors handling large batches (e.g. from `delete_prefix()`)
    /// SHOULD override this with a true batch operation (single lock
    /// acquisition) to avoid per-key lock contention.
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

    /// Deletes all key-value pairs whose key starts with `prefix` as part of a transaction.
    ///
    /// Returns the number of keys staged for deletion.
    ///
    /// Default implementation scans all matching keys, then delegates to [`delete_many`][Self::delete_many].
    /// Concrete implementors handling batch mutations should override `delete_many()` or `delete_prefix()`
    /// with a true batch operation to avoid per-key lock overhead.
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

    /// Commits a transaction — makes writes visible.
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back a transaction — discards staged uncommitted writes for the given ID.
    ///
    /// **Note**: `rollback()` only discards entries currently in the staging buffer.
    /// Once `commit()` has completed, `rollback()` on that `tx_id` is a no-op.
    /// Undoing a physically committed transaction requires a compensating transaction or `rollback_to_tx()`.
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back the entire storage state to a specific transaction ID.
    ///
    /// **Implementor contract**: MUST physically revert all state beyond `tx_id`.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Flushes the memtable to disk.
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>;

    /// Returns storage statistics.
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>;

    /// Returns the last sequence number committed to storage.
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>>;

    /// Returns the last transaction ID committed to storage.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;

    /// Pins a checkpoint for the given sequence number.
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Unpins a checkpoint for the given sequence number.
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Scans a range of keys with the given prefix.
    ///
    /// Für neue Call-Sites bevorzuge `scan_prefix_bounded` — lädt unbegrenzt und kann bei großen Prefixes das Speicherbudget sprengen.
    #[allow(clippy::type_complexity)]
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;

    /// Wie `scan_prefix`, aber mit hartem Limit auf die Anzahl zurückgegebener Einträge und
    /// optionalem Cursor (letzter zurückgegebener Key aus dem vorherigen Aufruf) für Pagination.
    ///
    /// # Cursor-Semantik
    /// Der Cursor dient als exklusive untere Schranke (`Bound::Excluded(cursor)`): Es werden nur Einträge
    /// zurückgegeben, deren Key lexikographisch *strikt größer* als der `cursor` ist (`k > cursor`).
    /// Falls der Cursor-Key nicht (mehr) im Datensatz existiert (z. B. durch Löschedits zwischen
    /// paginierten Aufrufen), setzt die Pagination nahtlos ab dem nächstgrößeren Key fort,
    /// anstatt ein leeres Ergebnis zurückzugeben.
    ///
    /// Bevorzugt gegenüber `scan_prefix` für jeden neuen Call-Site, der potenziell große
    /// Ergebnismengen erwarten muss.
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

    /// Scans keys with a prefix, returning only entries visible at or before `seq_no`.
    ///
    /// # Contract
    /// Must respect MVCC snapshot isolation.
    ///
    /// # Errors
    /// Returns [`MemFuseError::CapabilityUnsupported`][crate::MemFuseError::CapabilityUnsupported]
    /// with capability `"snapshot_read_at"` if snapshot-isolated prefix scan is not implemented.
    /// Tested via `capability_coverage` test module.
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

    /// Scans a range of keys between `start` and `end` bounds, optionally capped at `limit`.
    #[allow(clippy::type_complexity)]
    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;

    /// Scans a range of keys with start and end bounds, limit, and pagination cursor.
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
    use ahash::AHashMap;

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

    #[tokio::test]
    async fn test_storage_scan_prefix_at_capability() {
        struct StorageEnginePlaceholder;
        impl StorageEngine for StorageEnginePlaceholder {
            fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn get_at_seq<'a>(
                &'a self,
                _: &'a [u8],
                _: u64,
            ) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn put<'a>(&'a self, _: TxId, _: &'a [u8], _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
                Box::pin(async move {
                    Ok(StorageStats {
                        num_segments: 0,
                        total_size_bytes: 0,
                        memtable_size_bytes: 0,
                    })
                })
            }
            fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
                Box::pin(async move { Ok(0) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn scan_prefix<'a>(
                &'a self,
                _: &'a [u8],
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn scan<'a>(
                &'a self,
                _: std::ops::Bound<&'a [u8]>,
                _: std::ops::Bound<&'a [u8]>,
                _: Option<usize>,
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
        }

        let placeholder = StorageEnginePlaceholder;
        let result = placeholder.scan_prefix_at(b"prefix", 0).await;
        assert!(matches!(
            result,
            Err(crate::MemFuseError::CapabilityUnsupported { ref capability, .. }) if capability == "snapshot_read_at"
        ));
    }

    #[tokio::test]
    async fn test_storage_engine_default_put_batch() {
        type KVPair = (Vec<u8>, Vec<u8>);
        type Log = std::sync::Arc<std::sync::Mutex<Vec<KVPair>>>;
        struct MockStorage(Log);
        impl StorageEngine for MockStorage {
            fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn get_at_seq<'a>(
                &'a self,
                _: &'a [u8],
                _: u64,
            ) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn put<'a>(
                &'a self,
                _: TxId,
                key: &'a [u8],
                value: &'a [u8],
            ) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move {
                    self.0.lock().unwrap().push((key.to_vec(), value.to_vec()));
                    Ok(())
                })
            }
            fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
                Box::pin(async move {
                    Ok(StorageStats {
                        num_segments: 0,
                        total_size_bytes: 0,
                        memtable_size_bytes: 0,
                    })
                })
            }
            fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
                Box::pin(async move { Ok(0) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn scan_prefix<'a>(
                &'a self,
                _: &'a [u8],
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn scan<'a>(
                &'a self,
                _: std::ops::Bound<&'a [u8]>,
                _: std::ops::Bound<&'a [u8]>,
                _: Option<usize>,
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
        }

        let store = MockStorage(std::sync::Arc::new(std::sync::Mutex::new(vec![])));
        let entries = vec![
            (b"k1".to_vec(), b"v1".to_vec()),
            (b"k2".to_vec(), b"v2".to_vec()),
        ];
        store.put_batch(TxId(1), &entries).await.unwrap();
        assert_eq!(store.0.lock().unwrap().len(), 2);

        // Test scan_prefix_at default error
        let res = store.scan_prefix_at(b"pre", 1).await;
        match res {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "snapshot_read_at");
            }
            _ => panic!("Expected CapabilityUnsupported for scan_prefix_at"),
        }
    }

    #[tokio::test]
    async fn test_scan_prefix_bounded_cursor_semantics() {
        struct MemoryStorage {
            data: Vec<(Vec<u8>, Vec<u8>)>,
        }

        impl StorageEngine for MemoryStorage {
            fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn get_at_seq<'a>(
                &'a self,
                _: &'a [u8],
                _: u64,
            ) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn put<'a>(&'a self, _: TxId, _: &'a [u8], _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
                Box::pin(async move {
                    Ok(StorageStats {
                        num_segments: 0,
                        total_size_bytes: 0,
                        memtable_size_bytes: 0,
                    })
                })
            }
            fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
                Box::pin(async move { Ok(0) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn scan_prefix<'a>(
                &'a self,
                prefix: &'a [u8],
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move {
                    let matching = self
                        .data
                        .iter()
                        .filter(|(k, _)| k.starts_with(prefix))
                        .cloned()
                        .collect();
                    Ok(matching)
                })
            }
            fn scan<'a>(
                &'a self,
                _: std::ops::Bound<&'a [u8]>,
                _: std::ops::Bound<&'a [u8]>,
                _: Option<usize>,
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
        }

        let dataset = MemoryStorage {
            data: vec![
                (b"pfx:1".to_vec(), b"v1".to_vec()),
                (b"pfx:2".to_vec(), b"v2".to_vec()),
                (b"pfx:3".to_vec(), b"v3".to_vec()),
                (b"pfx:4".to_vec(), b"v4".to_vec()),
                (b"pfx:5".to_vec(), b"v5".to_vec()),
            ],
        };

        // 1. Valid cursor in dataset -> yields next page starting strictly after cursor
        let (batch1, next_cur1) = dataset
            .scan_prefix_bounded(b"pfx:", 2, Some(b"pfx:2"))
            .await
            .unwrap();
        assert_eq!(
            batch1,
            vec![
                (b"pfx:3".to_vec(), b"v3".to_vec()),
                (b"pfx:4".to_vec(), b"v4".to_vec()),
            ]
        );
        assert_eq!(next_cur1, Some(b"pfx:4".to_vec()));

        // 2. Cursor references key deleted between calls (e.g. b"pfx:2" deleted, cursor = b"pfx:2")
        let dataset_after_delete = MemoryStorage {
            data: vec![
                (b"pfx:1".to_vec(), b"v1".to_vec()),
                // pfx:2 was deleted!
                (b"pfx:3".to_vec(), b"v3".to_vec()),
                (b"pfx:4".to_vec(), b"v4".to_vec()),
                (b"pfx:5".to_vec(), b"v5".to_vec()),
            ],
        };
        let (batch2, next_cur2) = dataset_after_delete
            .scan_prefix_bounded(b"pfx:", 2, Some(b"pfx:2"))
            .await
            .unwrap();
        assert_eq!(
            batch2,
            vec![
                (b"pfx:3".to_vec(), b"v3".to_vec()),
                (b"pfx:4".to_vec(), b"v4".to_vec()),
            ]
        );
        assert_eq!(next_cur2, Some(b"pfx:4".to_vec()));

        // 3. Cursor is None -> scans from beginning up to limit
        let (batch3, next_cur3) = dataset.scan_prefix_bounded(b"pfx:", 2, None).await.unwrap();
        assert_eq!(
            batch3,
            vec![
                (b"pfx:1".to_vec(), b"v1".to_vec()),
                (b"pfx:2".to_vec(), b"v2".to_vec()),
            ]
        );
        assert_eq!(next_cur3, Some(b"pfx:2".to_vec()));

        // 4. Empty scan result -> returns (vec![], None)
        let empty_dataset = MemoryStorage { data: vec![] };
        let (batch4, next_cur4) = empty_dataset
            .scan_prefix_bounded(b"pfx:", 2, Some(b"pfx:1"))
            .await
            .unwrap();
        assert!(batch4.is_empty());
        assert_eq!(next_cur4, None);
    }

    #[tokio::test]
    async fn test_delete_many_default_impl_deletes_all_keys() {
        struct MockStorage {
            data: std::sync::Arc<std::sync::Mutex<AHashMap<Vec<u8>, Vec<u8>>>>,
            delete_call_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        }

        impl StorageEngine for MockStorage {
            fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(self.data.lock().unwrap().get(key).cloned()) })
            }
            fn get_at_seq<'a>(
                &'a self,
                _: &'a [u8],
                _: u64,
            ) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn put<'a>(
                &'a self,
                _: TxId,
                key: &'a [u8],
                value: &'a [u8],
            ) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move {
                    self.data
                        .lock()
                        .unwrap()
                        .insert(key.to_vec(), value.to_vec());
                    Ok(())
                })
            }
            fn delete<'a>(&'a self, _: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move {
                    self.delete_call_count
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    self.data.lock().unwrap().remove(key);
                    Ok(())
                })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
                Box::pin(async move {
                    Ok(StorageStats {
                        num_segments: 0,
                        total_size_bytes: 0,
                        memtable_size_bytes: 0,
                    })
                })
            }
            fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
                Box::pin(async move { Ok(0) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn scan_prefix<'a>(
                &'a self,
                prefix: &'a [u8],
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move {
                    let map = self.data.lock().unwrap();
                    let mut res = Vec::new();
                    for (k, v) in map.iter() {
                        if k.starts_with(prefix) {
                            res.push((k.clone(), v.clone()));
                        }
                    }
                    Ok(res)
                })
            }
            fn scan<'a>(
                &'a self,
                _: std::ops::Bound<&'a [u8]>,
                _: std::ops::Bound<&'a [u8]>,
                _: Option<usize>,
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
        }

        let map = std::sync::Arc::new(std::sync::Mutex::new(AHashMap::default()));
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let store = MockStorage {
            data: map.clone(),
            delete_call_count: count.clone(),
        };

        store.put(TxId(1), b"pref:1", b"v1").await.unwrap();
        store.put(TxId(1), b"pref:2", b"v2").await.unwrap();
        store.put(TxId(1), b"other:1", b"v3").await.unwrap();

        let deleted = store.delete_prefix(TxId(2), b"pref:").await.unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
        assert!(store.get(b"pref:1").await.unwrap().is_none());
        assert!(store.get(b"pref:2").await.unwrap().is_none());
        assert_eq!(store.get(b"other:1").await.unwrap().unwrap(), b"v3");
    }
}
