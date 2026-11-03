//! Storage engine subsystem trait definitions and statistics.

use super::BoxFuture;
use crate::{ContextraError, Result, TxId};
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

// INVARIANT: Implementor: LsmStorage (contextra-store/src/lsm.rs)
// Lifecycle: put/delete → commit/rollback → flush(background).

/// Harte Obergrenze für die Anzahl distinkter Keys, die eine StorageEngine-
/// Implementierung während eines einzelnen scan()/scan_prefix_bounded()-Aufrufs
/// intern akkumulieren darf, BEVOR limit/cursor angewendet wird. Verhindert
/// unbegrenztes Speicherwachstum bei sehr breiten Scans, unabhängig vom vom
/// Aufrufer angeforderten `limit`. Muss größer als jedes sinnvolle `limit` sein,
/// um normale paginierte Nutzung nicht zu beeinträchtigen.
pub const MAX_SCAN_MERGE_ACCUMULATOR: usize = 100_000;

/// Unified Storage Engine trait — retained for backward compatibility across higher layers.
///
/// # Dyn-Kompatibilität
/// Dieser Trait ist durch explizite `BoxFuture`-Rückgabetypen vtable-kompatibel (dyn-safe).
pub trait StorageEngine: Send + Sync + 'static {
    // TOCTOU-ACCEPTED: StorageEngine provides default batching/tracking delegate methods.
    /// Retrieves a value by key.
    ///
    /// # Fehler
    /// Liefert `ContextraError::Io` bei Lesefehlern oder `ContextraError::NotFound` falls Schlüssel fehlt.
    ///
    /// # Crash
    /// Reiner Lesezugriff; modifiziert keinen persistenten Zustand.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Retrieves a value by key at a specific sequence number (MVCC).
    ///
    /// # Fehler
    /// Liefert `ContextraError::Io` bei I/O-Fehlern oder `ContextraError::NotFound`.
    ///
    /// # Crash
    /// Lesezugriff auf historische Snapshot-Version; keine Zustandsmutation.
    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Retrieves a value by key tracked within transaction `tx_id` for SSI read-set validation.
    ///
    /// # ReadSet Tracking
    /// Untracked: Default implementation delegates to [`StorageEngine::get`] and registriert KEINE Lesezugriffe im ReadSet.
    /// Concrete SSI-enabled engines (e.g. `LsmStorage`) override this method to record keys in the transaction's `ReadSet`.
    fn get_tracked<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.get(key)
    }

    /// Retrieves a value by key at `seq` tracked within transaction `tx_id` for SSI read-set validation.
    ///
    /// # ReadSet Tracking
    /// Untracked: Default implementation delegates to [`StorageEngine::get_at_seq`] and registriert KEINE Lesezugriffe im ReadSet.
    /// Concrete SSI-enabled engines (e.g. `LsmStorage`) override this method to record keys in the transaction's `ReadSet`.
    fn get_at_seq_tracked<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.get_at_seq(key, seq)
    }

    /// Scans a range of keys with given prefix tracked within transaction `tx_id` for SSI read-set validation.
    ///
    /// # ReadSet Tracking
    /// Untracked: Default implementation delegates to [`StorageEngine::scan_prefix`] and registriert KEINE Lesezugriffe im ReadSet.
    /// Concrete SSI-enabled engines (e.g. `LsmStorage`) override this method to record keys/prefixes in the transaction's `ReadSet`.
    #[allow(clippy::type_complexity)]
    fn scan_prefix_tracked<'a>(
        &'a self,
        _tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.scan_prefix(prefix)
    }

    /// Returns `true` if this storage engine implementation actively tracks read-sets for SSI validation.
    ///
    /// Default implementation returns `false`.
    fn supports_ssi_tracking(&self) -> bool {
        false
    }

    /// Stores a key-value pair as part of a transaction.
    ///
    /// # Fehler
    /// Liefert `ContextraError::Io` bei Schreibfehlern oder `ContextraError::Conflict` bei MVCC-Konflikten.
    ///
    /// # Crash
    /// Schreibt in die Transaktions-Staging-Puffer; Persistierung erfolgt erst beim Commit.
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Atomically writes `value` under `key` within transaction ONLY IF no value exists.
    fn put_if_absent<'a>(
        &'a self,
        _tx_id: TxId,
        _key: &'a [u8],
        _value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            Err(ContextraError::capability_unsupported(
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
    ///
    /// # Fehler
    /// Liefert `ContextraError::Io` bei I/O-Fehlern.
    ///
    /// # Crash
    /// Schreibt einen Tombstone in den Transaktionspuffer; persistent erst nach Commit.
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
    // TOCTOU-ACCEPTED
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
    ///
    /// # Fehler
    /// Liefert `ContextraError::Io` bei WAL- oder fsync-Fehlern.
    ///
    /// # Crash
    /// Scheitert ein fsync während des Commits, wechselt der Speicher unverzüglich in den Zustand `Poisoned` (P14, D-55).
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back a transaction.
    ///
    /// # Fehler
    /// Liefert `ContextraError::Io` bei internen Fehlern.
    ///
    /// # Crash
    /// Verwirft unpersistierte Transaktionsdaten im RAM; atomar und crash-sicher.
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back the entire storage state to a specific transaction ID.
    ///
    /// # Fehler
    /// Liefert Fehler falls Transaktion nicht existiert oder unzulässig ist.
    ///
    /// # Crash
    /// Stellt vorherigen persistenten Zustand wieder her.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Flushes memtable to disk.
    ///
    /// # Fehler
    /// Liefert `ContextraError::Io` bei Schreib- oder Sync-Fehlern.
    ///
    /// # Crash
    /// Führt durable sync auf der neuen SSTable und dem Manifest aus; bei fsync-Fehler tritt Poisoning ein.
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>;

    /// Returns storage statistics.
    ///
    /// # Fehler
    /// Liefert Fehler bei Problemen beim Auslesen von Metriken.
    ///
    /// # Crash
    /// Reine Speicherabfrage ohne Seiteneffekte.
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>;

    /// Returns the last sequence number committed to storage.
    ///
    /// # Fehler
    /// Liefert Fehler falls Sequenz nicht ermittelbar ist.
    ///
    /// # Crash
    /// Deterministische In-Memory / Manifest-Abfrage.
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>>;

    /// Returns the last transaction ID committed to storage.
    ///
    /// # Fehler
    /// Liefert Fehler falls TxId nicht ermittelbar ist.
    ///
    /// # Crash
    /// Deterministische Zustandsabfrage.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;

    /// Pins a sequence number snapshot for durability / retention.
    ///
    /// # Fehler
    /// Liefert Fehler bei ungültiger Sequenz.
    ///
    /// # Crash
    /// Registriert Snapshot im RAM / Manifest.
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Unpins a sequence number snapshot.
    ///
    /// # Fehler
    /// Liefert Fehler falls Snapshot unbekannt ist.
    ///
    /// # Crash
    /// Entfernt Snapshot-Pin; gibt Ressourcen für GC frei.
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Scans a range of keys with given prefix.
    ///
    /// # Fehler
    /// Liefert `ContextraError::Io` bei Lesefehlern.
    ///
    /// # Crash
    /// Reiner Lese-Scan ohne Mutation.
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
            Err(ContextraError::capability_unsupported(
                "snapshot_read_at",
                "Storage-level snapshot-isolated prefix scan (scan_prefix_at) is not supported by default — implementors must override this method to guarantee MVCC snapshot isolation.",
            ))
        })
    }

    /// Scans range between start and end bounds.
    ///
    /// # Fehler
    /// Liefert `ContextraError::Io` bei Lesefehlern.
    ///
    /// # Crash
    /// Reiner Bereichs-Scan ohne Mutation.
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
            Err(ContextraError::capability_unsupported(
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
    fn test_storage_stats_serialization() -> Result<()> {
        let s_stats = StorageStats {
            num_segments: 2,
            total_size_bytes: 2048,
            memtable_size_bytes: 512,
        };
        let ser = serde_json::to_string(&s_stats)
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        let deser: StorageStats =
            serde_json::from_str(&ser).map_err(|e| ContextraError::Serialization(e.to_string()))?;
        assert_eq!(s_stats.total_size_bytes, deser.total_size_bytes);
        Ok(())
    }
}
