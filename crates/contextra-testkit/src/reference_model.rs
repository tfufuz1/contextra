// FILE-CONTEXT: Reference MVCC store model for deterministic property testing.
// STAND: 2026-10-12
// INVARIANTEN: Rein std, Ring-konform, deterministisch, zero time/rand dependencies.

use std::collections::BTreeMap;
use std::ops::Bound;

/// Operation for batch execution on the reference model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefOp {
    /// Inserts or overwrites a key-value pair.
    Put { key: Vec<u8>, value: Vec<u8> },
    /// Marks a key as deleted (tombstone).
    Delete { key: Vec<u8> },
    /// Permanently shred/forget a key and its history.
    Forget { key: Vec<u8> },
}

/// Deterministic, std-only MVCC reference model for Contextra store verification.
#[derive(Debug, Clone, Default)]
pub struct ReferenceModel {
    #[allow(clippy::type_complexity)]
    history: BTreeMap<Vec<u8>, Vec<(u64, Option<Vec<u8>>)>>,
    forgotten_at: BTreeMap<Vec<u8>, u64>,
    pending_writes: Vec<RefOp>,
    current_seq: u64,
    acked_seq: u64,
}

impl ReferenceModel {
    /// Creates a new empty reference model initialized at sequence number 0.
    pub fn new() -> Self {
        Self {
            history: BTreeMap::new(),
            forgotten_at: BTreeMap::new(),
            pending_writes: Vec::new(),
            current_seq: 0,
            acked_seq: 0,
        }
    }

    /// Stages a `Put` operation in uncommitted pending buffer.
    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        self.pending_writes.push(RefOp::Put {
            key: key.to_vec(),
            value: value.to_vec(),
        });
    }

    /// Stages a `Delete` operation in uncommitted pending buffer.
    pub fn delete(&mut self, key: &[u8]) {
        self.pending_writes
            .push(RefOp::Delete { key: key.to_vec() });
    }

    /// Stages a `Forget` operation in uncommitted pending buffer.
    pub fn forget(&mut self, key: &[u8]) {
        self.pending_writes
            .push(RefOp::Forget { key: key.to_vec() });
    }

    /// Clears any uncommitted pending writes without advancing sequence number.
    pub fn rollback(&mut self) {
        self.pending_writes.clear();
    }

    /// Commits all pending operations under a new sequence number and marks them acknowledged.
    pub fn commit(&mut self) -> u64 {
        self.commit_batch(Vec::new())
    }

    /// Commits pending operations under a new sequence number WITHOUT marking them acknowledged.
    pub fn commit_unacked(&mut self) -> u64 {
        let old_acked = self.acked_seq;
        let seq = self.commit_batch(Vec::new());
        self.acked_seq = old_acked;
        seq
    }

    /// Appends `ops` to pending buffer, commits them under a new sequence number, and marks acknowledged.
    pub fn commit_batch(&mut self, ops: Vec<RefOp>) -> u64 {
        self.pending_writes.extend(ops);
        if self.pending_writes.is_empty() {
            return self.current_seq;
        }

        self.current_seq += 1;
        let commit_seq = self.current_seq;

        for op in self.pending_writes.drain(..) {
            match op {
                RefOp::Put { key, value } => {
                    self.forgotten_at.remove(&key);
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
                RefOp::Forget { key } => {
                    self.forgotten_at.insert(key.clone(), commit_seq);
                    self.history
                        .entry(key)
                        .or_default()
                        .push((commit_seq, None));
                }
            }
        }

        self.acked_seq = commit_seq;
        commit_seq
    }

    /// Returns the last acknowledged sequence number.
    pub fn acked_seq(&self) -> u64 {
        self.acked_seq
    }

    /// Returns the current high water mark sequence number.
    pub fn snapshot_seq(&self) -> u64 {
        self.current_seq
    }

    /// Returns true if key was forgotten at or before current high water mark.
    pub fn is_forgotten(&self, key: &[u8]) -> bool {
        self.is_forgotten_at(key, self.current_seq)
    }

    /// Returns true if key was forgotten at or before `seq`.
    pub fn is_forgotten_at(&self, key: &[u8], seq: u64) -> bool {
        match self.forgotten_at.get(key) {
            Some(&f_seq) => f_seq <= seq,
            None => false,
        }
    }

    /// Retrieves the value of `key` as visible at `seq`.
    pub fn get_at(&self, key: &[u8], seq: u64) -> Option<Vec<u8>> {
        if self.is_forgotten_at(key, seq) {
            return None;
        }
        let versions = self.history.get(key)?;
        let latest = versions
            .iter()
            .rev()
            .find(|(version_seq, _)| *version_seq <= seq)?;

        latest.1.clone()
    }

    /// Scans keys starting with `prefix` and returns key-value pairs visible at `seq`.
    pub fn scan_prefix_at(&self, prefix: &[u8], seq: u64) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut results = Vec::new();
        for (key, versions) in self.history.range(prefix.to_vec()..) {
            if !key.starts_with(prefix) {
                break;
            }
            if self.is_forgotten_at(key, seq) {
                continue;
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
            if self.is_forgotten_at(key, seq) {
                continue;
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

    /// Gets the latest committed value for `key`.
    pub fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get_at(key, self.snapshot_seq())
    }

    /// Scans all entries starting with `prefix` at latest sequence number.
    pub fn scan_prefix_latest(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.scan_prefix_at(prefix, self.snapshot_seq())
    }

    /// Returns a full snapshot map of latest visible key-values at `seq`.
    pub fn snapshot_map_at(&self, seq: u64) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.scan_range_at(Bound::Unbounded, Bound::Unbounded, seq)
            .into_iter()
            .collect()
    }

    /// Verifies post-crash recovered state against oracle invariant rules.
    ///
    /// Rules:
    /// 1. Every acknowledged transaction MUST be fully visible (`seq >= acked_seq`).
    /// 2. Unacknowledged transactions are fully visible or fully invisible (atomic).
    /// 3. Commit order is preserved (Prefix Property).
    /// 4. Forgotten subjects yield no data.
    pub fn verify_recovery(&self, recovered: &BTreeMap<Vec<u8>, Vec<u8>>) -> Result<u64, String> {
        for (k, &f_seq) in &self.forgotten_at {
            if f_seq <= self.acked_seq && recovered.contains_key(k) {
                return Err(format!(
                    "Oracle Invariant Violation: Acknowledged forgotten key {:?} found in recovered snapshot",
                    k
                ));
            }
        }

        let mut matched_seq = None;
        for seq in self.acked_seq..=self.current_seq {
            let expected_map = self.snapshot_map_at(seq);
            if &expected_map == recovered {
                matched_seq = Some(seq);
                break;
            }
        }

        match matched_seq {
            Some(seq) => Ok(seq),
            None => Err(format!(
                "Oracle Invariant Violation: Recovered state does not match any valid transaction prefix state between acked_seq {} and max_seq {}",
                self.acked_seq, self.current_seq
            )),
        }
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
    fn test_reference_model_oracle_forget_and_recovery() {
        let mut model = ReferenceModel::new();

        model.put(b"k1", b"v1");
        model.put(b"k2", b"v2");
        let seq1 = model.commit();

        model.forget(b"k1");
        let seq2 = model.commit();

        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
        assert!(model.is_forgotten(b"k1"));

        let snapshot = model.snapshot_map_at(seq2);
        assert_eq!(model.verify_recovery(&snapshot), Ok(2));

        let mut invalid_snapshot = snapshot.clone();
        invalid_snapshot.insert(b"k1".to_vec(), b"v1".to_vec());
        assert!(model.verify_recovery(&invalid_snapshot).is_err());
    }
}
