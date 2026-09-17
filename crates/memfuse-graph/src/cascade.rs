// FILE-CONTEXT
// ZWECK: Cascading-Invalidation (Supersedes-Chunk -> Graph-Kanten-Tombstone)
// INVARIANTEN: Jede tombstonierte Kante erhaelt einen Provenienz-Eintrag mit WAL-Seq (INV-GRAPH-PROV-1).
// STAND: TS:2026-08-30T19:00:00Z

use crate::csr::CsrGraph;
use memfuse_core::{DocId, EntityId, Result, TxId};

/// Maximum number of hyperedges processed synchronously per cascade invalidation run
/// to bound latency spikes. Any remaining hyperedges are returned in `deferred`.
pub const MAX_HYPEREDGE_CASCADE_FANOUT: usize = 1_000;

/// Report summarizing the cascade invalidation of graph edges derived from a superseded document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CascadeInvalidationReport {
    /// Number of edges tombstoned as a result of the superseded document.
    pub tombstoned_edge_count: usize,
    /// Unique node (entity) IDs affected by the tombstoned edges.
    pub affected_node_ids: Vec<EntityId>,
}

/// Tombstoniert alle Kanten, die von `superseded_doc_id` abgeleitet wurden,
/// als Reaktion auf ein Supersedes-Ereignis. Idempotent: mehrfacher Aufruf
/// mit derselben DocId hat keinen zusätzlichen Effekt.
///
/// INVARIANTE: Jede hier tombstonierte Kante erhält einen Provenienz-Eintrag
/// mit der WAL-Sequenznummer dieses Aufrufs (INV-GRAPH-PROV-1).
pub async fn cascade_invalidate_edges_for_superseded_doc(
    graph: &CsrGraph,
    superseded_doc_id: DocId,
    wal_seq: u64,
) -> Result<CascadeInvalidationReport> {
    let wal_tx = TxId::new(wal_seq);

    // 1. Lookup aller EdgeIds via DocId -> Set<EdgeId>-Index.
    let edge_ids = graph.edges_for_doc(superseded_doc_id);
    if edge_ids.is_empty() {
        return Ok(CascadeInvalidationReport {
            tombstoned_edge_count: 0,
            affected_node_ids: Vec::new(),
        });
    }

    // 2. Für jede gefundene EdgeId: existierendes Tombstone-Verfahren aufrufen (idempotent).
    let (tombstoned_count, newly_tombstoned_edges, affected_nodes) =
        graph.tombstone_edges_direct(&edge_ids, wal_tx)?;

    // 3. Persistieren falls Storage vorhanden.
    if let Some(storage) = graph.storage() {
        for (from_id, to_id) in &newly_tombstoned_edges {
            graph
                .delete_edge_persistence(storage.as_ref(), wal_tx, from_id, to_id)
                .await?;
        }
    }

    Ok(CascadeInvalidationReport {
        tombstoned_edge_count: tombstoned_count,
        affected_node_ids: affected_nodes,
    })
}

/// Report summarizing the cascade invalidation of hyperedges derived from a superseded document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperedgeCascadeReport {
    /// Hyperedges tombstoned synchronously in this cascade run (up to MAX_HYPEREDGE_CASCADE_FANOUT).
    pub invalidated: Vec<crate::hyperedge::HyperEdgeId>,
    /// Hyperedges deferred for background processing due to fan-out limit.
    pub deferred: Vec<crate::hyperedge::HyperEdgeId>,
}

/// Cascade invalidates hyperedges derived from `superseded_doc_id` with fan-out protection.
///
/// Up to `MAX_HYPEREDGE_CASCADE_FANOUT` hyperedges are tombstoned synchronously.
/// Any excess hyperedges are returned in `deferred` for the caller (e.g., in `memfuse-db`)
/// to schedule for deferred background processing, preserving DAG layering without
/// cross-layer imports.
///
/// INVARIANTE: Jede hier tombstonierte Hyperkante erhält einen Provenienz-Eintrag
/// mit der WAL-Sequenznummer dieses Aufrufs (INV-GRAPH-PROV-1).
pub async fn cascade_invalidate_hyperedges_for_superseded_doc(
    graph: &CsrGraph,
    superseded_doc_id: DocId,
    wal_seq: u64,
) -> Result<HyperedgeCascadeReport> {
    let wal_tx = TxId::new(wal_seq);
    let candidate_ids = graph.hyperedges_for_doc(superseded_doc_id);
    if candidate_ids.is_empty() {
        return Ok(HyperedgeCascadeReport {
            invalidated: Vec::new(),
            deferred: Vec::new(),
        });
    }

    let mut invalidated = Vec::with_capacity(candidate_ids.len().min(MAX_HYPEREDGE_CASCADE_FANOUT));
    let mut deferred = Vec::new();

    for (idx, hyperedge_id) in candidate_ids.into_iter().enumerate() {
        if idx < MAX_HYPEREDGE_CASCADE_FANOUT {
            if graph.tombstone_hyperedge(hyperedge_id, wal_tx) {
                invalidated.push(hyperedge_id);
            }
        } else {
            deferred.push(hyperedge_id);
        }
    }

    Ok(HyperedgeCascadeReport {
        invalidated,
        deferred,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csr::EdgeType;
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
    use crate::path_rag::PathRAGEngine;
    use memfuse_core::{Edge, GraphIndex};

    #[tokio::test]
    async fn test_cascade_invalidation_tombstones_edges_of_superseded_doc() {
        let graph = std::sync::Arc::new(CsrGraph::new());
        let tx1 = TxId::new(1);
        let doc_a = DocId::from_key("doc-a").unwrap();
        let doc_b = DocId::from_key("doc-b").unwrap();

        let node1 = EntityId::new(100);
        let node2 = EntityId::new(200);
        let node3 = EntityId::new(300);

        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            memfuse_core::Entity::new(node1, "n1", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            memfuse_core::Entity::new(node2, "n2", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            memfuse_core::Entity::new(node3, "n3", "Node"),
        )
        .await
        .unwrap();

        // Edge 1 derived from Doc A
        let edge1 = Edge::new(node1, node2, "rel_a").with_source_doc_id(doc_a);
        // Edge 2 derived from Doc B
        let edge2 = Edge::new(node2, node3, "rel_b").with_source_doc_id(doc_b);

        GraphIndex::add_edge(graph.as_ref(), tx1, edge1)
            .await
            .unwrap();
        GraphIndex::add_edge(graph.as_ref(), tx1, edge2)
            .await
            .unwrap();
        GraphIndex::commit(graph.as_ref(), tx1).await.unwrap();

        assert_eq!(graph.neighbors(node1).await.unwrap().len(), 1);
        assert_eq!(graph.neighbors(node2).await.unwrap().len(), 1);

        // Cascade invalidate doc A
        let report = cascade_invalidate_edges_for_superseded_doc(&graph, doc_a, 2)
            .await
            .unwrap();

        assert_eq!(report.tombstoned_edge_count, 1);
        assert!(report.affected_node_ids.contains(&node1));
        assert!(report.affected_node_ids.contains(&node2));

        // Node 1 -> Node 2 edge should now be tombstoned
        assert!(graph.neighbors(node1).await.unwrap().is_empty());
        // Node 2 -> Node 3 edge (Doc B) remains unaffected
        assert_eq!(graph.neighbors(node2).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_cascade_invalidation_is_idempotent() {
        let graph = std::sync::Arc::new(CsrGraph::new());
        let tx1 = TxId::new(1);
        let doc_a = DocId::from_key("doc-a").unwrap();

        let node1 = EntityId::new(100);
        let node2 = EntityId::new(200);

        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            memfuse_core::Entity::new(node1, "n1", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            memfuse_core::Entity::new(node2, "n2", "Node"),
        )
        .await
        .unwrap();

        let edge = Edge::new(node1, node2, "rel").with_source_doc_id(doc_a);
        GraphIndex::add_edge(graph.as_ref(), tx1, edge)
            .await
            .unwrap();
        GraphIndex::commit(graph.as_ref(), tx1).await.unwrap();

        // First call
        let report1 = cascade_invalidate_edges_for_superseded_doc(&graph, doc_a, 2)
            .await
            .unwrap();
        assert_eq!(report1.tombstoned_edge_count, 1);

        // Second call (idempotent)
        let report2 = cascade_invalidate_edges_for_superseded_doc(&graph, doc_a, 3)
            .await
            .unwrap();
        assert_eq!(report2.tombstoned_edge_count, 0);
        assert!(report2.affected_node_ids.is_empty());
    }

    #[tokio::test]
    async fn test_pathrag_sufficiency_excludes_cascaded_tombstones() {
        let graph = std::sync::Arc::new(CsrGraph::new());
        let tx1 = TxId::new(1);
        let doc_a = DocId::from_key("doc-a").unwrap();

        let node1 = EntityId::new(10);
        let node2 = EntityId::new(20);

        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            memfuse_core::Entity::new(node1, "n1", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            memfuse_core::Entity::new(node2, "n2", "Node"),
        )
        .await
        .unwrap();

        let edge = Edge::new(node1, node2, "derived").with_source_doc_id(doc_a);
        GraphIndex::add_edge(graph.as_ref(), tx1, edge)
            .await
            .unwrap();
        GraphIndex::commit(graph.as_ref(), tx1).await.unwrap();

        let engine = PathRAGEngine::new(graph.as_ref(), 3, 0.5);

        // Before invalidation, path is found
        let paths_before = engine.find_all_paths(node1);
        assert!(!paths_before.is_empty());

        // Invalidate doc A
        cascade_invalidate_edges_for_superseded_doc(&graph, doc_a, 5)
            .await
            .unwrap();

        // After invalidation, PathRAGEngine finds no paths through tombstoned edge
        let paths_after = engine.find_all_paths(node1);
        assert!(paths_after.is_empty());
    }

    #[tokio::test]
    async fn test_cascade_invalidation_nonexistent_doc_returns_zero_count() {
        let graph = std::sync::Arc::new(CsrGraph::new());
        let doc_unknown = DocId::from_key("doc-unknown").unwrap();

        let report = cascade_invalidate_edges_for_superseded_doc(&graph, doc_unknown, 10)
            .await
            .unwrap();

        assert_eq!(report.tombstoned_edge_count, 0);
        assert!(report.affected_node_ids.is_empty());
    }

    #[tokio::test]
    async fn test_cascade_invalidation_max_wal_seq() {
        let graph = std::sync::Arc::new(CsrGraph::new());
        let tx1 = TxId::new(1);
        let doc_a = DocId::from_key("doc-max-seq").unwrap();

        let node1 = EntityId::new(1001);
        let node2 = EntityId::new(1002);

        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            memfuse_core::Entity::new(node1, "n1", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            memfuse_core::Entity::new(node2, "n2", "Node"),
        )
        .await
        .unwrap();

        let edge = Edge::new(node1, node2, "rel").with_source_doc_id(doc_a);
        GraphIndex::add_edge(graph.as_ref(), tx1, edge)
            .await
            .unwrap();
        GraphIndex::commit(graph.as_ref(), tx1).await.unwrap();

        let report = cascade_invalidate_edges_for_superseded_doc(&graph, doc_a, u64::MAX)
            .await
            .unwrap();

        assert_eq!(report.tombstoned_edge_count, 1);
        assert_eq!(report.affected_node_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_cascade_invalidation_multiple_edges_same_doc() {
        let graph = std::sync::Arc::new(CsrGraph::new());
        let tx1 = TxId::new(1);
        let doc_multi = DocId::from_key("doc-multi").unwrap();

        let n1 = EntityId::new(10);
        let n2 = EntityId::new(20);
        let n3 = EntityId::new(30);

        for n in [n1, n2, n3] {
            GraphIndex::add_entity(
                graph.as_ref(),
                tx1,
                memfuse_core::Entity::new(n, format!("n{}", n.inner()), "Node"),
            )
            .await
            .unwrap();
        }

        let e1 = Edge::new(n1, n2, "rel1").with_source_doc_id(doc_multi);
        let e2 = Edge::new(n2, n3, "rel2").with_source_doc_id(doc_multi);

        GraphIndex::add_edge(graph.as_ref(), tx1, e1).await.unwrap();
        GraphIndex::add_edge(graph.as_ref(), tx1, e2).await.unwrap();
        GraphIndex::commit(graph.as_ref(), tx1).await.unwrap();

        let report = cascade_invalidate_edges_for_superseded_doc(&graph, doc_multi, 42)
            .await
            .unwrap();

        assert_eq!(report.tombstoned_edge_count, 2);
        assert_eq!(report.affected_node_ids.len(), 3);
        assert!(report.affected_node_ids.contains(&n1));
        assert!(report.affected_node_ids.contains(&n2));
        assert!(report.affected_node_ids.contains(&n3));
    }

    #[tokio::test]
    async fn test_cascade_invalidate_hyperedges_normal() {
        let graph = CsrGraph::new();
        let doc_a = DocId::from_key("doc-a").unwrap();

        const ROLE_SUBJECT: RoleId = RoleId::new(1);
        const ROLE_OBJECT: RoleId = RoleId::new(2);

        for i in 1..=5 {
            let id = HyperEdgeId::new(i);
            let bindings = vec![
                RoleBinding::new(ROLE_SUBJECT, EntityId::new(100 + i)),
                RoleBinding::new(ROLE_OBJECT, EntityId::new(200 + i)),
            ];
            let he = HyperEdge::new(id, EdgeType::Default, bindings, 1.0)
                .with_source_doc_id(Some(doc_a));
            graph.insert_hyperedge_direct(he);
        }

        let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
            .await
            .unwrap();

        assert_eq!(report.invalidated.len(), 5);
        assert!(report.deferred.is_empty());

        for i in 1..=5 {
            assert!(graph.get_hyperedge(HyperEdgeId::new(i)).is_none());
        }
    }

    #[tokio::test]
    async fn test_cascade_invalidate_hyperedges_high_fanout() {
        let graph = CsrGraph::new();
        let doc_a = DocId::from_key("doc-a").unwrap();

        const ROLE_1: RoleId = RoleId::new(1);
        const ROLE_2: RoleId = RoleId::new(2);

        let total_count = 1_200;
        for i in 1..=total_count {
            let id = HyperEdgeId::new(i as u64);
            let bindings = vec![
                RoleBinding::new(ROLE_1, EntityId::new(1)),
                RoleBinding::new(ROLE_2, EntityId::new(2)),
            ];
            let he = HyperEdge::new(id, EdgeType::Default, bindings, 1.0)
                .with_source_doc_id(Some(doc_a));
            graph.insert_hyperedge_direct(he);
        }

        let start = std::time::Instant::now();
        let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
            .await
            .unwrap();
        let elapsed = start.elapsed();

        // Bounded latency assertion (< 2000ms in debug build)
        assert!(elapsed < std::time::Duration::from_millis(2000));

        assert_eq!(report.invalidated.len(), MAX_HYPEREDGE_CASCADE_FANOUT);
        assert_eq!(
            report.deferred.len(),
            total_count - MAX_HYPEREDGE_CASCADE_FANOUT
        );

        // Verify that candidate_ids remaining for doc_a equal the deferred count
        let remaining = graph.hyperedges_for_doc(doc_a);
        assert_eq!(remaining.len(), report.deferred.len());
    }

    #[tokio::test]
    async fn test_cascade_invalidate_hyperedges_idempotency() {
        let graph = CsrGraph::new();
        let doc_a = DocId::from_key("doc-a").unwrap();

        const ROLE_1: RoleId = RoleId::new(1);

        for i in 1..=3 {
            let id = HyperEdgeId::new(i);
            let bindings = vec![RoleBinding::new(ROLE_1, EntityId::new(i))];
            let he = HyperEdge::new(id, EdgeType::Default, bindings, 1.0)
                .with_source_doc_id(Some(doc_a));
            graph.insert_hyperedge_direct(he);
        }

        // First call
        let report1 = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
            .await
            .unwrap();
        assert_eq!(report1.invalidated.len(), 3);
        assert!(report1.deferred.is_empty());

        // Repeat call (idempotent)
        let report2 = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 3)
            .await
            .unwrap();
        assert!(report2.invalidated.is_empty());
        assert!(report2.deferred.is_empty());
    }

    #[tokio::test]
    async fn test_cascade_invalidate_hyperedges_atomic_role_bindings() {
        let graph = CsrGraph::new();
        let doc_a = DocId::from_key("doc-a").unwrap();

        let e1 = EntityId::new(10);
        let e2 = EntityId::new(20);
        let e3 = EntityId::new(30);
        let e4 = EntityId::new(40);

        const ROLE_BUYER: RoleId = RoleId::new(1);
        const ROLE_SELLER: RoleId = RoleId::new(2);
        const ROLE_ASSET: RoleId = RoleId::new(3);
        const ROLE_ESCROW: RoleId = RoleId::new(4);

        let he_id = HyperEdgeId::new(99);
        let bindings = vec![
            RoleBinding::new(ROLE_BUYER, e1),
            RoleBinding::new(ROLE_SELLER, e2),
            RoleBinding::new(ROLE_ASSET, e3),
            RoleBinding::new(ROLE_ESCROW, e4),
        ];

        let he =
            HyperEdge::new(he_id, EdgeType::Default, bindings, 1.0).with_source_doc_id(Some(doc_a));
        graph.insert_hyperedge_direct(he);

        // Before invalidation, all 4 entities link to the hyperedge
        for e in [e1, e2, e3, e4] {
            assert_eq!(graph.hyperedges_for_entity(e), vec![he_id]);
        }

        let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
            .await
            .unwrap();

        assert_eq!(report.invalidated, vec![he_id]);

        // After invalidation, querying any of the 4 entities returns empty (atomically invalidated)
        for e in [e1, e2, e3, e4] {
            assert!(graph.hyperedges_for_entity(e).is_empty());
        }
    }

    #[tokio::test]
    async fn test_cascade_invalidate_hyperedges_nonexistent_doc() {
        let graph = CsrGraph::new();
        let doc_unknown = DocId::from_key("doc-unknown").unwrap();

        let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_unknown, 10)
            .await
            .unwrap();

        assert!(report.invalidated.is_empty());
        assert!(report.deferred.is_empty());
    }

    #[tokio::test]
    async fn test_cascade_invalidate_hyperedges_mixed_docs() {
        let graph = CsrGraph::new();
        let doc_a = DocId::from_key("doc-a").unwrap();
        let doc_b = DocId::from_key("doc-b").unwrap();

        let id_a1 = HyperEdgeId::new(1);
        let id_a2 = HyperEdgeId::new(2);
        let id_b1 = HyperEdgeId::new(3);

        const ROLE_R: RoleId = RoleId::new(1);

        let he_a1 = HyperEdge::new(
            id_a1,
            EdgeType::Default,
            vec![RoleBinding::new(ROLE_R, EntityId::new(1))],
            1.0,
        )
        .with_source_doc_id(Some(doc_a));
        let he_a2 = HyperEdge::new(
            id_a2,
            EdgeType::Default,
            vec![RoleBinding::new(ROLE_R, EntityId::new(2))],
            1.0,
        )
        .with_source_doc_id(Some(doc_a));
        let he_b1 = HyperEdge::new(
            id_b1,
            EdgeType::Default,
            vec![RoleBinding::new(ROLE_R, EntityId::new(3))],
            1.0,
        )
        .with_source_doc_id(Some(doc_b));

        graph.insert_hyperedge_direct(he_a1);
        graph.insert_hyperedge_direct(he_a2);
        graph.insert_hyperedge_direct(he_b1);

        let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, 2)
            .await
            .unwrap();

        assert_eq!(report.invalidated.len(), 2);
        assert!(graph.get_hyperedge(id_a1).is_none());
        assert!(graph.get_hyperedge(id_a2).is_none());

        // Doc B's hyperedge remains unaffected
        assert!(graph.get_hyperedge(id_b1).is_some());
        assert_eq!(graph.hyperedges_for_doc(doc_b), vec![id_b1]);
    }
}
