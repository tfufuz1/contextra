//! Shared versioned sequence log for snapshot-isolated index searches (`_at` family).

// FILE-CONTEXT
// STAND: 2026-08-30T18:51:56Z (SESSION: e459bd5f)
// ZWECK: Shared Versioned Sequence Log für Snapshot-isolierte Index-Suchen (_at Familie).
// INVARIANTEN: Sichtbarkeitsprüfung: insert_seq <= as_of && (delete_seq.is_none() || delete_seq > as_of).
// HOTSPOTS: 20-75
// NICHT-OFFENSICHTLICH: Compaction prunt Einträge erst wenn delete_seq < min_active_seqno.
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md (ADR-024)

use std::time::{Duration, Instant};

use crate::types::DocId;

/// Default maximum pin duration (5 minutes) before diagnostic warnings are issued for expired pins.
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
    pinned_snapshots: ahash::AHashMap<u64, Vec<Instant>>,
    compacted_below: Option<u64>,
    deletions: ahash::AHashMap<DocId, u64>,
    max_pin_duration: Duration,
}

impl Default for SequenceLog {
    fn default() -> Self {
        Self::new()
    }
}

impl SequenceLog {
    /// Creates a new empty sequence log with default max pin duration (5 minutes).
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            pinned_snapshots: ahash::AHashMap::default(),
            compacted_below: None,
            deletions: ahash::AHashMap::default(),
            max_pin_duration: DEFAULT_MAX_PIN_DURATION,
        }
    }

    /// Sets a custom maximum pin duration for diagnostics and returns `self`.
    pub fn with_max_pin_duration(mut self, duration: Duration) -> Self {
        self.max_pin_duration = duration;
        self
    }

    /// Sets a custom maximum pin duration for diagnostics.
    pub fn set_max_pin_duration(&mut self, duration: Duration) {
        self.max_pin_duration = duration;
    }

    /// Returns the configured maximum pin duration.
    pub fn max_pin_duration(&self) -> Duration {
        self.max_pin_duration
    }

    /// Pins a historical sequence number to prevent rebuild from purging soft-deleted nodes active at this snapshot.
    pub fn pin_snapshot(&mut self, seq_no: u64) {
        self.pin_snapshot_at(seq_no, Instant::now());
    }

    /// Pins a historical sequence number at a specific timestamp for deterministic testing and lease tracking.
    pub fn pin_snapshot_at(&mut self, seq_no: u64, at: Instant) {
        self.pinned_snapshots.entry(seq_no).or_default().push(at);
    }

    /// Unpins a historical sequence number.
    pub fn unpin_snapshot(&mut self, seq_no: u64) {
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
    /// relative to the provided timestamp `now`.
    pub fn expired_pins_at(&self, now: Instant) -> Vec<u64> {
        let mut expired = Vec::new();
        for (&seq_no, timestamps) in &self.pinned_snapshots {
            for &ts in timestamps {
                if now.saturating_duration_since(ts) > self.max_pin_duration {
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
        self.expired_pins_at(Instant::now())
    }

    /// Calculates the minimum sequence number currently pinned for snapshot retention.
    /// Returns `None` if no snapshot sequence numbers are pinned.
    ///
    /// If an active pin exceeds `max_pin_duration`, a diagnostic warning is emitted.
    pub fn min_retention_seq(&self) -> Option<u64> {
        let min_seq = self.pinned_snapshots.keys().copied().min();
        if let Some(seq) = min_seq {
            let now = Instant::now();
            if let Some(timestamps) = self.pinned_snapshots.get(&seq) {
                for &ts in timestamps {
                    let duration = now.saturating_duration_since(ts);
                    if duration > self.max_pin_duration {
                        eprintln!(
                            "[WARN memfuse_core::seq_log] Snapshot pin active for seq_no {} exceeds max duration of {:?} (elapsed: {:?})",
                            seq,
                            self.max_pin_duration,
                            duration
                        );
                        break;
                    }
                }
            }
        }
        min_seq
    }

    /// Returns the deletion sequence number of the given document, if currently soft-deleted.
    pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64> {
        self.deletions.get(&doc_id).copied()
    }

    /// Records an insert operation at the given sequence number `seq`.
    pub fn record_insert(&mut self, doc_id: DocId, seq: u64) {
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
        let mut inserted = false;
        let mut deleted = false;
        for entry in &self.entries {
            if entry.doc_id == doc_id && entry.insert_seq <= seq_no {
                inserted = true;
                if let Some(del) = entry.delete_seq {
                    deleted = del <= seq_no;
                } else {
                    deleted = false;
                }
            }
        }
        inserted && !deleted
    }

    /// Compacts log entries where deletion sequence number is strictly less than `min_active_seqno`.
    pub fn compact(&mut self, min_active_seqno: u64) {
        self.compacted_below = Some(
            self.compacted_below
                .map_or(min_active_seqno, |c| c.max(min_active_seqno)),
        );
        self.entries.retain(|entry| {
            if let Some(del_seq) = entry.delete_seq {
                del_seq >= min_active_seqno
            } else {
                true
            }
        });
        self.deletions
            .retain(|_, &mut del_seq| del_seq >= min_active_seqno);
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
        let mut changes = Vec::new();
        for entry in &self.entries {
            if entry.insert_seq > snapshot_seq {
                changes.push(SeqLogChange::Insert {
                    doc_id: entry.doc_id,
                    seq: entry.insert_seq,
                });
            }
            if let Some(del_seq) = entry.delete_seq {
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
mod tests {
    use super::*;

    #[test]
    fn test_sequence_log_insert_delete_visibility() {
        let mut log = SequenceLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);

        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // expect #[cfg(test)]
        let doc2 = DocId::from_key("doc2").expect("valid doc id"); // expect #[cfg(test)]

        // Insert doc1 at seq 10, doc2 at seq 20
        log.record_insert(doc1, 10);
        log.record_insert(doc2, 20);
        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());

        // Visibility before insert
        assert!(!log.is_visible(doc1, 5));
        assert!(!log.is_visible(doc2, 15));

        // Visibility at and after insert
        assert!(log.is_visible(doc1, 10));
        assert!(log.is_visible(doc1, 15));
        assert!(log.is_visible(doc2, 20));

        // Record delete for doc1 at seq 30
        log.record_delete(doc1, 30);
        assert_eq!(log.len(), 2);

        // Visibility around delete seq
        assert!(log.is_visible(doc1, 29));
        assert!(!log.is_visible(doc1, 30));
        assert!(!log.is_visible(doc1, 35));

        // doc2 remains visible
        assert!(log.is_visible(doc2, 35));
    }

    #[test]
    fn test_sequence_log_reinsert_after_delete() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // expect #[cfg(test)]

        log.record_insert(doc1, 10);
        log.record_delete(doc1, 20);
        assert!(!log.is_visible(doc1, 25));

        // Re-insert doc1 at seq 30
        log.record_insert(doc1, 30);
        assert_eq!(log.len(), 2);

        // Visibility timelines
        assert!(log.is_visible(doc1, 15));
        assert!(!log.is_visible(doc1, 25));
        assert!(log.is_visible(doc1, 30));
        assert!(log.is_visible(doc1, 40));
    }

    #[test]
    fn test_sequence_log_compact() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // expect #[cfg(test)]
        let doc2 = DocId::from_key("doc2").expect("valid doc id"); // expect #[cfg(test)]

        log.record_insert(doc1, 10);
        log.record_delete(doc1, 20); // deleted at seq 20

        log.record_insert(doc2, 15);
        log.record_delete(doc2, 40); // deleted at seq 40

        assert_eq!(log.len(), 2);

        // Compact with min_active_seqno = 30
        // doc1 (delete_seq = 20 < 30) should be purged.
        // doc2 (delete_seq = 40 >= 30) should be retained.
        log.compact(30);
        assert_eq!(log.len(), 1);

        // Compact with min_active_seqno = 50
        // doc2 (delete_seq = 40 < 50) should be purged.
        log.compact(50);
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    #[test]
    fn test_sequence_log_edge_cases() {
        let mut log = SequenceLog::new();
        let doc1 = DocId::from_key("doc1").expect("valid doc id"); // expect #[cfg(test)]

        // Delete non-existent doc should not panic or add entry
        log.record_delete(doc1, 10);
        assert_eq!(log.len(), 0);

        // Multiple deletes on same active insert updates delete_seq or ignores second?
        log.record_insert(doc1, 5);
        log.record_delete(doc1, 15);
        log.record_delete(doc1, 25); // Should not overwrite existing delete_seq if already deleted
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

        // Pin snapshot at seq 5 (older than insert_seq 10)
        log.pin_snapshot(5);
        assert_eq!(log.min_retention_seq(), Some(5));

        // Pin another snapshot at seq 15
        log.pin_snapshot(15);
        assert_eq!(log.min_retention_seq(), Some(5));

        // Unpin seq 5
        log.unpin_snapshot(5);
        assert_eq!(log.min_retention_seq(), Some(15));

        // Unpin seq 15
        log.unpin_snapshot(15);
        assert_eq!(log.min_retention_seq(), None);
    }

    #[test]
    fn test_sequence_log_pin_ttl_and_expiration() {
        let mut log = SequenceLog::new().with_max_pin_duration(Duration::from_secs(10));
        assert_eq!(log.max_pin_duration(), Duration::from_secs(10));

        let now = Instant::now();
        let past_5s = now.checked_sub(Duration::from_secs(5)).unwrap_or(now);
        let past_15s = now.checked_sub(Duration::from_secs(15)).unwrap_or(now);

        // Pin seq 100 at past_5s (not expired)
        log.pin_snapshot_at(100, past_5s);
        // Pin seq 200 at past_15s (expired relative to now)
        log.pin_snapshot_at(200, past_15s);

        let expired = log.expired_pins_at(now);
        assert_eq!(expired, vec![200]);

        // Pin seq 200 a second time at past_5s (now has 2 pins: 1 expired, 1 active)
        log.pin_snapshot_at(200, past_5s);
        assert_eq!(log.min_retention_seq(), Some(100));

        // expired_pins_at should still report 200 because at least one pin on seq 200 is expired
        assert_eq!(log.expired_pins_at(now), vec![200]);

        // Unpin newest pin on 200 (pops past_5s pin)
        log.unpin_snapshot(200);
        // past_15s pin remains for seq 200, so seq 200 is still reported as expired relative to now
        assert_eq!(log.expired_pins_at(now), vec![200]);

        // Unpin second pin on 200 (pops past_15s pin) -> no pins left for seq 200
        log.unpin_snapshot(200);
        assert_eq!(log.expired_pins_at(now), Vec::<u64>::new());
        assert_eq!(log.min_retention_seq(), Some(100));

        // Unpin seq 100 -> no pins left
        log.unpin_snapshot(100);
        assert_eq!(log.min_retention_seq(), None);
    }

    #[test]
    fn test_sequence_log_pin_ttl_custom_duration() {
        let mut log = SequenceLog::new();
        assert_eq!(log.max_pin_duration(), DEFAULT_MAX_PIN_DURATION);

        log.set_max_pin_duration(Duration::from_millis(50));
        assert_eq!(log.max_pin_duration(), Duration::from_millis(50));

        let now = Instant::now();
        let past_100ms = now.checked_sub(Duration::from_millis(100)).unwrap_or(now);

        log.pin_snapshot_at(42, past_100ms);
        assert_eq!(log.expired_pins_at(now), vec![42]);

        // The pin is NOT automatically removed or forcibly unpinned
        assert_eq!(log.min_retention_seq(), Some(42));
        assert_eq!(log.expired_pins_at(now), vec![42]);
    }

    #[test]
    fn test_sequence_log_zero_panic_time_arithmetic() {
        let mut log = SequenceLog::new();
        let now = Instant::now();
        let future = now.checked_add(Duration::from_secs(100)).unwrap_or(now);

        // Pin in the future (clock skew edge case)
        log.pin_snapshot_at(50, future);

        // Checking expired pins with 'now' should handle saturating duration without panic
        let expired = log.expired_pins_at(now);
        assert!(expired.is_empty());

        assert_eq!(log.min_retention_seq(), Some(50));
    }
}
