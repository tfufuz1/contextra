use contextra_ports::{VectorIndex, VectorIndexStats};
use contextra_types::{ContextraError, DocId, Result, ScoredDocument, TxId};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
enum StagedVectorOp {
    Insert(Vec<f32>),
    Delete,
}

type VectorMap = BTreeMap<DocId, Vec<f32>>;
type StagedVectorTxMap = BTreeMap<TxId, BTreeMap<DocId, StagedVectorOp>>;

/// An in-memory reference implementation of [`VectorIndex`] for testing.
#[derive(Debug, Clone)]
pub struct InMemoryVectorIndex {
    dim: usize,
    data: Arc<RwLock<VectorMap>>,
    staged: Arc<RwLock<StagedVectorTxMap>>,
    last_tx: Arc<RwLock<TxId>>,
}

impl InMemoryVectorIndex {
    /// Creates a new `InMemoryVectorIndex` with expected vector dimension `dim`.
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            data: Arc::new(RwLock::new(BTreeMap::new())),
            staged: Arc::new(RwLock::new(BTreeMap::new())),
            last_tx: Arc::new(RwLock::new(TxId::new(0))),
        }
    }
}

impl VectorIndex for InMemoryVectorIndex {
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()> {
        if embedding.len() != self.dim {
            return Err(ContextraError::invalid_input(format!(
                "vector dimension mismatch: expected {}, got {}",
                self.dim,
                embedding.len()
            )));
        }
        for &val in embedding {
            if !val.is_finite() {
                return Err(ContextraError::invalid_input(
                    "vector embedding contains non-finite value (NaN/Infinity)",
                ));
            }
        }

        let mut staged = self.staged.write();
        staged
            .entry(tx)
            .or_default()
            .insert(id, StagedVectorOp::Insert(embedding.to_vec()));
        Ok(())
    }

    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        let mut staged = self.staged.write();
        staged
            .entry(tx)
            .or_default()
            .insert(id, StagedVectorOp::Delete);
        Ok(())
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        let ops = {
            let mut staged = self.staged.write();
            staged.remove(&tx)
        };

        if let Some(ops) = ops {
            let mut data = self.data.write();
            for (id, op) in ops {
                match op {
                    StagedVectorOp::Insert(v) => {
                        data.insert(id, v);
                    }
                    StagedVectorOp::Delete => {
                        data.remove(&id);
                    }
                }
            }
            *self.last_tx.write() = tx;
        }
        Ok(())
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        let mut staged = self.staged.write();
        staged.remove(&tx);
        Ok(())
    }

    async fn rollback_to_tx(&self, _tx_id: TxId) -> Result<()> {
        self.staged.write().clear();
        Ok(())
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        Ok(*self.last_tx.read())
    }

    async fn all_doc_ids(&self) -> Result<Vec<DocId>> {
        let data = self.data.read();
        Ok(data.keys().copied().collect())
    }

    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        if query.len() != self.dim {
            return Err(ContextraError::invalid_input(format!(
                "query dimension mismatch: expected {}, got {}",
                self.dim,
                query.len()
            )));
        }
        for &val in query {
            if !val.is_finite() {
                return Err(ContextraError::invalid_input(
                    "query vector contains non-finite value (NaN/Infinity)",
                ));
            }
        }

        let data = self.data.read();
        let mut scored: Vec<ScoredDocument> = data
            .iter()
            .map(|(&doc_id, vec)| {
                let dist_sq: f32 = query
                    .iter()
                    .zip(vec.iter())
                    .map(|(q, v)| (q - v) * (q - v))
                    .sum();
                let dist = dist_sq.sqrt();
                let score = 1.0 / (1.0 + dist);
                ScoredDocument::new(doc_id, score)
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        scored.truncate(k);
        Ok(scored)
    }

    async fn len(&self) -> usize {
        self.data.read().len()
    }

    async fn stats(&self) -> Result<VectorIndexStats> {
        let len = self.len().await;
        Ok(VectorIndexStats {
            num_vectors: len,
            memory_usage_bytes: len * self.dim * 4,
            num_layers: 1,
            deleted_ratio: 0.0,
            rebuild_count: 0,
        })
    }
}
