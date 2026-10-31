//! # In-Memory-Speicher-Engine für schnelle Tests (`in_memory_store`)
//!
//! Dieses Modul stellt mit [`InMemoryStorageEngine`] eine rein speicherbasierte Implementierung
//! des [`contextra_ports::StorageEngine`]-Ports bereit.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Port:** Implementiert [`contextra_ports::StorageEngine`].
//! - **Keine Platten-I/O:** Führt keinerlei Dateisystemzugriffe oder `fsync`-Befehle aus.
//! - **Transaktionales Staging:** Unterstützt Transaktionen (`TxId`), `put`, `delete`,
//!   `commit` und `rollback` vollständig im Arbeitsspeicher.
//! - **MVCC-Unterstützung:** Speichert Sequenznummern pro Schlüsselwert-Paar, um
//!   zeitschnittbasierte Leseoperationen ([`contextra_ports::StorageEngine::get_at_seq`],
//!   [`contextra_ports::StorageEngine::scan_prefix_at`]) zu bedienen.
//!
//! ## Nebenläufigkeit & Thread-Sicherheit
//! - Hauptdaten (`data`) und Staging-Bereich (`staged`) sind jeweils in [`parking_lot::RwLock`] gekapselt.
//! - Die Sequenznummer (`seq`) wird über ein [`std::sync::atomic::AtomicU64`] mit `SeqCst`-Ordnung verwaltet.
//! - Mehrere Transaktionen können parallel stagen; beim Commit wird eine neue Sequenznummer
//!   vergeben und die Daten atomar in den Hauptspeicher überführt.

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

// ANCHOR[contextra-testkit:STAGED-OP-ENUM-001]
/// Repräsentiert eine unbestätigte Operation innerhalb einer Transaktion.
#[derive(Debug, Clone)]
enum StagedOp {
    /// Einfügen oder Aktualisieren eines Wertes.
    Put(Vec<u8>),
    /// Löschmarkierung für einen Schlüssel.
    Delete,
}

/// Typalias für den bestätigten Datenbestand: Schlüssel -> (Wert, Sequenznummer).
type StoreDataMap = BTreeMap<Vec<u8>, (Vec<u8>, u64)>;
/// Typalias für unbestätigte Transaktionen: Transaktions-ID -> (Schlüssel -> Operation).
type StagedTxMap = BTreeMap<TxId, BTreeMap<Vec<u8>, StagedOp>>;

// ANCHOR[contextra-testkit:IN-MEM-STORE-STRUCT-001]
/// Thread-sichere In-Memory-Implementierung von `StorageEngine` für Tests.
///
/// **Zweck:** Ermöglicht Unit- und Integrationstests von Engines und Abfrageschichten,
/// ohne Festplatten-I/O oder temporäre Verzeichnisse anlegen zu müssen.
///
/// **Transaktionsablauf:**
/// 1. `put` oder `delete` schreibt Operationen in die Staging-Tabelle `staged[tx_id]`.
/// 2. Vor dem Commit sind diese Änderungen für `get` oder Scans unsichtbar.
/// 3. `commit(tx_id)` entnimmt die Operationen aus dem Staging, vergibt eine neue Sequenznummer
///    und schreibt sie in die globale `data`-Map.
/// 4. `rollback(tx_id)` verwirft die Änderungen aus `staged` ersatzlos.
#[derive(Debug, Clone)]
pub struct InMemoryStorageEngine {
    /// Bestätigter globaler Datenbestand.
    data: Arc<RwLock<StoreDataMap>>,
    /// Unbestätigte Staging-Puffer nach Transaktions-ID geordnet.
    staged: Arc<RwLock<StagedTxMap>>,
    /// Globaler Sequenznummern-Zähler.
    seq: Arc<AtomicU64>,
    /// ID der zuletzt erfolgreich bestätigten Transaktion.
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
    // ANCHOR[contextra-testkit:IN-MEM-STORE-NEW-001]
    /// Erstellt eine neue, leere In-Memory-Speicher-Engine.
    pub fn new() -> Self {
        Self::default()
    }
}

// ANCHOR[contextra-testkit:IN-MEM-STORE-TRAIT-IMPL-001]
impl StorageEngine for InMemoryStorageEngine {
    /// Liest den aktuellsten bestätigten Wert eines Schlüssels aus.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let guard = self.data.read();
            Ok(guard.get(key).map(|(val, _)| Bytes::copy_from_slice(val)))
        })
    }

    /// Liest den Wert eines Schlüssels zum Stand einer bestimmten Sequenznummer `seq` aus.
    ///
    /// Liefert nur dann einen Wert zurück, wenn die Eintragssequenznummer `<= seq` ist.
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

    /// Trägt eine `Put`-Operation in den Staging-Puffer der Transaktion `tx_id` ein.
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

    /// Fügt einen Schlüssel nur dann ein, wenn er weder bestätigt noch im Staging vorhanden ist.
    ///
    /// Gibt `true` zurück, wenn der Eintrag erfolgreich gestaged wurde, `false` falls bereits vorhanden.
    fn put_if_absent<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            let mut staged = self.staged.write();
            let data = self.data.read();
            if data.contains_key(key) {
                return Ok(false);
            }
            if let Some(ops) = staged.get(&tx_id) {
                if ops.contains_key(key) {
                    return Ok(false);
                }
            }
            staged
                .entry(tx_id)
                .or_default()
                .insert(key.to_vec(), StagedOp::Put(value.to_vec()));
            Ok(true)
        })
    }

    /// Trägt eine `Delete`-Operation (Löschmarker) in den Staging-Puffer der Transaktion ein.
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

    /// Bestätigt alle Änderungen einer Transaktion und macht sie global sichtbar.
    ///
    /// **Ablauf:**
    /// 1. Entnimmt alle Operationen der `tx_id` aus `staged`.
    /// 2. Inkrementiert atomar die Sequenznummer `seq`.
    /// 3. Aktualisiert die `data`-Map (`Put` fügt ein, `Delete` entfernt).
    /// 4. Aktualisiert `last_tx`.
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

    /// Verwirft alle unbestätigten Operationen einer Transaktion.
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staged = self.staged.write();
            staged.remove(&tx_id);
            Ok(())
        })
    }

    /// Vereinfachtes Rollback aller Staging-Puffer bis zu einer bestimmten Transaktion.
    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            // Simplified in-memory rollback: reset staging
            self.staged.write().clear();
            Ok(())
        })
    }

    /// Flush-Operation: Im reinen In-Memory-Speicher eine No-Op.
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    /// Berechnet In-Memory-Speicherstatistiken (Gesamtgröße aller Schlüssel und Werte).
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

    /// Liefert die zuletzt vergebene Sequenznummer.
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(self.seq.load(Ordering::SeqCst)) })
    }

    /// Liefert die ID der zuletzt bestätigten Transaktion.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(*self.last_tx.read()) })
    }

    /// Fixiert einen Checkpoint (im Speicher eine No-Op).
    fn pin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    /// Gibt einen fixierten Checkpoint wieder frei (No-Op).
    fn unpin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    /// Führt einen Präfix-Scan über die aktuellsten Daten durch.
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

    /// Führt einen Präfix-Scan mit Zeiteinschränkung `seq_no` durch.
    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.read();
            let mut results = Vec::new();
            for (key, (val, item_seq)) in data.range(prefix.to_vec()..) {
                if key.starts_with(prefix) {
                    if *item_seq <= seq_no {
                        results.push((key.clone(), val.clone()));
                    }
                } else {
                    break;
                }
            }
            Ok(results)
        })
    }

    /// Scannt einen zusammenhängenden Schlüsselbereich `[start, end]` mit optionalem Limit.
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

            if limit == Some(0) {
                return Ok(Vec::new());
            }

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

    // ANCHOR[contextra-testkit:TEST-IN-MEM-STORE-COMMIT-001]
    // Prüft, dass gestagete Werte erst nach dem Commit sichtbar werden.
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

    // ANCHOR[contextra-testkit:TEST-IN-MEM-STORE-ROLLBACK-001]
    // Prüft, dass ein Rollback unbestätigte Änderungen verwirft und diese nicht sichtbar werden.
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
