//! Graph index trait definition and statistics.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: GraphIndex Trait & GraphIndexStats für CSR-basierte Entity-Relation-Graph-Operationen.
// INVARIANTEN: AGT-GRAPH-001 TxId-Origin-Invariante. BoxFuture dyn-safety.

use super::BoxFuture;
use crate::types::{Edge, Entity, EntityId, PprConfig, TxId};
use crate::Result;
use ahash::AHashMap;

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
        start_node: EntityId,
        max_hops: usize,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>;

    /// Returns direct (1-hop) neighbor EntityIds for the given entity.
    fn neighbors<'a>(&'a self, start_node: EntityId) -> BoxFuture<'a, Result<Vec<EntityId>>> {
        Box::pin(async move {
            let results = self.traverse(start_node, 1).await?;
            Ok(results.into_iter().map(|(id, _)| id).collect())
        })
    }

    /// Removes an edge between two entities.
    fn remove_edge<'a>(
        &'a self,
        tx: TxId,
        from: EntityId,
        to: EntityId,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let _ = (tx, from, to);
            Ok(())
        })
    }

    /// Adds a bidirectional edge between two entities.
    fn add_bidirectional<'a>(
        &'a self,
        tx: TxId,
        from: EntityId,
        to: EntityId,
        label: &'a str,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.add_edge(tx, Edge::new(from, to, label)).await?;
            self.add_edge(tx, Edge::new(to, from, label)).await?;
            Ok(())
        })
    }

    /// Traverses the entity graph starting from multiple anchor entities up to max_hops.
    /// Aggregates decay weights (keeping max score per entity) across anchors.
    fn multi_traverse<'a>(
        &'a self,
        start_nodes: &'a [EntityId],
        max_hops: usize,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let mut combined: AHashMap<EntityId, f32> = AHashMap::default();
            for &start in start_nodes {
                let results = self.traverse(start, max_hops).await?;
                for (entity_id, score) in results {
                    combined
                        .entry(entity_id)
                        .and_modify(|s| *s = s.max(score))
                        .or_insert(score);
                }
            }
            let mut results: Vec<(EntityId, f32)> = combined.into_iter().collect();
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            Ok(results)
        })
    }

    /// Traverses the entity graph starting from multiple anchor entities up to max_hops at a specific sequence number.
    /// Aggregates decay weights (keeping max score per entity) across anchors.
    fn multi_traverse_at<'a>(
        &'a self,
        start_nodes: &'a [EntityId],
        max_hops: usize,
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let mut combined: AHashMap<EntityId, f32> = AHashMap::default();
            for &start in start_nodes {
                let results = self.traverse_at(start, max_hops, seq_no).await?;
                for (entity_id, score) in results {
                    combined
                        .entry(entity_id)
                        .and_modify(|s| *s = s.max(score))
                        .or_insert(score);
                }
            }
            let mut results: Vec<(EntityId, f32)> = combined.into_iter().collect();
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
        _start_node: EntityId,
        _max_hops: usize,
        _seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
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
        _start_node: EntityId,
        _max_hops: usize,
        _as_of: TxId,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
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
        _start_node: EntityId,
        _max_hops: usize,
        _as_of_tx: TxId,
        _as_of_business: Option<i64>,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
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
        _seed_nodes: &'a [EntityId],
        _config: &'a PprConfig,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            Err(crate::error::MemFuseError::capability_unsupported(
                "graph_ppr",
                "Personalized PageRank (personalized_page_rank) is not supported by default for this GraphIndex implementation",
            ))
        })
    }

    /// Inserts or updates a node entity.
    fn add_entity<'a>(&'a self, tx: TxId, entity: Entity) -> BoxFuture<'a, Result<()>>;

    /// Inserts or updates an edge between two entities.
    fn add_edge<'a>(&'a self, tx: TxId, edge: Edge) -> BoxFuture<'a, Result<()>>;

    /// Commits a transaction.
    fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back a transaction.
    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rolls back the entire graph state to a specific transaction ID.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Returns the last transaction ID processed by the index.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;

    /// Returns the number of entities in the index.
    fn len<'a>(&'a self) -> BoxFuture<'a, usize>;

    /// Returns true if the index is empty.
    fn is_empty<'a>(&'a self) -> BoxFuture<'a, bool> {
        Box::pin(async move { self.len().await == 0 })
    }

    /// Collects statistics for the Graph.
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>>;
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

// AI-TAG[ARCH][MINOR][RESOLVED] (virtuell verschoben von memfuse-router/ports_local.rs per TODO(welle-3), siehe docs/refactor/router-db-edge-audit.md)
/// Contract for resolving graph community assignments for entities.
pub trait CommunityResolver: Send + Sync {
    /// Resolves the optional community ID for a given entity.
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>>;
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn test_graph_index_defaults() {
        struct MockGraphIndex;
        impl GraphIndex for MockGraphIndex {
            fn traverse<'a>(
                &'a self,
                _: EntityId,
                _: usize,
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

        let index = MockGraphIndex;
        let res = index.traverse_at(EntityId::new(1), 2, 42).await;
        match res {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, reason }) => {
                assert_eq!(capability, "graph_traverse_at");
                assert!(reason.contains("ADR-024"), "Unexpected reason: {reason}");
            }
            _ => panic!("Expected CapabilityUnsupported with ADR-024"),
        }

        let res_time = index
            .traverse_at_time(EntityId::new(1), 2, TxId::new(10))
            .await;
        match res_time {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "graph_traverse_at_time");
            }
            _ => panic!("Expected CapabilityUnsupported for traverse_at_time"),
        }

        let res_ppr = index
            .personalized_page_rank(&[EntityId::new(1)], &PprConfig::default())
            .await;
        match res_ppr {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "graph_ppr");
            }
            _ => panic!("Expected CapabilityUnsupported for personalized_page_rank"),
        }
    }
}
