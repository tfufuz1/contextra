//! Indexing subsystem traits (VectorIndex, TextIndex, GraphIndex, DistanceCalculator).

use super::BoxFuture;
use crate::types::*;
use crate::Result;
use ahash::AHashMap;
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

/// Text embedding engine trait.
pub trait TextEmbeddingEngine: Send + Sync + 'static {
    /// Generates an embedding for the given text.
    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>>;

    /// Generates embeddings for multiple texts.
    /// Default implementation executes sequential calls.
    fn embed_batch<'a>(&'a self, texts: &'a [&'a str]) -> BoxFuture<'a, Result<Vec<Vec<f32>>>> {
        Box::pin(async move {
            let mut results = Vec::with_capacity(texts.len());
            for text in texts {
                results.push(self.embed(text).await?);
            }
            Ok(results)
        })
    }
}

/// Trait-Abstraktion für LLM-Synthesizer zur Segment-Zusammenfassung (REM-Phase).
pub trait SegmentSynthesizer: Send + Sync {
    /// Synthetisiert ein Segment von Texten zu einer abstrakten Zusammenfassung.
    fn synthesize_segment<'a>(
        &'a self,
        segment_texts: &'a [&'a str],
    ) -> BoxFuture<'a, Result<String>>;
    /// Gibt die Modell-ID des Synthesizers zurück.
    fn model_id(&self) -> &str;
}

/// Statistics for a text index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextIndexStats {
    /// Total number of documents indexed.
    pub num_documents: usize,
    /// Total number of tokens across all documents.
    pub num_tokens: usize,
    /// Estimated memory usage in bytes.
    pub memory_usage_bytes: usize,
}

/// Text-Index Trait — abstrahiert BM25/Inverted-Index-Operationen.
///
/// # Dyn-Kompatibilität
/// Verwendet native `async fn` (AFIT) für statischen Dispatch.
pub trait TextIndex: Send + Sync + 'static {
    /// Searches for documents matching the query.
    fn search(
        &self,
        query: &str,
        k: usize,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send;

    /// Searches for documents matching the query at a specific sequence number.
    ///
    /// # Errors
    /// Returns [`MemFuseError::CapabilityUnsupported`][crate::MemFuseError::CapabilityUnsupported]
    /// with capability `"snapshot_read_at"` if snapshot-isolated text search is not implemented.
    /// Tested via `capability_coverage` test module.
    fn search_at(
        &self,
        query: &str,
        k: usize,
        seq_no: u64,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send {
        async move {
            let _ = (query, k, seq_no);
            Err(crate::error::MemFuseError::capability_unsupported(
                "snapshot_read_at",
                "Text search snapshot isolation (search_at) is not supported by default — tracked in ADR-024",
            ))
        }
    }

    /// Inserts or updates a document in the index.
    fn insert(&self, tx: TxId, id: DocId, text: &str) -> impl Future<Output = Result<()>> + Send;

    /// Deletes a document from the index.
    fn delete(&self, tx: TxId, id: DocId) -> impl Future<Output = Result<()>> + Send;

    /// Commits a transaction.
    fn commit(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back a transaction.
    fn rollback(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back the entire index state to a specific transaction ID.
    fn rollback_to_tx(&self, tx_id: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Returns the last transaction ID processed by the index.
    fn last_tx_id(&self) -> impl Future<Output = Result<TxId>> + Send;

    /// Returns the number of documents in the index.
    fn len(&self) -> impl Future<Output = usize> + Send;

    /// Returns true if the index is empty.
    fn is_empty(&self) -> impl Future<Output = bool> + Send {
        async { self.len().await == 0 }
    }

    /// Returns index statistics.
    fn stats(&self) -> impl Future<Output = Result<TextIndexStats>> + Send;
}

// INVARIANT: Graph Engine Trait (Signal 3)

/// Graph-Index Trait — CSR-basierte Entity-Relation-Traversal.
///
/// # Dyn-Kompatibilität
/// Dieser Trait ist durch explizite `BoxFuture`-Rückgabetypen vtable-kompatibel (dyn-safe).
///
/// # TxId-Origin-Invariant (AGT-GRAPH-001)
///
/// **Aufrufer MÜSSEN tx entweder aus der Collection-eigenen next_tx-Sequenz oder aus TxId::INTERNAL_BASE-Offset-Bereich beziehen.**
///
/// **Aufrufer MÜSSEN sicherstellen, dass `tx`-Argumente für [`add_entity`],
/// [`add_edge`] und [`commit`] ausschließlich aus einer der folgenden beiden
/// kanonischen Quellen stammen:**
///
/// 1. **Collection-eigene Sequenz**: Der `next_tx: Arc<AtomicU64>` Zähler in
///    `memfuse-db/src/collection.rs`, der kollisionsfrei aufsteigend inkrementiert
///    wird. Solche TxIds liegen typischerweise im Bereich `[1, ~10^12]`.
///
/// 2. **Interner Systembereich**: `TxId::INTERNAL_BASE` (`u64::MAX - 1_000_000`)
///    aufwärts — reserviert für Checkpoint, WAL-Replay und andere
///    System-Transaktionen (Muster: `memfuse-checkpoint/src/lib.rs:76-79`).
///
/// **Verbotene Quellen:**
/// - Wall-Clock-abgeleitete TxIds (z.B. `SystemTime::now().as_nanos() as u64`
///   ≈ `1.7×10¹⁸`). Diese liegen zufällig zwischen den beiden erlaubten
///   Bereichen und korrumpieren die `rollback_to_tx()`-Kausalordnung: Der Graph
///   "vergisst" nie committed Daten, aber Time-Travel-Wiederherstellung kann
///   die falsche Transaktionsgrenze wählen.
/// - Beliebige fremde IDs ohne Korrelation zur Collection-eigenen Sequenz.
///
/// Implementierungen DÜRFEN bei Verletzung dieses Vertrags eine Warnung loggen
/// (mittels `tracing::warn!`), aber MÜSSEN die Operation nicht hart ablehnen,
/// da der `next_tx`-Höchststand der aufrufenden Collection dem Graph nicht
/// bekannt ist.
///
/// [`add_entity`]: GraphIndex::add_entity
/// [`add_edge`]: GraphIndex::add_edge
/// [`commit`]: GraphIndex::commit
pub trait GraphIndex: Send + Sync + 'static {
    /// Traverses the entity graph using BFS up to a maximum number of hops.
    /// Distributes traversing decay weights across related entities.
    fn traverse<'a>(
        &'a self,
        start_node: crate::types::EntityId,
        max_hops: usize,
    ) -> BoxFuture<'a, crate::Result<Vec<(crate::types::EntityId, f32)>>>;

    /// Returns direct (1-hop) neighbor EntityIds for the given entity.
    fn neighbors<'a>(
        &'a self,
        start_node: crate::types::EntityId,
    ) -> BoxFuture<'a, crate::Result<Vec<crate::types::EntityId>>> {
        Box::pin(async move {
            let results = self.traverse(start_node, 1).await?;
            Ok(results.into_iter().map(|(id, _)| id).collect())
        })
    }

    /// Removes an edge between two entities.
    fn remove_edge<'a>(
        &'a self,
        tx: crate::types::TxId,
        from: crate::types::EntityId,
        to: crate::types::EntityId,
    ) -> BoxFuture<'a, crate::Result<()>> {
        Box::pin(async move {
            let _ = (tx, from, to);
            Ok(())
        })
    }

    /// Adds a bidirectional edge between two entities.
    fn add_bidirectional<'a>(
        &'a self,
        tx: crate::types::TxId,
        from: crate::types::EntityId,
        to: crate::types::EntityId,
        label: &'a str,
    ) -> BoxFuture<'a, crate::Result<()>> {
        Box::pin(async move {
            self.add_edge(tx, crate::types::Edge::new(from, to, label))
                .await?;
            self.add_edge(tx, crate::types::Edge::new(to, from, label))
                .await?;
            Ok(())
        })
    }

    /// Traverses the entity graph starting from multiple anchor entities up to max_hops.
    /// Aggregates decay weights (keeping max score per entity) across anchors.
    fn multi_traverse<'a>(
        &'a self,
        start_nodes: &'a [crate::types::EntityId],
        max_hops: usize,
    ) -> BoxFuture<'a, crate::Result<Vec<(crate::types::EntityId, f32)>>> {
        Box::pin(async move {
            let mut combined: AHashMap<crate::types::EntityId, f32> = AHashMap::default();
            for &start in start_nodes {
                let results = self.traverse(start, max_hops).await?;
                for (entity_id, score) in results {
                    combined
                        .entry(entity_id)
                        .and_modify(|s| *s = s.max(score))
                        .or_insert(score);
                }
            }
            let mut results: Vec<(crate::types::EntityId, f32)> = combined.into_iter().collect();
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            Ok(results)
        })
    }

    /// Traverses the entity graph starting from multiple anchor entities up to max_hops at a specific sequence number.
    /// Aggregates decay weights (keeping max score per entity) across anchors.
    fn multi_traverse_at<'a>(
        &'a self,
        start_nodes: &'a [crate::types::EntityId],
        max_hops: usize,
        seq_no: u64,
    ) -> BoxFuture<'a, crate::Result<Vec<(crate::types::EntityId, f32)>>> {
        Box::pin(async move {
            let mut combined: AHashMap<crate::types::EntityId, f32> = AHashMap::default();
            for &start in start_nodes {
                let results = self.traverse_at(start, max_hops, seq_no).await?;
                for (entity_id, score) in results {
                    combined
                        .entry(entity_id)
                        .and_modify(|s| *s = s.max(score))
                        .or_insert(score);
                }
            }
            let mut results: Vec<(crate::types::EntityId, f32)> = combined.into_iter().collect();
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            Ok(results)
        })
    }

    /// Traverses the entity graph using BFS up to a maximum number of hops at a specific sequence number.
    ///
    /// # Errors
    /// Returns [`MemFuseError::CapabilityUnsupported`][crate::MemFuseError::CapabilityUnsupported]
    /// with capability `"graph_traverse_at"` if snapshot-isolated graph traversal is not implemented.
    /// Tested via `capability_coverage` test module.
    fn traverse_at<'a>(
        &'a self,
        _start_node: crate::types::EntityId,
        _max_hops: usize,
        _seq_no: u64,
    ) -> BoxFuture<'a, crate::Result<Vec<(crate::types::EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::MemFuseError::capability_unsupported(
                "graph_traverse_at",
                "Graph traversal snapshot isolation (traverse_at) is not supported by default — tracked in ADR-024",
            ))
        })
    }

    /// Traverses the entity graph using BFS at a specific point in time (bi-temporal edge filtering).
    ///
    /// # Errors
    /// Returns [`MemFuseError::CapabilityUnsupported`][crate::MemFuseError::CapabilityUnsupported]
    /// with capability `"graph_traverse_at_time"` if bi-temporal graph traversal is not implemented.
    /// Tested via `capability_coverage` test module.
    fn traverse_at_time<'a>(
        &'a self,
        _start_node: crate::types::EntityId,
        _max_hops: usize,
        _as_of: crate::types::TxId,
    ) -> BoxFuture<'a, crate::Result<Vec<(crate::types::EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::MemFuseError::capability_unsupported(
                "graph_traverse_at_time",
                "Bi-temporal graph traversal (traverse_at_time) is not supported by default",
            ))
        })
    }

    /// Traverses the entity graph using BFS with independent system time and business time constraints.
    ///
    /// # Errors
    /// Returns [`MemFuseError::CapabilityUnsupported`][crate::MemFuseError::CapabilityUnsupported]
    /// with capability `"graph_traverse_at_bitemporal"` if bitemporal graph traversal is not implemented.
    fn traverse_at_bitemporal<'a>(
        &'a self,
        _start_node: crate::types::EntityId,
        _max_hops: usize,
        _as_of_tx: crate::types::TxId,
        _as_of_business: Option<i64>,
    ) -> BoxFuture<'a, crate::Result<Vec<(crate::types::EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::MemFuseError::capability_unsupported(
                "graph_traverse_at_bitemporal",
                "Bi-temporal graph traversal (traverse_at_bitemporal) is not supported by default",
            ))
        })
    }

    /// Calculates Personalized PageRank (PPR) starting from seed nodes.
    ///
    /// # Convergence Behavior
    /// Power iteration terminates when the L1 norm difference between iterations drops below `config.convergence_epsilon`,
    /// or when `config.max_iterations` is reached. If `config.max_iterations` is reached without full convergence,
    /// the function returns the best-effort intermediate ranking state (no `Err`) and emits a `tracing::warn!` log entry.
    ///
    /// # Errors
    /// Returns [`MemFuseError::CapabilityUnsupported`][crate::MemFuseError::CapabilityUnsupported]
    /// with capability `"graph_ppr"` if Personalized PageRank is not supported by this implementation.
    /// Tested via `capability_coverage` test module.
    fn personalized_page_rank<'a>(
        &'a self,
        _seed_nodes: &'a [crate::types::EntityId],
        _config: &'a crate::types::PprConfig,
    ) -> BoxFuture<'a, crate::Result<Vec<(crate::types::EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::MemFuseError::capability_unsupported(
                "graph_ppr",
                "Personalized PageRank (personalized_page_rank) is not supported by default for this GraphIndex implementation",
            ))
        })
    }

    /// Inserts or updates a node entity.
    fn add_entity<'a>(
        &'a self,
        tx: crate::types::TxId,
        entity: crate::types::Entity,
    ) -> BoxFuture<'a, crate::Result<()>>;

    /// Inserts or updates an edge between two entities.
    fn add_edge<'a>(
        &'a self,
        tx: crate::types::TxId,
        edge: crate::types::Edge,
    ) -> BoxFuture<'a, crate::Result<()>>;

    /// Commits a transaction.
    fn commit<'a>(&'a self, tx: crate::types::TxId) -> BoxFuture<'a, crate::Result<()>>;

    /// Rolls back a transaction.
    fn rollback<'a>(&'a self, tx: crate::types::TxId) -> BoxFuture<'a, crate::Result<()>>;

    /// Rolls back the entire graph state to a specific transaction ID.
    fn rollback_to_tx<'a>(&'a self, tx_id: crate::types::TxId) -> BoxFuture<'a, crate::Result<()>>;

    /// Returns the last transaction ID processed by the index.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, crate::Result<crate::types::TxId>>;

    /// Returns the number of entities in the index.
    fn len<'a>(&'a self) -> BoxFuture<'a, usize>;

    /// Returns true if the index is empty.
    fn is_empty<'a>(&'a self) -> BoxFuture<'a, bool> {
        Box::pin(async move { self.len().await == 0 })
    }

    /// Collects statistics for the Graph.
    fn stats<'a>(&'a self) -> BoxFuture<'a, crate::Result<GraphIndexStats>>;
}

/// Statistics for the GraphIndex layer.
#[derive(Debug, Clone)]
pub struct GraphIndexStats {
    /// Number of active nodes (Entities).
    pub num_entities: usize,
    /// Number of active edges.
    pub num_edges: usize,
    /// Total bytes allocated by CSR representation.
    pub memory_usage_bytes: usize,
}

/// Distance calculator trait for vector comparison.
pub trait DistanceCalculator: Send + Sync {
    /// Computes the distance between two f32 vectors.
    fn compute_f32(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes the distance between two u8 vectors.
    fn compute_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_stats_serialization() {
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

        let t_stats = TextIndexStats {
            num_documents: 10,
            num_tokens: 1000,
            memory_usage_bytes: 256,
        };
        let ser = serde_json::to_string(&t_stats).unwrap();
        let deser: TextIndexStats = serde_json::from_str(&ser).unwrap();
        assert_eq!(t_stats.num_documents, deser.num_documents);
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
    async fn test_csr_graph_capability() {
        struct GraphIndexPlaceholder;
        impl GraphIndex for GraphIndexPlaceholder {
            fn traverse<'a>(
                &'a self,
                _: EntityId,
                _: usize,
            ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn traverse_at<'a>(
                &'a self,
                _: EntityId,
                _: usize,
                _: u64,
            ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn traverse_at_time<'a>(
                &'a self,
                _: EntityId,
                _: usize,
                _: TxId,
            ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn traverse_at_bitemporal<'a>(
                &'a self,
                _: EntityId,
                _: usize,
                _: TxId,
                _: Option<i64>,
            ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn add_entity<'a>(&'a self, _: TxId, _: Entity) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn add_edge<'a>(&'a self, _: TxId, _: Edge) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
                Box::pin(async move { 0 })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>> {
                Box::pin(async move {
                    Ok(GraphIndexStats {
                        num_entities: 0,
                        num_edges: 0,
                        memory_usage_bytes: 0,
                    })
                })
            }
        }
        let graph = GraphIndexPlaceholder;
        let res_traverse_at = graph.traverse_at(EntityId::new(1), 2, 1).await;
        assert!(
            !matches!(
                res_traverse_at,
                Err(crate::MemFuseError::CapabilityUnsupported { .. })
            ),
            "traverse_at returned CapabilityUnsupported"
        );

        let res_traverse_at_time = graph
            .traverse_at_time(EntityId::new(1), 2, TxId::new(1))
            .await;
        assert!(
            !matches!(
                res_traverse_at_time,
                Err(crate::MemFuseError::CapabilityUnsupported { .. })
            ),
            "traverse_at_time returned CapabilityUnsupported"
        );

        let res_traverse_at_bitemporal = graph
            .traverse_at_bitemporal(EntityId::new(1), 2, TxId::new(1), Some(1000))
            .await;
        assert!(
            !matches!(
                res_traverse_at_bitemporal,
                Err(crate::MemFuseError::CapabilityUnsupported { .. })
            ),
            "traverse_at_bitemporal returned CapabilityUnsupported"
        );
    }

    #[tokio::test]
    async fn test_text_index_search_at_capability() {
        struct TextIndexPlaceholder;
        impl TextIndex for TextIndexPlaceholder {
            async fn search(&self, _: &str, _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn search_at(&self, _: &str, _: usize, _: u64) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn insert(&self, _: TxId, _: DocId, _: &str) -> Result<()> {
                Ok(())
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
            async fn stats(&self) -> Result<TextIndexStats> {
                Ok(TextIndexStats {
                    num_documents: 0,
                    num_tokens: 0,
                    memory_usage_bytes: 0,
                })
            }
        }
        let text_index = TextIndexPlaceholder;
        let res = text_index.search_at("test", 5, 1).await;
        assert!(
            !matches!(res, Err(crate::MemFuseError::CapabilityUnsupported { .. })),
            "search_at returned CapabilityUnsupported"
        );
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

    #[tokio::test]
    async fn test_text_index_defaults() {
        struct MockTextIndex;
        impl TextIndex for MockTextIndex {
            async fn search(&self, _: &str, _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn insert(&self, _: TxId, _: DocId, _: &str) -> Result<()> {
                Ok(())
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
            async fn stats(&self) -> Result<TextIndexStats> {
                Ok(TextIndexStats {
                    num_documents: 0,
                    num_tokens: 0,
                    memory_usage_bytes: 0,
                })
            }
        }

        let index = MockTextIndex;
        let res = index.search_at("query", 10, 42).await;
        match res {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, reason }) => {
                assert_eq!(capability, "snapshot_read_at");
                assert!(reason.contains("ADR-024"), "Unexpected reason: {reason}");
            }
            _ => panic!("Expected CapabilityUnsupported for search_at"),
        }
    }

    #[tokio::test]
    async fn test_graph_index_defaults() {
        struct MockGraphIndex;
        impl GraphIndex for MockGraphIndex {
            fn traverse<'a>(
                &'a self,
                _: crate::types::EntityId,
                _: usize,
            ) -> BoxFuture<'a, crate::Result<Vec<(crate::types::EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn add_entity<'a>(
                &'a self,
                _: crate::types::TxId,
                _: crate::types::Entity,
            ) -> BoxFuture<'a, crate::Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn add_edge<'a>(
                &'a self,
                _: crate::types::TxId,
                _: crate::types::Edge,
            ) -> BoxFuture<'a, crate::Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: crate::types::TxId) -> BoxFuture<'a, crate::Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: crate::types::TxId) -> BoxFuture<'a, crate::Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(
                &'a self,
                _: crate::types::TxId,
            ) -> BoxFuture<'a, crate::Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, crate::Result<crate::types::TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
                Box::pin(async move { 0 })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, crate::Result<GraphIndexStats>> {
                Box::pin(async move {
                    Ok(GraphIndexStats {
                        num_entities: 0,
                        num_edges: 0,
                        memory_usage_bytes: 0,
                    })
                })
            }
        }

        let index = MockGraphIndex;
        let res = index
            .traverse_at(crate::types::EntityId::new(1), 2, 42)
            .await;
        match res {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, reason }) => {
                assert_eq!(capability, "graph_traverse_at");
                assert!(reason.contains("ADR-024"), "Unexpected reason: {reason}");
            }
            _ => panic!("Expected CapabilityUnsupported with ADR-024"),
        }

        let res_time = index
            .traverse_at_time(
                crate::types::EntityId::new(1),
                2,
                crate::types::TxId::new(10),
            )
            .await;
        match res_time {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "graph_traverse_at_time");
            }
            _ => panic!("Expected CapabilityUnsupported for traverse_at_time"),
        }

        let res_ppr = index
            .personalized_page_rank(
                &[crate::types::EntityId::new(1)],
                &crate::types::PprConfig::default(),
            )
            .await;
        match res_ppr {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "graph_ppr");
            }
            _ => panic!("Expected CapabilityUnsupported for personalized_page_rank"),
        }
    }
}
