//! Vector index trait definition and statistics.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: VectorIndex Trait & VectorIndexStats für HNSW/Vektor-Indizes.
// INVARIANTEN: Rebuild-Triggering & Filtered Search Fallback.

use crate::types::{DocId, ScoredDocument, TxId};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;

/// Statistics for a vector index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct VectorIndexStats {
    /// Number of active (non-deleted) vectors.
    pub num_vectors: usize,
    /// Estimated memory usage in bytes.
    pub memory_usage_bytes: usize,
    /// Number of HNSW layers.
    pub num_layers: usize,
    /// Fraction of deleted vectors (0.0 to 1.0).
    #[serde(default)]
    pub deleted_ratio: f64,
    /// Number of full index rebuilds completed.
    #[serde(default)]
    pub rebuild_count: u64,
}

/// Vector Index Trait — abstrahiert die HNSW-Vektorsuche.
///
/// # Dyn-Kompatibilität
/// Verwendet native `async fn` (AFIT) für statischen Dispatch.
pub trait VectorIndex: Send + Sync + 'static {
    /// Inserts a vector with an associated document ID.
    fn insert(
        &self,
        tx: TxId,
        id: DocId,
        embedding: &[f32],
    ) -> impl Future<Output = Result<()>> + Send;

    /// Returns all active (non-deleted) document IDs in the index.
    fn all_doc_ids(&self) -> impl Future<Output = Result<Vec<DocId>>> + Send {
        async { Ok(Vec::new()) }
    }

    /// Inserts multiple vectors with associated document IDs.
    fn insert_batch(
        &self,
        tx: TxId,
        vectors: &[(DocId, &[f32])],
    ) -> impl Future<Output = Result<()>> + Send {
        async move {
            for (id, embedding) in vectors {
                self.insert(tx, *id, embedding).await?;
            }
            Ok(())
        }
    }

    /// Searches for the k nearest neighbors to a query vector.
    fn search(
        &self,
        query: &[f32],
        k: usize,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send;

    /// Searches for the k nearest neighbors to a query vector at a specific sequence number.
    ///
    /// # Errors
    /// Returns [`MemFuseError::CapabilityUnsupported`][crate::MemFuseError::CapabilityUnsupported]
    /// with capability `"snapshot_read_at"` if snapshot-isolated vector search is not implemented.
    /// Tested via `capability_coverage` test module.
    fn search_at(
        &self,
        query: &[f32],
        k: usize,
        seq_no: u64,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send {
        async move {
            let _ = (query, k, seq_no);
            Err(crate::error::MemFuseError::capability_unsupported(
                "snapshot_read_at",
                "Vector search snapshot isolation (search_at) is not supported by default — tracked in ADR-024",
            ))
        }
    }

    /// Searches with an optional filter predicate.
    ///
    /// # Default Behaviour
    /// Returns an error if a filter is provided. Implementors **MUST** override
    /// this method if filtered search is supported by their vector engine.
    ///
    /// # Errors
    /// Returns [`MemFuseError::CapabilityUnsupported`][crate::MemFuseError::CapabilityUnsupported]
    /// with capability `"vector_filtered_search"` if a filter predicate is passed to an engine without filter support.
    /// Tested via `capability_coverage` test module.
    ///
    /// # Note
    /// This default exists solely for backward compatibility.
    fn search_filtered(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send {
        async move {
            if filter.is_some() {
                return Err(crate::error::MemFuseError::capability_unsupported(
                    "vector_filtered_search",
                    "Filtered vector search is not supported by default for this vector engine",
                ));
            }
            self.search(query, k).await
        }
    }

    /// Deletes a vector by its document ID.
    fn delete(&self, tx: TxId, id: DocId) -> impl Future<Output = Result<()>> + Send;

    /// Commits a transaction.
    fn commit(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back a transaction.
    fn rollback(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back the entire index state to a specific transaction ID.
    fn rollback_to_tx(&self, tx_id: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Returns the last transaction ID processed by the index.
    fn last_tx_id(&self) -> impl Future<Output = Result<TxId>> + Send;

    /// Returns the number of vectors in the index.
    fn len(&self) -> impl Future<Output = usize> + Send;

    /// Returns true if the index is empty.
    fn is_empty(&self) -> impl Future<Output = bool> + Send {
        async { self.len().await == 0 }
    }

    /// Returns index statistics.
    fn stats(&self) -> impl Future<Output = Result<VectorIndexStats>> + Send;

    /// Returns true if the index requires a background rebuild (e.g. due to tombstone accumulation).
    fn is_rebuild_required(&self) -> bool {
        false
    }

    /// Triggers an asynchronous background rebuild of the index if supported.
    fn trigger_rebuild_async(&self) {}
}
