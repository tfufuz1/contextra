//! Shared versioned sequence log for snapshot-isolated index searches (`_at` family).

// FILE-CONTEXT
// STAND: 2026-10-15T00:00:00Z
// ZWECK: Shared Versioned Sequence Log für Snapshot-isolierte Index-Suchen (_at Familie).
// INVARIANTEN: Sichtbarkeitsprüfung: insert_seq <= as_of && (delete_seq.is_none() || delete_seq > as_of).
// HOTSPOTS: 20-75
// NICHT-OFFENSICHTLICH: Compaction prunt Einträge erst wenn delete_seq < min_active_seqno.

use std::time::Duration;

use crate::types::DocId;

/// Default maximum pin duration (5 minutes in nanoseconds / ticks) before diagnostic warnings are issued for expired pins.
pub const DEFAULT_MAX_PIN_DURATION_TICKS: u64 = 300 * 1_000_000_000;

/// Default maximum pin duration (5 minutes) for diagnostics.
pub const DEFAULT_MAX_PIN_DURATION: Duration = Duration::from_secs(300);

/// Versioned sequence log entry for snapshot isolation (`_at` family).
///
/// **Memory Overhead**: Each entry is 24 bytes (8 bytes `DocId` + 8 bytes `insert_seq` + 8 bytes `delete_seq: Option<u64>`).
/// Entries where `delete_seq` is below `min_active_seqno` can be pruned via `compact`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeqLogEntry {
    /// Document ID.
    pub doc_id: DocId,
    /// Sequence number at which this document entry was inserted.
    pub insert_seq: u64,
    /// Sequence number at which this document entry was deleted (if deleted).
    pub delete_seq: Option<u64>,
}

impl SeqLogEntry {
    /// Returns `true` if this log entry is visible at snapshot sequence number `as_of`.
    ///
    /// # Visibility Rule
    /// An entry is visible if `insert_seq <= as_of` AND (`delete_seq` is `None` OR `delete_seq > as_of`).
    ///
    /// # Complexity
    /// $O(1)$ allocation-free, loop-free check using at most two integer comparisons.
    #[inline]
    pub fn is_visible(&self, as_of: u64) -> bool {
        let as_of = as_of & !crate::types::TOMBSTONE_BIT;
        let insert_seq = self.insert_seq & !crate::types::TOMBSTONE_BIT;
        let delete_seq = self.delete_seq.map(|d| d & !crate::types::TOMBSTONE_BIT);
        insert_seq <= as_of && delete_seq.is_none_or(|del| del > as_of)
    }
}

/// Represents a historical sequence log change (insert or delete) for delta replaying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqLogChange {
    /// A document insertion at a specific sequence number.
    Insert {
        /// Document ID.
        doc_id: DocId,
        /// Sequence number of the insert.
        seq: u64,
    },
    /// A document deletion at a specific sequence number.
    Delete {
        /// Document ID.
        doc_id: DocId,
        /// Sequence number of the delete.
        seq: u64,
    },
}

impl SeqLogChange {
    /// Returns the sequence number associated with this change.
    pub fn seq(&self) -> u64 {
        match self {
            Self::Insert { seq, .. } | Self::Delete { seq, .. } => *seq,
        }
    }

    /// Returns the document ID associated with this change.
    pub fn doc_id(&self) -> DocId {
        match self {
            Self::Insert { doc_id, .. } | Self::Delete { doc_id, .. } => *doc_id,
        }
    }
}

/// Helper structure managing sequence log tracking and visibility filtering for index implementations.
#[derive(Debug, Clone)]
pub struct SequenceLog {
    entries: Vec<SeqLogEntry>,
    pinned_snapshots: ahash::AHashMap<u64, Vec<u64>>,
    compacted_below: Option<u64>,
    deletions: ahash::AHashMap<DocId, u64>,
    max_pin_duration_ticks: u64,
}

impl Default for SequenceLog {
    fn default() -> Self {
        Self::new()
    }
}

impl SequenceLog {
    /// Creates a new empty sequence log with default max pin duration (5 minutes in ticks).
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            pinned_snapshots: ahash::AHashMap::default(),
            compacted_below: None,
            deletions: ahash::AHashMap::default(),
            max_pin_duration_ticks: DEFAULT_MAX_PIN_DURATION_TICKS,
        }
    }

    /// Sets a custom maximum pin duration for diagnostics and returns `self`.
    pub fn with_max_pin_duration(mut self, duration: Duration) -> Self {
        self.set_max_pin_duration(duration);
        self
    }

    /// Sets a custom maximum pin duration for diagnostics in ticks (nanoseconds).
    pub fn set_max_pin_duration_ticks(&mut self, ticks: u64) {
        self.max_pin_duration_ticks = ticks;
    }

    /// Sets a custom maximum pin duration for diagnostics.
    pub fn set_max_pin_duration(&mut self, duration: Duration) {
        self.max_pin_duration_ticks = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
    }

    /// Returns the configured maximum pin duration in ticks (nanoseconds).
    pub fn max_pin_duration_ticks(&self) -> u64 {
        self.max_pin_duration_ticks
    }

    /// Returns the configured maximum pin duration.
    pub fn max_pin_duration(&self) -> Duration {
        Duration::from_nanos(self.max_pin_duration_ticks)
    }

    /// Pins a historical sequence number at baseline timestamp 0.
    pub fn pin_snapshot(&mut self, seq_no: u64) {
        self.pin_snapshot_at(seq_no, 0);
    }

    /// Pins a historical sequence number at a specific timestamp in ticks (`at_ticks`) for deterministic testing and lease tracking.
    pub fn pin_snapshot_at(&mut self, seq_no: u64, at_ticks: u64) {
        let seq_no = seq_no & !crate::types::TOMBSTONE_BIT;
        self.pinned_snapshots
            .entry(seq_no)
            .or_default()
            .push(at_ticks);
    }

    /// Unpins a historical sequence number.
    pub fn unpin_snapshot(&mut self, seq_no: u64) {
        let seq_no = seq_no & !crate::types::TOMBSTONE_BIT;
        if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.pinned_snapshots.entry(seq_no)
        {
            let timestamps = entry.get_mut();
            timestamps.pop();
            if timestamps.is_empty() {
                entry.remove();
            }
        }
    }

    /// Returns all pinned sequence numbers whose active pin duration exceeds `max_pin_duration`
    /// relative to the provided timestamp `now_ticks`.
    pub fn expired_pins_at(&self, now_ticks: u64) -> Vec<u64> {
        let mut expired = Vec::new();
        for (&seq_no, timestamps) in &self.pinned_snapshots {
            for &ts in timestamps {
                if now_ticks.saturating_sub(ts) > self.max_pin_duration_ticks {
                    expired.push(seq_no);
                    break;
                }
            }
        }
        expired.sort_unstable();
        expired
    }

    /// Returns all pinned sequence numbers whose active pin duration exceeds `max_pin_duration`.
    pub fn expired_pins(&self) -> Vec<u64> {
        self.expired_pins_at(0)
    }

    /// Calculates the minimum sequence number currently pinned for snapshot retention at timestamp `now_ticks`.
    /// Returns `None` if no snapshot sequence numbers are pinned.
    ///
    /// If an active pin exceeds `max_pin_duration`, a diagnostic warning is emitted.
    pub fn min_retention_seq_at(&self, now_ticks: u64) -> Option<u64> {
        let min_seq = self.pinned_snapshots.keys().copied().min();
        if let Some(seq) = min_seq {
            if let Some(timestamps) = self.pinned_snapshots.get(&seq) {
                for &ts in timestamps {
                    let duration_ticks = now_ticks.saturating_sub(ts);
                    if duration_ticks > self.max_pin_duration_ticks {
                        tracing::warn!(
                            seq_no = seq,
                            max_pin_duration_ticks = self.max_pin_duration_ticks,
                            elapsed_ticks = duration_ticks,
                            "Snapshot pin active beyond max duration"
                        );
                        break;
                    }
                }
            }
        }
        min_seq
    }

    /// Calculates the minimum sequence number currently pinned for snapshot retention.
    /// Returns `None` if no snapshot sequence numbers are pinned.
    pub fn min_retention_seq(&self) -> Option<u64> {
        self.min_retention_seq_at(0)
    }

    /// Returns the deletion sequence number of the given document, if currently soft-deleted.
    pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64> {
        self.deletions.get(&doc_id).copied()
    }

    /// Records an insert operation at the given sequence number `seq`.
    pub fn record_insert(&mut self, doc_id: DocId, seq: u64) {
        let seq = seq & !crate::types::TOMBSTONE_BIT;
        if let Some(entry) = self
            .entries
            .iter_mut()
            .rfind(|e| e.doc_id == doc_id && e.delete_seq.is_none())
        {
            entry.delete_seq = Some(seq);
        }
        self.deletions.remove(&doc_id);
        self.entries.push(SeqLogEntry {
            doc_id,
            insert_seq: seq,
            delete_seq: None,
        });
    }

    /// Records a delete operation at the given sequence number `seq`.
    pub fn record_delete(&mut self, doc_id: DocId, seq: u64) {
        let seq = seq & !crate::types::TOMBSTONE_BIT;
        if let Some(entry) = self
            .entries
            .iter_mut()
            .rfind(|e| e.doc_id == doc_id && e.delete_seq.is_none())
        {
            entry.delete_seq = Some(seq);
            self.deletions.insert(doc_id, seq);
        }
    }

    /// Checks if `doc_id` was inserted at or before `seq_no` and not deleted at or before `seq_no`.
    pub fn is_visible(&self, doc_id: DocId, seq_no: u64) -> bool {
        let seq_no = seq_no & !crate::types::TOMBSTONE_BIT;
        let mut inserted = false;
        let mut deleted = false;
        for entry in &self.entries {
            let insert_seq = entry.insert_seq & !crate::types::TOMBSTONE_BIT;
            if entry.doc_id == doc_id && insert_seq <= seq_no {
                inserted = true;
                if let Some(del) = entry.delete_seq {
                    let del_seq = del & !crate::types::TOMBSTONE_BIT;
                    deleted = del_seq <= seq_no;
                } else {
                    deleted = false;
                }
            }
        }
        inserted && !deleted
    }

    /// Compacts log entries where deletion sequence number is strictly less than `min_active_seqno`.
    pub fn compact(&mut self, min_active_seqno: u64) {
        let min_active_seqno = min_active_seqno & !crate::types::TOMBSTONE_BIT;
        self.compacted_below = Some(
            self.compacted_below
                .map_or(min_active_seqno, |c| c.max(min_active_seqno)),
        );
        self.entries.retain(|entry| {
            if let Some(del_seq) = entry.delete_seq {
                let del_seq = del_seq & !crate::types::TOMBSTONE_BIT;
                del_seq >= min_active_seqno
            } else {
                true
            }
        });
        self.deletions
            .retain(|_, &mut del_seq| (del_seq & !crate::types::TOMBSTONE_BIT) >= min_active_seqno);
    }

    /// Returns the number of entries in the sequence log.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the sequence log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns all sequence log changes that occurred strictly after `snapshot_seq`,
    /// ordered chronologically by sequence number.
    pub fn changes_since(&self, snapshot_seq: u64) -> Vec<SeqLogChange> {
        let snapshot_seq = snapshot_seq & !crate::types::TOMBSTONE_BIT;
        let mut changes = Vec::new();
        for entry in &self.entries {
            let insert_seq = entry.insert_seq & !crate::types::TOMBSTONE_BIT;
            if insert_seq > snapshot_seq {
                changes.push(SeqLogChange::Insert {
                    doc_id: entry.doc_id,
                    seq: insert_seq,
                });
            }
            if let Some(del_seq) = entry.delete_seq {
                let del_seq = del_seq & !crate::types::TOMBSTONE_BIT;
                if del_seq > snapshot_seq {
                    changes.push(SeqLogChange::Delete {
                        doc_id: entry.doc_id,
                        seq: del_seq,
                    });
                }
            }
        }
        changes.sort_by_key(|c| c.seq());
        changes
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_log_entry_is_visible_o1() {
        let doc = DocId::from_key("doc1").expect("valid doc id");

        let active_entry = SeqLogEntry {
            doc_id: doc,
            insert_seq: 10,
            delete_seq: None,
        };
        assert!(!active_entry.is_visible(9));
        assert!(active_entry.is_visible(10));
        assert!(active_entry.is_visible(100));

        let deleted_entry = SeqLogEntry {
            doc_id: doc,
            insert_seq: 10,
            delete_seq: Some(20),
        };
        assert!(!deleted_entry.is_visible(9));
        assert!(deleted_entry.is_visible(10));
        assert!(deleted_entry.is_visible(19));
        assert!(!deleted_entry.is_visible(20));
        assert!(!deleted_entry.is_visible(21));
    }

    #[test]
    fn test_sequence_log_insert_delete_visibility() {
        let mut log = SequenceLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);

        let doc1 = DocId::from_key("doc1").expect("valid doc id");
        let doc2 = DocId::from_key("doc2").expect("valid doc id");

        log.record_insert(doc1, 10);
        log.record_insert(doc2, 20);
        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());

        assert!(!log.is_visible(doc1, 5));
        assert!(!log.is_visible(doc2, 15));

        assert!(log.is_visible(doc1, 10));
        assert!(log.is_visible(doc1, 15));
        assert!(log.is_visible(doc2, 20));

        log.record_delete(doc1, 30);
        assert_eq!(log.len(), 2);

        assert!(log.is_visible(doc1, 29));
        assert!(!log.is_visible(doc1, 30));
        assert!(!log.is_visible(doc1, 35));

        assert!(log.is_visible(doc2, 35));
    }

    #[test]
    fn test_sequence_log_reinsert_after_delete() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id");

        log.record_insert(doc1, 10);
        log.record_delete(doc1, 20);
        assert!(!log.is_visible(doc1, 25));

        log.record_insert(doc1, 30);
        assert_eq!(log.len(), 2);

        assert!(log.is_visible(doc1, 15));
        assert!(!log.is_visible(doc1, 25));
        assert!(log.is_visible(doc1, 30));
        assert!(log.is_visible(doc1, 40));
    }

    #[test]
    fn test_sequence_log_compact() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id");
        let doc2 = DocId::from_key("doc2").expect("valid doc id");

        log.record_insert(doc1, 10);
        log.record_delete(doc1, 20);

        log.record_insert(doc2, 15);
        log.record_delete(doc2, 40);

        assert_eq!(log.len(), 2);

        log.compact(30);
        assert_eq!(log.len(), 1);

        log.compact(50);
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    #[test]
    fn test_sequence_log_edge_cases() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id");

        log.record_delete(doc1, 10);
        assert_eq!(log.len(), 0);

        log.record_insert(doc1, 5);
        log.record_delete(doc1, 15);
        log.record_delete(doc1, 25);
        assert!(log.is_visible(doc1, 10));
        assert!(!log.is_visible(doc1, 15));
    }

    #[test]
    fn test_sequence_log_changes_since() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id");
        let doc2 = DocId::from_key("doc2").expect("valid doc id");

        log.record_insert(doc1, 10);
        log.record_insert(doc2, 20);
        log.record_delete(doc1, 30);
        log.record_insert(doc1, 40);

        let changes_0 = log.changes_since(0);
        assert_eq!(
            changes_0,
            vec![
                SeqLogChange::Insert {
                    doc_id: doc1,
                    seq: 10
                },
                SeqLogChange::Insert {
                    doc_id: doc2,
                    seq: 20
                },
                SeqLogChange::Delete {
                    doc_id: doc1,
                    seq: 30
                },
                SeqLogChange::Insert {
                    doc_id: doc1,
                    seq: 40
                },
            ]
        );

        let changes_15 = log.changes_since(15);
        assert_eq!(
            changes_15,
            vec![
                SeqLogChange::Insert {
                    doc_id: doc2,
                    seq: 20
                },
                SeqLogChange::Delete {
                    doc_id: doc1,
                    seq: 30
                },
                SeqLogChange::Insert {
                    doc_id: doc1,
                    seq: 40
                },
            ]
        );

        let changes_35 = log.changes_since(35);
        assert_eq!(
            changes_35,
            vec![SeqLogChange::Insert {
                doc_id: doc1,
                seq: 40
            }]
        );

        let changes_50 = log.changes_since(50);
        assert!(changes_50.is_empty());
    }

    #[test]
    fn test_sequence_log_pinning_and_retention() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id");

        log.record_insert(doc1, 10);
        log.record_delete(doc1, 20);

        assert_eq!(log.deletion_seq(doc1), Some(20));
        assert_eq!(log.min_retention_seq(), None);

        log.pin_snapshot_at(5, 100);
        assert_eq!(log.min_retention_seq_at(100), Some(5));

        log.pin_snapshot_at(15, 100);
        assert_eq!(log.min_retention_seq_at(100), Some(5));

        log.unpin_snapshot(5);
        assert_eq!(log.min_retention_seq_at(100), Some(15));

        log.unpin_snapshot(15);
        assert_eq!(log.min_retention_seq_at(100), None);
    }

    #[test]
    fn test_sequence_log_pin_ttl_and_expiration() {
        let mut log = SequenceLog::new().with_max_pin_duration(Duration::from_secs(10));
        assert_eq!(log.max_pin_duration(), Duration::from_secs(10));

        let now_ticks = 20_000_000_000u64; // 20s
        let past_5s = 15_000_000_000u64;
        let past_15s = 5_000_000_000u64;

        log.pin_snapshot_at(100, past_5s);
        log.pin_snapshot_at(200, past_15s);

        let expired = log.expired_pins_at(now_ticks);
        assert_eq!(expired, vec![200]);

        log.pin_snapshot_at(200, past_5s);
        assert_eq!(log.min_retention_seq_at(now_ticks), Some(100));

        assert_eq!(log.expired_pins_at(now_ticks), vec![200]);

        log.unpin_snapshot(200);
        assert_eq!(log.expired_pins_at(now_ticks), vec![200]);

        log.unpin_snapshot(200);
        assert_eq!(log.expired_pins_at(now_ticks), Vec::<u64>::new());
        assert_eq!(log.min_retention_seq_at(now_ticks), Some(100));

        log.unpin_snapshot(100);
        assert_eq!(log.min_retention_seq_at(now_ticks), None);
    }

    #[test]
    fn test_sequence_log_pin_ttl_custom_duration() {
        let mut log = SequenceLog::new();
        assert_eq!(log.max_pin_duration(), DEFAULT_MAX_PIN_DURATION);

        log.set_max_pin_duration(Duration::from_millis(50));
        assert_eq!(log.max_pin_duration(), Duration::from_millis(50));

        let now_ticks = 200_000_000u64; // 200ms
        let past_100ms = 100_000_000u64; // 100ms

        log.pin_snapshot_at(42, past_100ms);
        assert_eq!(log.expired_pins_at(now_ticks), vec![42]);

        assert_eq!(log.min_retention_seq_at(now_ticks), Some(42));
        assert_eq!(log.expired_pins_at(now_ticks), vec![42]);
    }
}
