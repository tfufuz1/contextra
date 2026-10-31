//! # MVCC-Referenzmodell für deterministisches Property-Testing (`reference_model`)
//!
//! Dieses Modul stellt mit [`ReferenceModel`] eine rein speicherbasierte, minimale
//! und formale Referenzimplementierung für Multi-Version Concurrency Control (MVCC) bereit.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Zweck:** Dient als mathematisch einfaches Orakel für differenzielles Testen
//!   und zustandsbasiertes Property-Testing gegen komplexe Speicher-Engines (z. B. `contextra-store`).
//! - **Frei von Nebenwirkungen:** Keine Zeit-, Zufalls- oder Dateisystemabhängigkeiten
//!   (reine Standardbibliothek `std::collections::BTreeMap`).
//!
//! ## Funktionsweise
//! Für jeden Schlüssel wird eine chronologische Versionshistorie gespeichert:
//! `(commit_seq, Option<value>)`.
//! - `Some(value)`: Ein aktiver Wert (Put).
//! - `None`: Ein Löschmarker (Tombstone).
//! Bei einer Leseoperation mit Sequenznummer `seq` wird die Historie rückwärts
//! durchsucht, bis der jüngste Eintrag mit `version_seq <= seq` gefunden wird.

// FILE-CONTEXT: Reference MVCC store model for deterministic property testing.
// STAND: 2026-09-28
// INVARIANTEN: Rein std, Ring-konform, deterministisch, zero time/rand dependencies.

use std::collections::BTreeMap;
use std::ops::Bound;

// ANCHOR[contextra-testkit:REF-OP-ENUM-001]
/// Operation for batch execution on the reference model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefOp {
    /// Inserts or overwrites a key-value pair.
    Put { key: Vec<u8>, value: Vec<u8> },
    /// Marks a key as deleted (tombstone).
    Delete { key: Vec<u8> },
}

// ANCHOR[contextra-testkit:REFERENCE-MODEL-STRUCT-001]
/// Deterministic, std-only MVCC reference model for Contextra store verification.
///
/// **Zweck:** Bietet eine überprüfbare "Ground Truth" für Transaktionsisolation,
/// Snapshot-Isolation und Rollbacks.
///
/// **Ablauf von Schreibvorgängen:**
/// 1. Operationen (`Put`, `Delete`) werden zunächst im unbestätigten Puffer `pending_writes` gesammelt.
/// 2. `rollback()` verwirft alle Puffer-Einträge ohne Erhöhung der Sequenznummer.
/// 3. `commit()` erhöht `current_seq` atomar um 1 und überträgt alle Puffer-Einträge
///    in die `history`-Map.
#[derive(Debug, Clone, Default)]
pub struct ReferenceModel {
    /// Versionshistorie: Schlüssel -> Liste von (Sequenznummer, Optionaler Wert).
    #[allow(clippy::type_complexity)]
    history: BTreeMap<Vec<u8>, Vec<(u64, Option<Vec<u8>>)>>,
    /// Unbestätigte Staging-Operationen vor dem Commit.
    pending_writes: Vec<RefOp>,
    /// Aktuelle High-Water-Mark-Sequenznummer (Beginnt bei 0).
    current_seq: u64,
}

impl ReferenceModel {
    // ANCHOR[contextra-testkit:REF-MODEL-NEW-001]
    /// Creates a new empty reference model initialized at sequence number 0.
    pub fn new() -> Self {
        Self {
            history: BTreeMap::new(),
            pending_writes: Vec::new(),
            current_seq: 0,
        }
    }

    // ANCHOR[contextra-testkit:REF-MODEL-PUT-001]
    /// Stages a `Put` operation in uncommitted pending buffer.
    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        self.pending_writes.push(RefOp::Put {
            key: key.to_vec(),
            value: value.to_vec(),
        });
    }

    // ANCHOR[contextra-testkit:REF-MODEL-DELETE-001]
    /// Stages a `Delete` operation in uncommitted pending buffer.
    pub fn delete(&mut self, key: &[u8]) {
        self.pending_writes
            .push(RefOp::Delete { key: key.to_vec() });
    }

    // ANCHOR[contextra-testkit:REF-MODEL-ROLLBACK-001]
    /// Clears any uncommitted pending writes without advancing sequence number.
    pub fn rollback(&mut self) {
        self.pending_writes.clear();
    }

    // ANCHOR[contextra-testkit:REF-MODEL-COMMIT-001]
    /// Commits all pending operations under a new sequence number.
    ///
    /// Returns the sequence number assigned to this commit batch.
    ///
    /// **Ablauf:**
    /// 1. Ist der Puffer leer, bleibt die Sequenznummer unverändert.
    /// 2. `current_seq` wird um 1 erhöht.
    /// 3. Alle Puffer-Operationen werden entnommen (`drain`) und an die Historie
    ///    des jeweiligen Schlüssels angehängt.
    pub fn commit(&mut self) -> u64 {
        if self.pending_writes.is_empty() {
            return self.current_seq;
        }

        self.current_seq += 1;
        let commit_seq = self.current_seq;

        for op in self.pending_writes.drain(..) {
            match op {
                RefOp::Put { key, value } => {
                    self.history
                        .entry(key)
                        .or_default()
                        .push((commit_seq, Some(value)));
                }
                RefOp::Delete { key } => {
                    self.history
                        .entry(key)
                        .or_default()
                        .push((commit_seq, None));
                }
            }
        }

        commit_seq
    }

    // ANCHOR[contextra-testkit:REF-MODEL-COMMIT-BATCH-001]
    /// Appends `ops` to pending buffer and commits them under a new sequence number.
    pub fn commit_batch(&mut self, ops: Vec<RefOp>) -> u64 {
        self.pending_writes.extend(ops);
        self.commit()
    }

    // ANCHOR[contextra-testkit:REF-MODEL-SNAPSHOT-SEQ-001]
    /// Returns the current high water mark sequence number.
    pub fn snapshot_seq(&self) -> u64 {
        self.current_seq
    }

    // ANCHOR[contextra-testkit:REF-MODEL-GET-AT-001]
    /// Retrieves the value of `key` as visible at `seq`.
    ///
    /// Returns `Some(value)` if key exists and was active (not deleted) at `seq`, `None` otherwise.
    ///
    /// **Ablauf:**
    /// 1. Sucht die Versionsliste zum Schlüssel in `history`.
    /// 2. Iteriert rückwärts (`rev`), um die jüngste Version mit `version_seq <= seq` zu finden.
    /// 3. Gibt `Some(bytes)` zurück, falls es ein aktiver Wert war, oder `None` bei Löschung/Nichtexistenz.
    pub fn get_at(&self, key: &[u8], seq: u64) -> Option<Vec<u8>> {
        let versions = self.history.get(key)?;
        // Find latest version with version_seq <= seq
        let latest = versions
            .iter()
            .rev()
            .find(|(version_seq, _)| *version_seq <= seq)?;

        latest.1.clone()
    }

    // ANCHOR[contextra-testkit:REF-MODEL-SCAN-PREFIX-AT-001]
    /// Scans keys starting with `prefix` and returns key-value pairs visible at `seq`.
    ///
    /// **Ablauf:**
    /// Nutzt `BTreeMap::range` ab dem Präfix und filtert gelöschte Einträge sowie
    /// Versionen mit `version_seq > seq` heraus.
    pub fn scan_prefix_at(&self, prefix: &[u8], seq: u64) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut results = Vec::new();
        for (key, versions) in self.history.range(prefix.to_vec()..) {
            if !key.starts_with(prefix) {
                break;
            }
            if let Some((_, Some(value))) = versions
                .iter()
                .rev()
                .find(|(version_seq, _)| *version_seq <= seq)
            {
                results.push((key.clone(), value.clone()));
            }
        }
        results
    }

    // ANCHOR[contextra-testkit:REF-MODEL-SCAN-RANGE-AT-001]
    /// Scans key range `[start, end]` (using `Bound`) and returns visible pairs at `seq`.
    pub fn scan_range_at(
        &self,
        start: Bound<&[u8]>,
        end: Bound<&[u8]>,
        seq: u64,
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        let map_bound = |b: Bound<&[u8]>| match b {
            Bound::Included(k) => Bound::Included(k.to_vec()),
            Bound::Excluded(k) => Bound::Excluded(k.to_vec()),
            Bound::Unbounded => Bound::Unbounded,
        };

        let mut results = Vec::new();
        for (key, versions) in self.history.range((map_bound(start), map_bound(end))) {
            if let Some((_, Some(value))) = versions
                .iter()
                .rev()
                .find(|(version_seq, _)| *version_seq <= seq)
            {
                results.push((key.clone(), value.clone()));
            }
        }
        results
    }

    // ANCHOR[contextra-testkit:REF-MODEL-GET-LATEST-001]
    /// Gets the latest committed value for `key`.
    pub fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get_at(key, self.current_seq)
    }

    // ANCHOR[contextra-testkit:REF-MODEL-SCAN-PREFIX-LATEST-001]
    /// Scans all entries starting with `prefix` at latest sequence number.
    pub fn scan_prefix_latest(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.scan_prefix_at(prefix, self.current_seq)
    }

    // ANCHOR[contextra-testkit:REF-MODEL-SNAPSHOT-MAP-AT-001]
    /// Returns a full snapshot map of latest visible key-values at `seq`.
    pub fn snapshot_map_at(&self, seq: u64) -> BTreeMap<Vec<u8>, Vec<u8>> {
        let mut map = BTreeMap::new();
        for (key, versions) in &self.history {
            if let Some((_, Some(val))) = versions
                .iter()
                .rev()
                .find(|(version_seq, _)| *version_seq <= seq)
            {
                map.insert(key.clone(), val.clone());
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-testkit:TEST-REF-MODEL-ISOLATION-001]
    // Prüft MVCC-Snapshot-Isolation über mehrere Sequenznummern und Präfix-Scans.
    #[test]
    fn test_reference_model_mvcc_isolation() {
        let mut model = ReferenceModel::new();

        model.put(b"k1", b"v1_seq1");
        model.put(b"k2", b"v2_seq1");
        let seq1 = model.commit();

        model.put(b"k1", b"v1_seq2");
        model.delete(b"k2");
        let seq2 = model.commit();

        assert_eq!(model.get_at(b"k1", seq1), Some(b"v1_seq1".to_vec()));
        assert_eq!(model.get_at(b"k2", seq1), Some(b"v2_seq1".to_vec()));

        assert_eq!(model.get_at(b"k1", seq2), Some(b"v1_seq2".to_vec()));
        assert_eq!(model.get_at(b"k2", seq2), None);

        // Sequence 0 should see nothing
        assert_eq!(model.get_at(b"k1", 0), None);

        // Prefix scan isolation
        let p_seq1 = model.scan_prefix_at(b"k", seq1);
        assert_eq!(
            p_seq1,
            vec![
                (b"k1".to_vec(), b"v1_seq1".to_vec()),
                (b"k2".to_vec(), b"v2_seq1".to_vec())
            ]
        );

        let p_seq2 = model.scan_prefix_at(b"k", seq2);
        assert_eq!(p_seq2, vec![(b"k1".to_vec(), b"v1_seq2".to_vec())]);
    }
}
