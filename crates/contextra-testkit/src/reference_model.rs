//! # ReferenceModel für deterministische MVCC- und Gate-Validierung
//!
//! Dieses Modul stellt mit [`ReferenceModel`] eine rein speicherbasierte, minimale
//! und formale Referenzimplementierung für Multi-Version Concurrency Control (MVCC),
//! Snapshot-Isolation, Tombstone-Verwaltung (`vlen == 0`), Gate-Sperren und SSI-Konfliktprüfung
//! bereit.
//!
//! ## Eigenschaften
//! - **Keine Sentinels:** `min_read_snapshot` gibt `Option<u64>` zurück (niemals `u64::MAX`).
//! - **Tombstone als vlen == 0:** Gelöschte Einträge werden als `vlen == 0` (leere Payload bzw. `None`) gespeichert.
//! - **Gate-Simulation:** Schlüssel oder Präfixe können ab einer Sequenznummer gegatet werden (sofortige Nicht-Sichtbarkeit).
//! - **SSI-Konflikt-Orakel:** Prüft, ob Transaktions-Lese-Sets durch spätere Commits ungültig wurden.
//! - **Keine Nebenwirkungen:** Reine Standardbibliothek (`std::collections::BTreeMap`), deterministisch und frei von Time-/Rand-Abhängigkeiten.

// FILE-CONTEXT: Reference MVCC store and gate model for deterministic property testing.
// STAND: 2026-11-02
// INVARIANTEN: Rein std, Ring-konform, deterministisch, zero time/rand dependencies, no sentinels, tombstone vlen=0.

use std::collections::BTreeMap;
use std::ops::Bound;

// ANCHOR[contextra-testkit:REF-OP-ENUM-001]
/// Operation for batch execution on the reference model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefOp {
    /// Inserts or overwrites a key-value pair.
    Put { key: Vec<u8>, value: Vec<u8> },
    /// Marks a key as deleted (tombstone with vlen 0).
    Delete { key: Vec<u8> },
}

// ANCHOR[contextra-testkit:REFERENCE-MODEL-STRUCT-001]
/// Deterministic, std-only MVCC reference model for Contextra store and gate verification.
#[derive(Debug, Clone, Default)]
pub struct ReferenceModel {
    /// Versionshistorie: Schlüssel -> Liste von (Sequenznummer, Optionaler Wert).
    /// `None` bzw. leere Payload signalisiert einen Tombstone (`vlen == 0`).
    #[allow(clippy::type_complexity)]
    history: BTreeMap<Vec<u8>, Vec<(u64, Option<Vec<u8>>)>>,
    /// Unbestätigte Staging-Operationen vor dem Commit.
    pending_writes: Vec<RefOp>,
    /// Aktuelle High-Water-Mark-Sequenznummer (Beginnt bei 0).
    current_seq: u64,
    /// Letzte Commit-Sequenznummer je Schlüssel (für SSI-Konfliktprüfung).
    latest_commit: BTreeMap<Vec<u8>, u64>,
    /// Aktive Snapshots mit Referenzzähler (ohne Sentinel-Werte).
    active_snapshots: BTreeMap<u64, usize>,
    /// Gekeyte Gates: Schlüssel -> Gate-Sequenznummer (ab dieser Seq ist der Key gesperrt).
    gated_keys: BTreeMap<Vec<u8>, u64>,
    /// Präfix-Gates: Präfix -> Gate-Sequenznummer.
    gated_prefixes: BTreeMap<Vec<u8>, u64>,
}

impl ReferenceModel {
    // ANCHOR[contextra-testkit:REF-MODEL-NEW-001]
    /// Creates a new empty reference model initialized at sequence number 0.
    pub fn new() -> Self {
        Self {
            history: BTreeMap::new(),
            pending_writes: Vec::new(),
            current_seq: 0,
            latest_commit: BTreeMap::new(),
            active_snapshots: BTreeMap::new(),
            gated_keys: BTreeMap::new(),
            gated_prefixes: BTreeMap::new(),
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

    /// Stages a tombstone operation (`vlen == 0`) in uncommitted pending buffer.
    pub fn put_tombstone(&mut self, key: &[u8]) {
        self.delete(key);
    }

    // ANCHOR[contextra-testkit:REF-MODEL-DELETE-001]
    /// Stages a `Delete` operation in uncommitted pending buffer (Tombstone mit vlen 0).
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
    pub fn commit(&mut self) -> u64 {
        if self.pending_writes.is_empty() {
            return self.current_seq;
        }

        self.current_seq += 1;
        let commit_seq = self.current_seq;

        for op in self.pending_writes.drain(..) {
            match op {
                RefOp::Put { key, value } => {
                    self.latest_commit.insert(key.clone(), commit_seq);
                    self.history
                        .entry(key)
                        .or_default()
                        .push((commit_seq, Some(value)));
                }
                RefOp::Delete { key } => {
                    self.latest_commit.insert(key.clone(), commit_seq);
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

    /// Opens an active snapshot at sequence number `seq`.
    pub fn open_snapshot(&mut self, seq: u64) {
        *self.active_snapshots.entry(seq).or_insert(0) += 1;
    }

    /// Closes an active snapshot at sequence number `seq`.
    pub fn close_snapshot(&mut self, seq: u64) {
        if let Some(count) = self.active_snapshots.get_mut(&seq) {
            if *count <= 1 {
                self.active_snapshots.remove(&seq);
            } else {
                *count -= 1;
            }
        }
    }

    /// Returns the lowest sequence number among all active snapshots,
    /// or `None` if no snapshots are currently active.
    ///
    /// Garantiert sentinel-frei (kein `u64::MAX`).
    pub fn min_read_snapshot(&self) -> Option<u64> {
        self.active_snapshots.keys().next().copied()
    }

    /// Gates a specific key starting at sequence number `gated_at`.
    pub fn gate_key(&mut self, key: &[u8], gated_at: u64) {
        self.gated_keys.insert(key.to_vec(), gated_at);
    }

    /// Gates an entire prefix starting at sequence number `gated_at`.
    pub fn gate_prefix(&mut self, prefix: &[u8], gated_at: u64) {
        self.gated_prefixes.insert(prefix.to_vec(), gated_at);
    }

    /// Checks whether `key` is gated at `seq`.
    pub fn is_gated_at(&self, key: &[u8], seq: u64) -> bool {
        if let Some(&gated_seq) = self.gated_keys.get(key) {
            if seq >= gated_seq {
                return true;
            }
        }
        for (prefix, &gated_seq) in &self.gated_prefixes {
            if key.starts_with(prefix) && seq >= gated_seq {
                return true;
            }
        }
        false
    }

    // ANCHOR[contextra-testkit:REF-MODEL-GET-AT-001]
    /// Retrieves the value of `key` as visible at `seq` without gate check.
    pub fn get_at(&self, key: &[u8], seq: u64) -> Option<Vec<u8>> {
        let versions = self.history.get(key)?;
        let latest = versions
            .iter()
            .rev()
            .find(|(version_seq, _)| *version_seq <= seq)?;

        latest.1.clone()
    }

    /// Retrieves the value of `key` at `seq` with gate validation.
    /// Returns `None` if the key is gated at `seq` or deleted/absent.
    pub fn get_at_with_gate(&self, key: &[u8], seq: u64) -> Option<Vec<u8>> {
        if self.is_gated_at(key, seq) {
            return None;
        }
        self.get_at(key, seq)
    }

    /// Checks if the latest version visible at `seq` is an explicit tombstone (`vlen == 0`).
    pub fn is_tombstone_at(&self, key: &[u8], seq: u64) -> bool {
        let Some(versions) = self.history.get(key) else {
            return false;
        };
        let Some(latest) = versions
            .iter()
            .rev()
            .find(|(version_seq, _)| *version_seq <= seq)
        else {
            return false;
        };

        latest.1.is_none()
    }

    /// SSI validation oracle: Checks whether any key in `reads` (key, read_snapshot_seq)
    /// was modified/committed after the respective snapshot sequence.
    pub fn would_conflict(&self, reads: &[(Vec<u8>, u64)]) -> bool {
        for (key, snapshot_seq) in reads {
            if let Some(&latest) = self.latest_commit.get(key) {
                if latest > *snapshot_seq {
                    return true;
                }
            }
        }
        false
    }

    // ANCHOR[contextra-testkit:REF-MODEL-SCAN-PREFIX-AT-001]
    /// Scans keys starting with `prefix` and returns key-value pairs visible at `seq`.
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

        assert_eq!(model.get_at(b"k1", 0), None);

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

    #[test]
    fn test_reference_model_sentinel_and_tombstones() {
        let mut model = ReferenceModel::new();
        assert_eq!(model.min_read_snapshot(), None);

        model.open_snapshot(10);
        model.open_snapshot(20);
        assert_eq!(model.min_read_snapshot(), Some(10));

        model.close_snapshot(10);
        assert_eq!(model.min_read_snapshot(), Some(20));

        model.close_snapshot(20);
        assert_eq!(model.min_read_snapshot(), None);
    }

    #[test]
    fn test_reference_model_property_self_consistency() {
        let mut model = ReferenceModel::new();
        let mut shadow: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();

        for i in 0..50u8 {
            let k = vec![i % 5];
            if i % 3 == 0 {
                model.put_tombstone(&k);
                shadow.insert(k, None);
            } else {
                let v = vec![i];
                model.put(&k, &v);
                shadow.insert(k, Some(v));
            }
            let seq = model.commit();

            for (sk, sv) in &shadow {
                assert_eq!(model.get_at(sk, seq), sv.clone());
                assert_eq!(model.is_tombstone_at(sk, seq), sv.is_none());
            }
        }
    }
}
