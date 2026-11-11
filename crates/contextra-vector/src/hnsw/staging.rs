// FILE-CONTEXT
// ZWECK: Autarke Abstraktion für Transaktions-Staging und Versionsprotokollierung (Inversion of Control).
// INVARIANTEN: Zero-Panic Doctrine; isolierte Puffer-Verwaltung ohne feste Bindung an konkrete MVCC-Implementierungen.

use crate::types::{DocId, TxId};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::time::Duration;

/// Operation within a transactional staging buffer.
#[derive(Debug, Clone, PartialEq)]
pub enum VectorStagingOp {
    /// Insertion or update of a vector for a document.
    Insert { doc_id: DocId, embedding: Vec<f32> },
    /// Deletion of a document vector.
    Delete { doc_id: DocId },
}

/// Transaction staging buffer for vector operations.
///
/// Encapsulates transactional staging locally, isolating
/// from direct coupling to storage transaction buffers.
#[derive(Debug)]
pub struct VectorTxBuffer {
    buffers: RwLock<BTreeMap<u64, Vec<VectorStagingOp>>>,
    capacity: usize,
    #[allow(dead_code)]
    timeout: Duration,
}

impl VectorTxBuffer {
    /// Creates a new vector staging buffer with the specified concurrency ceiling and timeout.
    pub fn new(capacity: usize, timeout: Duration) -> Self {
        Self {
            buffers: RwLock::new(BTreeMap::new()),
            capacity,
            timeout,
        }
    }

    /// Stages an operation into the transaction buffer.
    pub fn stage(&self, tx: TxId, op: VectorStagingOp) -> crate::error::Result<()> {
        let mut guard = self.buffers.write();
        let ops = guard.entry(tx.inner()).or_default();
        if ops.len() >= self.capacity {
            return Err(crate::error::VectorError::Conflict(
                "Staging buffer capacity exceeded for transaction".into(),
            ));
        }
        ops.push(op);
        Ok(())
    }

    /// Drains and returns all staged operations for a given transaction.
    pub fn drain(&self, tx: TxId) -> Vec<VectorStagingOp> {
        self.buffers.write().remove(&tx.inner()).unwrap_or_default()
    }

    /// Discards staged operations on transaction rollback.
    pub fn discard(&self, tx: TxId) {
        self.buffers.write().remove(&tx.inner());
    }
}
