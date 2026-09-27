//! Serializable Snapshot Isolation (SSI) read-set tracking and conflict validation.
//!
//! # Architecture Role (Ring 0)
//!
//! Provides in-memory SSI conflict detection for transactions:
//! - [`ReadSet`]: Tracks read keys and their snapshot sequence numbers per transaction.
//! - [`SsiValidator`]: Trait for checking concurrent write conflicts against a transaction's `ReadSet`.
//! - [`SequenceLogSsiValidator`]: Reference validator implementation using MVCC sequence tracking.

// FILE-CONTEXT
// STAND: 2026-09-27T00:00:00Z
// ZWECK: In-memory SSI Read-Set Tracking und Write-Skew-Konfliktvalidierung.
// INVARIANTEN: INV-MVCC-SSI-1: Konfliktauflösung ist rein datengetrieben (Sequenznummernvergleich) und deterministisch.
// HOTSPOTS: 35-120
// NICHT-OFFENSICHTLICH: Speicherverbrauch pro Transaktion wächst linear O(R) mit der Anzahl gelesener Schlüssel.
// SIEHE AUCH: crates/contextra-mvcc/src/seq_log.rs, crates/contextra-mvcc/src/tx_buffer.rs

use crate::error::{ContextraError, Result};
use crate::seq_log::SequenceLog;
use crate::types::{DocId, TxId};
use ahash::AHashMap;
use parking_lot::RwLock;
use std::hash::Hasher;
use std::sync::Arc;

/// Tracked read keys and their snapshot sequence numbers for Serializable Snapshot Isolation (SSI).
///
/// # Memory Overhead & Risk Analysis
/// Memory consumption per transaction grows linearly $O(R)$ with the number of read keys $R$,
/// where each entry retains a key byte vector (`Vec<u8>`) and an 8-byte (`u64`) snapshot sequence number.
/// High read-volume transactions should limit or monitor read-set size to avoid unbounded memory growth.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadSet {
    keys: AHashMap<Vec<u8>, u64>,
}

impl ReadSet {
    /// Creates a new empty [`ReadSet`].
    pub fn new() -> Self {
        Self {
            keys: AHashMap::default(),
        }
    }

    /// Records a read operation for `key` at snapshot sequence `snapshot_seq`.
    ///
    /// If the key was previously recorded in the same transaction, the lowest (earliest)
    /// snapshot sequence number is preserved to enforce strict isolation boundaries.
    pub fn record_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let k = key.into();
        self.keys
            .entry(k)
            .and_modify(|existing_seq| {
                if snapshot_seq < *existing_seq {
                    *existing_seq = snapshot_seq;
                }
            })
            .or_insert(snapshot_seq);
    }

    /// Alias for [`ReadSet::record_read`].
    pub fn register_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        self.record_read(key, snapshot_seq);
    }

    /// Returns the snapshot sequence number recorded for `key`, if present.
    pub fn get(&self, key: &[u8]) -> Option<u64> {
        self.keys.get(key).copied()
    }

    /// Returns `true` if no read keys are tracked in this set.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Returns the number of read keys tracked in this set.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Clears all tracked read keys.
    pub fn clear(&mut self) {
        self.keys.clear();
    }

    /// Returns an iterator over tracked read keys and their snapshot sequence numbers.
    pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)> {
        self.keys.iter()
    }

    /// Returns the inner map of read keys to snapshot sequence numbers.
    pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64> {
        &self.keys
    }
}

/// Trait for Serializable Snapshot Isolation (SSI) validation.
///
/// Implementations evaluate whether concurrent committed write transactions have mutated
/// any keys in the transaction's [`ReadSet`] after the snapshot sequence numbers recorded
/// when those keys were read.
pub trait SsiValidator {
    /// Validates the [`ReadSet`] for transaction `tx_id`.
    ///
    /// # Errors
    /// Returns `Err(ContextraError::Conflict(...))` if a key in `read_set` was committed by a
    /// concurrent transaction at a sequence number strictly greater than the key's snapshot sequence number.
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()>;
}

/// Reference implementation of [`SsiValidator`] backed by MVCC [`SequenceLog`] and committed key tracking.
///
/// Operates strictly in-memory without disk I/O or external storage dependencies.
///
/// # INVARIANT (INV-MVCC-SSI-1)
/// Conflict resolution is strictly data-driven (sequence number comparison) and deterministic.
/// Given identical commit sequences and read-sets, repeated executions yield identical outcomes.
#[derive(Debug, Clone, Default)]
pub struct SequenceLogSsiValidator {
    committed_writes: Arc<RwLock<AHashMap<Vec<u8>, u64>>>,
    sequence_log: Option<Arc<RwLock<SequenceLog>>>,
}

impl SequenceLogSsiValidator {
    /// Creates a new [`SequenceLogSsiValidator`] with empty committed write tracking.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`SequenceLogSsiValidator`] attached to an existing [`SequenceLog`].
    pub fn with_sequence_log(seq_log: Arc<RwLock<SequenceLog>>) -> Self {
        Self {
            committed_writes: Arc::new(RwLock::new(AHashMap::default())),
            sequence_log: Some(seq_log),
        }
    }

    /// Records a committed write for `key` at sequence number `commit_seq`.
    pub fn record_commit_key(&self, key: &[u8], commit_seq: u64) {
        let mut writes = self.committed_writes.write();
        writes
            .entry(key.to_vec())
            .and_modify(|existing| {
                if commit_seq > *existing {
                    *existing = commit_seq;
                }
            })
            .or_insert(commit_seq);
    }

    /// Records committed writes for multiple keys at sequence number `commit_seq`.
    pub fn record_commit_keys<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a [u8]>,
        commit_seq: u64,
    ) {
        let mut writes = self.committed_writes.write();
        for key in keys {
            writes
                .entry(key.to_vec())
                .and_modify(|existing| {
                    if commit_seq > *existing {
                        *existing = commit_seq;
                    }
                })
                .or_insert(commit_seq);
        }
    }
}

impl SsiValidator for SequenceLogSsiValidator {
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()> {
        if read_set.is_empty() {
            return Ok(());
        }

        let committed_writes = self.committed_writes.read();

        // Sort read keys deterministically by key bytes to guarantee reproducible validation order
        let mut entries: Vec<(&Vec<u8>, &u64)> = read_set.iter().collect();
        entries.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));

        for (key, &snapshot_seq) in entries {
            // 1. Direct key commit sequence check
            if let Some(&commit_seq) = committed_writes.get(key) {
                if commit_seq > snapshot_seq {
                    return Err(ContextraError::Conflict(format!(
                        "Serializable isolation violation for TxId({}): key '{:?}' modified at commit_seq {} > snapshot_seq {}",
                        tx_id.inner(),
                        String::from_utf8_lossy(key),
                        commit_seq,
                        snapshot_seq
                    )));
                }
            }

            // 2. SequenceLog entry check if attached
            if let Some(ref seq_log_lock) = self.sequence_log {
                let seq_log = seq_log_lock.read();
                let doc_id = derive_doc_id_for_key(key);

                let changes = seq_log.changes_since(snapshot_seq);
                for change in changes {
                    if change.doc_id() == doc_id && change.seq() > snapshot_seq {
                        return Err(ContextraError::Conflict(format!(
                            "Serializable isolation violation for TxId({}): doc_id {} modified at seq {} > snapshot_seq {}",
                            tx_id.inner(),
                            doc_id,
                            change.seq(),
                            snapshot_seq
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

fn derive_doc_id_for_key(key: &[u8]) -> DocId {
    if let Ok(s) = std::str::from_utf8(key) {
        if let Ok(doc_id) = DocId::from_key(s) {
            return doc_id;
        }
    }
    let mut hasher = ahash::AHasher::default();
    hasher.write(key);
    DocId::from(hasher.finish())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_read_set_basic() {
        let mut rs = ReadSet::new();
        assert!(rs.is_empty());
        assert_eq!(rs.len(), 0);

        rs.record_read(b"key1".to_vec(), 10);
        rs.record_read(b"key2".to_vec(), 15);

        assert!(!rs.is_empty());
        assert_eq!(rs.len(), 2);
        assert_eq!(rs.get(b"key1"), Some(10));
        assert_eq!(rs.get(b"key2"), Some(15));
        assert_eq!(rs.get(b"key3"), None);

        // Lower snapshot sequence replaces higher
        rs.record_read(b"key1".to_vec(), 5);
        assert_eq!(rs.get(b"key1"), Some(5));

        // Higher snapshot sequence ignored
        rs.record_read(b"key1".to_vec(), 20);
        assert_eq!(rs.get(b"key1"), Some(5));

        rs.clear();
        assert!(rs.is_empty());
    }

    #[test]
    fn test_validator_no_conflict() {
        let validator = SequenceLogSsiValidator::new();
        let tx = TxId::new(1);

        let mut rs = ReadSet::new();
        rs.record_read(b"key1".to_vec(), 10);

        // Record a commit for key1 at seq 10 (equal to snapshot_seq)
        validator.record_commit_key(b"key1", 10);

        assert!(validator.validate(tx, &rs).is_ok());
    }

    #[test]
    fn test_validator_conflict_detected() {
        let validator = SequenceLogSsiValidator::new();
        let tx = TxId::new(2);

        let mut rs = ReadSet::new();
        rs.record_read(b"key1".to_vec(), 10);

        // Record a commit for key1 at seq 11 (> snapshot_seq 10)
        validator.record_commit_key(b"key1", 11);

        let res = validator.validate(tx, &rs);
        assert!(res.is_err());
        assert!(matches!(res, Err(ContextraError::Conflict(_))));
    }

    #[test]
    fn test_validator_with_sequence_log() {
        let seq_log = Arc::new(RwLock::new(SequenceLog::new()));
        let validator = SequenceLogSsiValidator::with_sequence_log(seq_log.clone());
        let tx = TxId::new(3);

        let doc_key = "doc_alpha";
        let doc_id = DocId::from_key(doc_key).expect("valid doc_id");

        let mut rs = ReadSet::new();
        rs.record_read(doc_key.as_bytes(), 5);

        // No changes in sequence log -> OK
        assert!(validator.validate(tx, &rs).is_ok());

        // Record insert in sequence log at seq 8 (> snapshot_seq 5)
        seq_log.write().record_insert(doc_id, 8);

        let res = validator.validate(tx, &rs);
        assert!(res.is_err());
        assert!(matches!(res, Err(ContextraError::Conflict(_))));
    }
}
