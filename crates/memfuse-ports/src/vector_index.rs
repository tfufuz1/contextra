//! Vector index trait definition and statistics.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: VectorIndex Trait & VectorIndexStats für HNSW/Vektor-Indizes.
// INVARIANTEN: Rebuild-Triggering & Filtered Search Fallback.

use super::BoxFuture;
use crate::types::{ContextChunk, DocId, ScoredDocument, TxId};
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

// INVARIANT: Implementor: HnswIndex (memfuse-index/src/hnsw.rs)
// Rebuild: Automatisch bei >20% gelöschten Nodes.

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

// AI-TAG[ARCH][MINOR][RESOLVED] (virtuell verschoben von memfuse-router/ports_local.rs per TODO(welle-3), siehe docs/refactor/router-db-edge-audit.md)
/// Contract for executing hybrid (vector + text) queries for profile routing.
pub trait HybridSearchProvider: Send + Sync {
    /// Executes a hybrid query returning matched context chunks with relevance scores.
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_index_stats_serialization() {
        let v_stats = VectorIndexStats {
            num_vectors: 100,
            memory_usage_bytes: 1024,
            num_layers: 5,
            deleted_ratio: 0.1,
            rebuild_count: 1,
        };
        let ser = serde_json::to_string(&v_stats).unwrap();
        let deser: VectorIndexStats = serde_json::from_str(&ser).unwrap();
        assert_eq!(v_stats.num_vectors, deser.num_vectors);
    }

    #[tokio::test]
    async fn test_hnsw_search_at_capability() {
        struct VectorIndexPlaceholder;
        impl VectorIndex for VectorIndexPlaceholder {
            async fn insert(&self, _: TxId, _: DocId, _: &[f32]) -> Result<()> {
                Ok(())
            }
            async fn search(&self, _: &[f32], _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn search_at(&self, _: &[f32], _: usize, _: u64) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn delete(&self, _: TxId, _: DocId) -> Result<()> {
                Ok(())
            }
            async fn commit(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback_to_tx(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn last_tx_id(&self) -> Result<TxId> {
                Ok(TxId(0))
            }
            async fn len(&self) -> usize {
                0
            }
            async fn stats(&self) -> Result<VectorIndexStats> {
                Ok(VectorIndexStats {
                    num_vectors: 0,
                    memory_usage_bytes: 0,
                    num_layers: 0,
                    deleted_ratio: 0.0,
                    rebuild_count: 0,
                })
            }
        }
        let index = VectorIndexPlaceholder;
        let res = index.search_at(&[1.0, 0.0], 5, 1).await;
        assert!(
            !matches!(res, Err(crate::MemFuseError::CapabilityUnsupported { .. })),
            "search_at returned CapabilityUnsupported"
        );
        assert!(!index.is_rebuild_required());
        index.trigger_rebuild_async();
    }

    #[tokio::test]
    async fn test_vector_index_defaults() {
        struct MockIndex(std::sync::atomic::AtomicUsize);
        impl VectorIndex for MockIndex {
            async fn insert(&self, _: TxId, _: DocId, _: &[f32]) -> Result<()> {
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
            async fn search(&self, _: &[f32], _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn delete(&self, _: TxId, _: DocId) -> Result<()> {
                Ok(())
            }
            async fn commit(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback_to_tx(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn last_tx_id(&self) -> Result<TxId> {
                Ok(TxId(0))
            }
            async fn len(&self) -> usize {
                self.0.load(std::sync::atomic::Ordering::SeqCst)
            }
            async fn stats(&self) -> Result<VectorIndexStats> {
                Ok(VectorIndexStats {
                    num_vectors: 0,
                    memory_usage_bytes: 0,
                    num_layers: 0,
                    deleted_ratio: 0.0,
                    rebuild_count: 0,
                })
            }
        }

        let index = MockIndex(std::sync::atomic::AtomicUsize::new(0));
        assert!(index.is_empty().await);

        let vectors = vec![
            (DocId(1), [1.0, 2.0].as_slice()),
            (DocId(2), [3.0, 4.0].as_slice()),
        ];
        index.insert_batch(TxId(1), &vectors).await.unwrap();
        assert_eq!(index.len().await, 2);
        assert!(!index.is_empty().await);

        // Test search_filtered default error
        let res = index.search_filtered(&[1.0], 1, Some(&|_| true)).await;
        match res {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "vector_filtered_search");
            }
            _ => panic!("Expected CapabilityUnsupported for search_filtered"),
        }

        // Test search_at default error
        let res2 = index.search_at(&[1.0], 1, 42).await;
        match res2 {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, reason }) => {
                assert_eq!(capability, "snapshot_read_at");
                assert!(reason.contains("ADR-024"), "Unexpected reason: {reason}");
            }
            _ => panic!("Expected CapabilityUnsupported with ADR-024 for search_at"),
        }
    }
}
