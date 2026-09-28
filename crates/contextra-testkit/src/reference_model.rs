// FILE-CONTEXT
// STAND: 2026-09-28T20:00:00Z
// ZWECK: In-Memory Referenzmodell fuer Contextra Store MVCC Invarianten und Crash Recovery Verification.
// INVARIANTEN: Rein std, deterministic, keine Zeit/Zufall, zero panic/unwrap/expect.

use std::collections::BTreeMap;

type VersionEntry = (u64, Option<Vec<u8>>);

/// Eine atomare Schreiboperation im Referenzmodell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefOp {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}

/// In-Memory MVCC Referenzmodell fuer Contextra Store.
/// Speichert Key-Historien geordnet nach Sequence Number.
#[derive(Debug, Clone, Default)]
pub struct ReferenceModel {
    /// Mapping: Key -> Vec<(sequence_number, Option<Value>)>
    /// None repraesentiert ein Tombstone (Delete).
    store: BTreeMap<Vec<u8>, Vec<VersionEntry>>,
    /// Uncommitted Staging Buffer fuer die aktuelle Transaktion/Batch: Key -> Option<Value>
    staging: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    /// Aktuelle global hochzaehlende Commit Sequence Number
    current_seq: u64,
}

impl ReferenceModel {
    /// Erstellt ein neues, leeres Referenzmodell.
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
            staging: BTreeMap::new(),
            current_seq: 0,
        }
    }

    /// Fuegt ein Key-Value Pair in den Staging-Puffer ein.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.staging.insert(key, Some(value));
    }

    /// Fuegt einen Tombstone (Delete) in den Staging-Puffer ein.
    pub fn delete(&mut self, key: Vec<u8>) {
        self.staging.insert(key, None);
    }

    /// Verwirt alle uncommitted Staging-Aenderungen.
    pub fn rollback(&mut self) {
        self.staging.clear();
    }

    /// Comitted alle gestageten Operationen mit einer neuen Sequence Number.
    /// Gibt die neue Commit Sequence Number zurueck (oder `current_seq`, falls Staging leer war).
    pub fn commit(&mut self) -> u64 {
        if self.staging.is_empty() {
            return self.current_seq;
        }
        self.current_seq += 1;
        let seq = self.current_seq;

        let staging = std::mem::take(&mut self.staging);
        for (key, val_opt) in staging {
            self.store.entry(key).or_default().push((seq, val_opt));
        }

        seq
    }

    /// Fuehrt einen Batch von RefOps direkt als einen atomaren Commit aus.
    pub fn commit_batch(&mut self, ops: Vec<RefOp>) -> u64 {
        if ops.is_empty() {
            return self.current_seq;
        }
        self.current_seq += 1;
        let seq = self.current_seq;

        for op in ops {
            match op {
                RefOp::Put(k, v) => {
                    self.store.entry(k).or_default().push((seq, Some(v)));
                }
                RefOp::Delete(k) => {
                    self.store.entry(k).or_default().push((seq, None));
                }
            }
        }

        seq
    }

    /// Liest den Zustand eines Keys zum Zeitpunkt `seq` (inclusive).
    /// Liefert `Some(Vec<u8>)`, wenn der Key existiert und kein Tombstone war.
    /// Liefert `None`, falls der Key nicht existiert, zum Zeitpunkt `seq` geloescht war
    /// oder nach `seq` erst erstellt wurde.
    pub fn get_at(&self, key: &[u8], seq: u64) -> Option<Vec<u8>> {
        let history = self.store.get(key)?;
        // Find the latest version with sequence_number <= seq
        for (version_seq, val_opt) in history.iter().rev() {
            if *version_seq <= seq {
                return val_opt.clone();
            }
        }
        None
    }

    /// Gibt das neueste sichtbare Value fuer einen Key zurueck (aktuelle `current_seq`).
    pub fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get_at(key, self.current_seq)
    }

    /// Scannt alle Keys mit dem angegebenen Praefix zum Zeitpunkt `seq`.
    /// Liefert `Vec<(Key, Value)>` sortiert nach Key.
    pub fn scan_prefix_at(&self, prefix: &[u8], seq: u64) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut results = Vec::new();
        for (key, history) in self.store.range(prefix.to_vec()..) {
            if !key.starts_with(prefix) {
                break;
            }
            // Find latest version <= seq
            for (version_seq, val_opt) in history.iter().rev() {
                if *version_seq <= seq {
                    if let Some(ref val) = val_opt {
                        results.push((key.clone(), val.clone()));
                    }
                    break;
                }
            }
        }
        results
    }

    /// Scannt alle Keys im Bereich [start_key, end_key) zum Zeitpunkt `seq`.
    pub fn scan_range_at(
        &self,
        start_key: &[u8],
        end_key: &[u8],
        seq: u64,
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut results = Vec::new();
        for (key, history) in self.store.range(start_key.to_vec()..end_key.to_vec()) {
            for (version_seq, val_opt) in history.iter().rev() {
                if *version_seq <= seq {
                    if let Some(ref val) = val_opt {
                        results.push((key.clone(), val.clone()));
                    }
                    break;
                }
            }
        }
        results
    }

    /// Scannt den aktuellen Stand aller Keys mit dem angegebenen Praefix.
    pub fn scan_prefix_latest(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.scan_prefix_at(prefix, self.current_seq)
    }

    /// Liefert die aktuelle hoechste Commit Sequence Number.
    pub fn snapshot_seq(&self) -> u64 {
        self.current_seq
    }

    /// Liefert alle aktiven (nicht geloeschten) Keys zum Zeitpunkt `seq`.
    pub fn keys_at(&self, seq: u64) -> Vec<Vec<u8>> {
        let mut keys = Vec::new();
        for (key, history) in &self.store {
            for (version_seq, val_opt) in history.iter().rev() {
                if *version_seq <= seq {
                    if val_opt.is_some() {
                        keys.push(key.clone());
                    }
                    break;
                }
            }
        }
        keys
    }
}
