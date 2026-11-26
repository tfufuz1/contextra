// FILE-CONTEXT
// ZWECK: Autarke Abstraktion für Transaktions-Staging und Versionsprotokollierung (Inversion of Control).
// INVARIANTEN: Zero-Panic Doctrine; isolierte Puffer-Verwaltung ohne feste Bindung an konkrete MVCC-Implementierungen.

use contextra_types::{DocId, TxId};

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
    inner: contextra_mvcc::TxBuffer<Vec<f32>>,
}

impl VectorTxBuffer {
    /// Creates a new vector staging buffer with the specified concurrency ceiling and timeout.
    pub fn new(capacity: usize, timeout: std::time::Duration) -> Self {
        Self {
            inner: contextra_mvcc::TxBuffer::new_with_config(capacity, timeout),
        }
    }

    /// Stages an operation into the transaction buffer.
    pub fn stage(&self, tx: TxId, op: VectorStagingOp) -> contextra_types::Result<()> {
        match op {
            VectorStagingOp::Insert { doc_id, embedding } => {
                self.inner.stage(
                    tx,
                    contextra_mvcc::IndexOp::Insert {
                        doc_id,
                        data: embedding,
                    },
                )?;
            }
            VectorStagingOp::Delete { doc_id } => {
                self.inner
                    .stage(tx, contextra_mvcc::IndexOp::Delete { doc_id, data: None })?;
            }
        }
        Ok(())
    }

    /// Drains and returns all staged operations for a given transaction.
    pub fn drain(&self, tx: TxId) -> Vec<VectorStagingOp> {
        self.inner
            .drain(tx)
            .into_iter()
            .filter_map(|op| match op {
                contextra_mvcc::IndexOp::Insert { doc_id, data } => Some(VectorStagingOp::Insert {
                    doc_id,
                    embedding: data,
                }),
                contextra_mvcc::IndexOp::Delete { doc_id, .. } => {
                    Some(VectorStagingOp::Delete { doc_id })
                }
                _ => None,
            })
            .collect()
    }

    /// Discards staged operations on transaction rollback.
    pub fn discard(&self, tx: TxId) {
        self.inner.discard(tx);
    }
}
