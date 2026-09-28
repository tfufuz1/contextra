//! Serializable Snapshot Isolation (SSI) read-set tracking and conflict validation.
//!
//! # Architecture Role (Ring 0)
//!
//! Provides in-memory SSI conflict detection for transactions:
//! - [`ReadSet`]: Tracks read keys, range prefixes, and their snapshot sequence numbers per transaction.
//! - [`SsiValidator`]: Trait for checking concurrent write conflicts against a transaction's `ReadSet`.
//! - [`SequenceLogSsiValidator`]: Reference validator implementation using MVCC sequence tracking.

// FILE-CONTEXT
// STAND: 2026-09-28T00:00:00Z
// ZWECK: In-memory SSI Read-Set Tracking, Write-Skew-Konfliktvalidierung, Fail-Closed Pruning und atomare Commit-Registrierung.
// INVARIANTEN: INV-MVCC-SSI-1: Konfliktauflösung ist rein datengetrieben (Sequenznummernvergleich) und deterministisch.
// HOTSPOTS: 35-220
// NICHT-OFFENSICHTLICH: Pruning unter `committed_writes` Schreib-Lock garantiert atomaren Wasserstand. Fail-closed Protection verweigert Reads mit snapshot_seq < pruned_through.
// SIEHE AUCH: crates/contextra-mvcc/src/seq_log.rs, crates/contextra-mvcc/src/tx_buffer.rs

use crate::error::{ContextraError, Result};
use crate::seq_log::SequenceLog;
use crate::types::{DocId, TxId};
use ahash::AHashMap;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::hash::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Default maximum tracked committed write keys in [`SequenceLogSsiValidator`] (1,000,000 keys).
pub const DEFAULT_MAX_TRACKED_COMMIT_KEYS: usize = 1_000_000;

/// Tracked read keys, range prefixes, and their snapshot sequence numbers for Serializable Snapshot Isolation (SSI).
///
/// # Memory Overhead & Risk Analysis
/// Memory consumption per transaction grows linearly $O(R + P)$ with the number of read keys $R$ and range prefixes $P$,
/// where each entry retains a key/prefix byte vector (`Vec<u8>`) and an 8-byte (`u64`) snapshot sequence number.
/// High read-volume transactions should limit or monitor read-set size to avoid unbounded memory growth.
/// Use [`crate::tx_buffer::try_register_read`] with [`crate::tx_buffer::DEFAULT_MAX_READ_SET_KEYS`] to enforce bounded limits.
///
/// # Range Reads & Phantom Protection
/// SSI protects point reads directly via recorded keys. Range scans (e.g. prefix scans) MUST call
/// [`ReadSet::record_prefix`] to enable phantom protection against concurrent key insertions matching the prefix.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadSet {
    keys: AHashMap<Vec<u8>, u64>,
    prefixes: AHashMap<Vec<u8>, u64>,
    min_snapshot_seq: Option<u64>,
}

impl ReadSet {
    /// Creates a new empty [`ReadSet`].
    pub fn new() -> Self {
        Self {
            keys: AHashMap::default(),
            prefixes: AHashMap::default(),
            min_snapshot_seq: None,
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

        self.min_snapshot_seq = Some(
            self.min_snapshot_seq
                .map_or(snapshot_seq, |curr| curr.min(snapshot_seq)),
        );
    }

    /// Records a range prefix read operation at `snapshot_seq` for phantom protection.
    ///
    /// SSI protects point reads directly; range scans are protected against phantoms
    /// only when `record_prefix` is explicitly called.
    pub fn record_prefix(&mut self, prefix: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let p = prefix.into();
        self.prefixes
            .entry(p)
            .and_modify(|existing_seq| {
                if snapshot_seq < *existing_seq {
                    *existing_seq = snapshot_seq;
                }
            })
            .or_insert(snapshot_seq);

        self.min_snapshot_seq = Some(
            self.min_snapshot_seq
                .map_or(snapshot_seq, |curr| curr.min(snapshot_seq)),
        );
    }

    /// Alias for [`ReadSet::record_read`].
    pub fn register_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        self.record_read(key, snapshot_seq);
    }

    /// Returns the snapshot sequence number recorded for `key`, if present.
    pub fn get(&self, key: &[u8]) -> Option<u64> {
        self.keys.get(key).copied()
    }

    /// Returns `true` if no read keys or range prefixes are tracked in this set.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty() && self.prefixes.is_empty()
    }

    /// Returns the total number of read keys and range prefixes tracked in this set.
    pub fn len(&self) -> usize {
        self.keys.len() + self.prefixes.len()
    }

    /// Clears all tracked read keys, range prefixes, and resets the cached minimum snapshot sequence.
    pub fn clear(&mut self) {
        self.keys.clear();
        self.prefixes.clear();
        self.min_snapshot_seq = None;
    }

    /// Returns the lowest snapshot sequence number recorded across all read keys and prefixes in this set,
    /// or `None` if the set is empty.
    pub fn min_snapshot_seq(&self) -> Option<u64> {
        self.min_snapshot_seq
    }

    /// Returns an iterator over tracked read keys and their snapshot sequence numbers.
    pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)> {
        self.keys.iter()
    }

    /// Returns an iterator over tracked range prefixes and their snapshot sequence numbers.
    pub fn prefixes_iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)> {
        self.prefixes.iter()
    }

    /// Returns the inner map of read keys to snapshot sequence numbers.
    pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64> {
        &self.keys
    }

    /// Returns the inner map of range prefixes to snapshot sequence numbers.
    pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64> {
        &self.prefixes
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

#[derive(Debug, Default)]
struct CommittedWrites {
    keys: AHashMap<Vec<u8>, u64>,
    seq_index: BTreeMap<u64, Vec<Vec<u8>>>,
}

impl CommittedWrites {
    fn insert(&mut self, key: Vec<u8>, commit_seq: u64) {
        if let Some(old_seq) = self.keys.insert(key.clone(), commit_seq) {
            if old_seq != commit_seq {
                if let Some(keys) = self.seq_index.get_mut(&old_seq) {
                    keys.retain(|k| k != &key);
                    if keys.is_empty() {
                        self.seq_index.remove(&old_seq);
                    }
                }
            }
        }
        self.seq_index.entry(commit_seq).or_default().push(key);
    }

    fn remove_from_seq(&mut self, first_seq: u64) -> usize {
        let mut removed = 0;
        let mut seqs_to_remove = Vec::new();

        for (&seq, keys) in self.seq_index.range(first_seq..) {
            seqs_to_remove.push(seq);
            for key in keys {
                if self.keys.remove(key).is_some() {
                    removed += 1;
                }
            }
        }

        for seq in seqs_to_remove {
            self.seq_index.remove(&seq);
        }

        removed
    }

    fn prune_through(&mut self, bound_seq: u64) -> usize {
        let mut removed = 0;
        let mut seqs_to_remove = Vec::new();

        for (&seq, keys) in self.seq_index.range(..=bound_seq) {
            seqs_to_remove.push(seq);
            for key in keys {
                if self.keys.remove(key).is_some() {
                    removed += 1;
                }
            }
        }

        for seq in seqs_to_remove {
            self.seq_index.remove(&seq);
        }

        removed
    }

    fn len(&self) -> usize {
        self.keys.len()
    }

    fn get(&self, key: &[u8]) -> Option<u64> {
        self.keys.get(key).copied()
    }
}

/// Reference implementation of [`SsiValidator`] backed by MVCC [`SequenceLog`] and committed key tracking.
///
/// Operates strictly in-memory without disk I/O or external storage dependencies.
///
/// # INVARIANT (INV-MVCC-SSI-1)
/// Conflict resolution is strictly data-driven (sequence number comparison) and deterministic.
/// Given identical commit sequences and read-sets, repeated executions yield identical outcomes.
#[derive(Debug, Clone)]
pub struct SequenceLogSsiValidator {
    committed_writes: Arc<RwLock<CommittedWrites>>,
    sequence_log: Option<Arc<RwLock<SequenceLog>>>,
    pruned_through: Arc<AtomicU64>,
    max_tracked_keys: usize,
}

impl Default for SequenceLogSsiValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SequenceLogSsiValidator {
    /// Creates a new [`SequenceLogSsiValidator`] with default maximum commit key capacity (1,000,000 keys).
    pub fn new() -> Self {
        Self::new_with_bounds(DEFAULT_MAX_TRACKED_COMMIT_KEYS)
    }

    /// Creates a new [`SequenceLogSsiValidator`] with specified maximum commit key bounds `max_tracked_keys`.
    pub fn new_with_bounds(max_tracked_keys: usize) -> Self {
        Self {
            committed_writes: Arc::new(RwLock::new(CommittedWrites::default())),
            sequence_log: None,
            pruned_through: Arc::new(AtomicU64::new(0)),
            max_tracked_keys,
        }
    }

    /// Creates a new [`SequenceLogSsiValidator`] attached to an existing [`SequenceLog`].
    pub fn with_sequence_log(seq_log: Arc<RwLock<SequenceLog>>) -> Self {
        Self {
            committed_writes: Arc::new(RwLock::new(CommittedWrites::default())),
            sequence_log: Some(seq_log),
            pruned_through: Arc::new(AtomicU64::new(0)),
            max_tracked_keys: DEFAULT_MAX_TRACKED_COMMIT_KEYS,
        }
    }

    /// Prunes tracked committed writes with `commit_seq <= bound_seq`.
    ///
    /// Atomically updates `pruned_through` watermark monotonically under the `committed_writes` write lock.
    /// Returns the number of entries removed.
    pub fn prune_through(&self, bound_seq: u64) -> usize {
        let mut writes = self.committed_writes.write();
        let removed = writes.prune_through(bound_seq);
        self.pruned_through.fetch_max(bound_seq, Ordering::SeqCst);
        removed
    }

    /// Removes all committed key registrations with `commit_seq >= first_seq`.
    ///
    /// Used when WAL append or commit execution fails after keys were registered conservatively.
    /// Returns the number of entries removed in $O(K_{\text{removed}})$ time using the sequence index.
    pub fn forget_from(&self, first_seq: u64) -> usize {
        let mut writes = self.committed_writes.write();
        writes.remove_from_seq(first_seq)
    }

    /// Returns the current `pruned_through` watermark sequence number.
    pub fn pruned_through(&self) -> u64 {
        self.pruned_through.load(Ordering::SeqCst)
    }

    /// Returns the number of committed keys currently tracked.
    pub fn tracked_commit_keys(&self) -> usize {
        self.committed_writes.read().len()
    }

    /// Returns the configured maximum commit key tracking limit.
    pub fn max_tracked_keys(&self) -> usize {
        self.max_tracked_keys
    }

    /// Records a committed write for `key` at sequence number `commit_seq`.
    pub fn record_commit_key(&self, key: &[u8], commit_seq: u64) {
        let mut writes = self.committed_writes.write();
        writes.insert(key.to_vec(), commit_seq);
    }

    /// Records committed writes for multiple keys at sequence number `commit_seq`.
    pub fn record_commit_keys<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a [u8]>,
        commit_seq: u64,
    ) {
        let mut writes = self.committed_writes.write();
        for key in keys {
            writes.insert(key.to_vec(), commit_seq);
        }
    }

    /// Validates `read_set` against `committed_writes` map and `sequence_log`.
    fn validate_internal(
        &self,
        committed_writes: &CommittedWrites,
        tx_id: TxId,
        read_set: &ReadSet,
    ) -> Result<()> {
        if read_set.is_empty() {
            return Ok(());
        }

        // Fail-closed check if commit key capacity limit is reached or exceeded
        if committed_writes.len() >= self.max_tracked_keys {
            return Err(ContextraError::Conflict(format!(
                "Serializable isolation validation failed for TxId({}): commit key capacity limit reached ({} >= {})",
                tx_id.inner(),
                committed_writes.len(),
                self.max_tracked_keys
            )));
        }

        let pruned_through = self.pruned_through.load(Ordering::SeqCst);

        // Sort read keys deterministically by key bytes to guarantee reproducible validation order
        let mut entries: Vec<(&Vec<u8>, &u64)> = read_set.iter().collect();
        entries.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));

        for (key, &snapshot_seq) in entries {
            if snapshot_seq < pruned_through {
                return Err(ContextraError::Conflict(format!(
                    "Serializable isolation violation for TxId({}): snapshot too old (snapshot_seq {} < pruned_through {}) for key '{:?}'",
                    tx_id.inner(),
                    snapshot_seq,
                    pruned_through,
                    String::from_utf8_lossy(key)
                )));
            }

            // 1. Direct key commit sequence check
            if let Some(commit_seq) = committed_writes.get(key) {
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

        // Range scan phantom protection checks
        let mut prefixes: Vec<(&Vec<u8>, &u64)> = read_set.prefixes_iter().collect();
        prefixes.sort_unstable_by(|(p1, _), (p2, _)| p1.cmp(p2));

        for (prefix, &snapshot_seq) in prefixes {
            if snapshot_seq < pruned_through {
                return Err(ContextraError::Conflict(format!(
                    "Serializable isolation violation for TxId({}): snapshot too old (snapshot_seq {} < pruned_through {}) for range prefix '{:?}'",
                    tx_id.inner(),
                    snapshot_seq,
                    pruned_through,
                    String::from_utf8_lossy(prefix)
                )));
            }

            for (committed_key, &commit_seq) in &committed_writes.keys {
                if committed_key.starts_with(prefix) && commit_seq > snapshot_seq {
                    return Err(ContextraError::Conflict(format!(
                        "Serializable isolation phantom violation for TxId({}): key '{:?}' matching range prefix '{:?}' committed at commit_seq {} > snapshot_seq {}",
                        tx_id.inner(),
                        String::from_utf8_lossy(committed_key),
                        String::from_utf8_lossy(prefix),
                        commit_seq,
                        snapshot_seq
                    )));
                }
            }
        }

        Ok(())
    }

    /// Atomically validates `read_set` and records `write_keys` at `commit_seq` under a single write lock.
    pub fn validate_and_record<'a>(
        &self,
        tx_id: TxId,
        read_set: &ReadSet,
        write_keys: impl IntoIterator<Item = &'a [u8]>,
        commit_seq: u64,
    ) -> Result<()> {
        let mut writes = self.committed_writes.write();
        self.validate_internal(&writes, tx_id, read_set)?;

        for key in write_keys {
            writes.insert(key.to_vec(), commit_seq);
        }

        Ok(())
    }
}

impl SsiValidator for SequenceLogSsiValidator {
    fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()> {
        let committed_writes = self.committed_writes.read();
        self.validate_internal(&committed_writes, tx_id, read_set)
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
        assert_eq!(rs.min_snapshot_seq(), None);

        rs.record_read(b"key1".to_vec(), 10);
        rs.record_read(b"key2".to_vec(), 15);

        assert!(!rs.is_empty());
        assert_eq!(rs.len(), 2);
        assert_eq!(rs.get(b"key1"), Some(10));
        assert_eq!(rs.get(b"key2"), Some(15));
        assert_eq!(rs.get(b"key3"), None);
        assert_eq!(rs.min_snapshot_seq(), Some(10));

        // Lower snapshot sequence replaces higher
        rs.record_read(b"key1".to_vec(), 5);
        assert_eq!(rs.get(b"key1"), Some(5));
        assert_eq!(rs.min_snapshot_seq(), Some(5));

        // Higher snapshot sequence ignored for key1, min remains 5
        rs.record_read(b"key1".to_vec(), 20);
        assert_eq!(rs.get(b"key1"), Some(5));
        assert_eq!(rs.min_snapshot_seq(), Some(5));

        rs.clear();
        assert!(rs.is_empty());
        assert_eq!(rs.min_snapshot_seq(), None);
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
