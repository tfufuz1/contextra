// FILE-CONTEXT
// ZWECK: Autarke Abstraktion für Transaktions-Staging und Versionsprotokollierung (Inversion of Control).
// INVARIANTEN: Zero-Panic Doctrine; isolierte Puffer-Verwaltung ohne feste Bindung an konkrete MVCC-Implementierungen.

use contextra_types::{ContextraError, DocId, TxId};
use parking_lot::Mutex;
use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};

/// Operation within a transactional staging buffer.
#[derive(Debug, Clone, PartialEq)]
pub enum VectorStagingOp {
    /// Insertion or update of a vector for a document.
    Insert { doc_id: DocId, embedding: Vec<f32> },
    /// Deletion of a document vector.
    Delete { doc_id: DocId },
}

#[derive(Debug, Clone)]
enum LocalIndexOp {
    Insert { doc_id: DocId, data: Vec<f32> },
    Delete { doc_id: DocId },
}

#[derive(Debug)]
struct StagedTx {
    ops: Vec<LocalIndexOp>,
    created_at: Instant,
}

/// Transaction staging buffer for vector operations.
///
/// Encapsulates transactional staging locally, isolating
/// from direct coupling to external MVCC crates (SPEC K-48).
#[derive(Debug)]
pub struct VectorTxBuffer {
    capacity: usize,
    timeout: Duration,
    staged: Mutex<HashMap<TxId, StagedTx>>,
}

impl VectorTxBuffer {
    /// Creates a new vector staging buffer with the specified concurrency ceiling and timeout.
    pub fn new(capacity: usize, timeout: Duration) -> Self {
        Self {
            capacity,
            timeout,
            staged: Mutex::new(HashMap::new()),
        }
    }

    /// Stages an operation into the transaction buffer.
    pub fn stage(&self, tx: TxId, op: VectorStagingOp) -> contextra_types::Result<()> {
        let mut guard = self.staged.lock();
        if !guard.contains_key(&tx) && guard.len() >= self.capacity {
            let now = Instant::now();
            let timeout = self.timeout;
            guard.retain(|_, entry| now.duration_since(entry.created_at) < timeout);
            if guard.len() >= self.capacity {
                return Err(ContextraError::Transaction(
                    "VectorTxBuffer capacity exceeded".to_string(),
                ));
            }
        }
        let entry = guard.entry(tx).or_insert_with(|| StagedTx {
            ops: Vec::new(),
            created_at: Instant::now(),
        });
        match op {
            VectorStagingOp::Insert { doc_id, embedding } => {
                entry.ops.push(LocalIndexOp::Insert {
                    doc_id,
                    data: embedding,
                });
            }
            VectorStagingOp::Delete { doc_id } => {
                entry.ops.push(LocalIndexOp::Delete { doc_id });
            }
        }
        Ok(())
    }

    /// Drains and returns all staged operations for a given transaction.
    pub fn drain(&self, tx: TxId) -> Vec<VectorStagingOp> {
        let mut guard = self.staged.lock();
        let Some(entry) = guard.remove(&tx) else {
            return Vec::new();
        };
        entry
            .ops
            .into_iter()
            .map(|op| match op {
                LocalIndexOp::Insert { doc_id, data } => VectorStagingOp::Insert {
                    doc_id,
                    embedding: data,
                },
                LocalIndexOp::Delete { doc_id } => VectorStagingOp::Delete { doc_id },
            })
            .collect()
    }

    /// Discards staged operations on transaction rollback.
    pub fn discard(&self, tx: TxId) {
        self.staged.lock().remove(&tx);
    }
}

/// Change entry recorded in the local sequence log for delta replay during rebuild.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqLogChange {
    Insert { doc_id: DocId, seq: u64 },
    Delete { doc_id: DocId, seq: u64 },
}

/// Self-contained sequence log for tracking committed vector visibility across snapshots.
#[derive(Debug, Default)]
pub struct SequenceLog {
    history: BTreeMap<u64, Vec<SeqLogChange>>,
    doc_events: HashMap<u64, Vec<(u64, bool)>>,
}

impl SequenceLog {
    /// Creates an empty `SequenceLog`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a committed insert at sequence `seq`.
    pub fn record_insert(&mut self, doc_id: DocId, seq: u64) {
        self.history
            .entry(seq)
            .or_default()
            .push(SeqLogChange::Insert { doc_id, seq });
        self.doc_events
            .entry(doc_id.inner())
            .or_default()
            .push((seq, true));
    }

    /// Records a committed delete at sequence `seq`.
    pub fn record_delete(&mut self, doc_id: DocId, seq: u64) {
        self.history
            .entry(seq)
            .or_default()
            .push(SeqLogChange::Delete { doc_id, seq });
        self.doc_events
            .entry(doc_id.inner())
            .or_default()
            .push((seq, false));
    }

    /// Returns true if `doc_id` is visible at snapshot sequence `snap_seq`.
    pub fn is_visible(&self, doc_id: DocId, snap_seq: u64) -> bool {
        let Some(events) = self.doc_events.get(&doc_id.inner()) else {
            return false;
        };
        let mut visible = false;
        for &(seq, is_insert) in events {
            if seq <= snap_seq {
                visible = is_insert;
            } else {
                break;
            }
        }
        visible
    }

    /// Returns all changes strictly after `since_seq`.
    pub fn changes_since(&self, since_seq: u64) -> Vec<SeqLogChange> {
        let mut out = Vec::new();
        for (_seq, changes) in self.history.range((since_seq.saturating_add(1))..) {
            out.extend(changes.iter().copied());
        }
        out
    }
}
