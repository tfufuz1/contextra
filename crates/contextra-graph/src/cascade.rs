// FILE-CONTEXT
// ZWECK: Cascading-Invalidation (Supersedes-Chunk -> Graph-Kanten-Tombstone)
// INVARIANTEN: Jede tombstonierte Kante erhaelt einen Provenienz-Eintrag mit WAL-Seq (INV-GRAPH-PROV-1).
// STAND: TS:2026-08-30T19:00:00Z

use crate::csr::CsrGraph;
use contextra_types::{DocId, EntityId, Result, TxId};

/// Maximum number of hyperedges processed synchronously per cascade invalidation run
/// to bound latency spikes. Any remaining hyperedges are returned in `deferred`.
pub const MAX_HYPEREDGE_CASCADE_FANOUT: usize = 1_000;

/// Default fan-out limit for hyperedge cascade invalidation per document (§6.8).
pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = MAX_HYPEREDGE_CASCADE_FANOUT;

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

/// LSM-Key-Präfix für die persistent cascade queue (§6.6 / §6.8).
pub const CASCADE_QUEUE_PREFIX: &str = "__graph:cascade_queue:";

/// Status tracking for a cascade invalidation ticket (§6.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CascadeStatus {
    /// Invalidation task is queued and pending.
    Pending,
    /// Invalidation task is currently processing.
    InFlight,
    /// Invalidation completed successfully with a deletion proof.
    Completed,
    /// Invalidation failed.
    Failed,
}

/// Proof of edge / hyperedge deletion during cascade invalidation (§6.8).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeletionProof {
    pub doc_id: DocId,
    pub tombstoned_count: usize,
    pub wal_seq: u64,
}

/// Ticket tracking persistent cascade invalidation status (§6.8).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CascadeTicket {
    pub id: u64,
    pub doc_id: DocId,
    pub status: CascadeStatus,
    pub proof: Option<DeletionProof>,
}

/// Garbage collection trait for sweeping orphaned hyperedge references (Anhang B §B.5.1.7).
pub trait GraphGarbageCollection {
    /// Sweeps orphaned hyperedges whose participants no longer exist or are fully tombstoned.
    fn sweep_orphans(&self, wal_tx: TxId) -> Result<usize>;
}

impl GraphGarbageCollection for CsrGraph {
    fn sweep_orphans(&self, wal_tx: TxId) -> Result<usize> {
        let candidate_ids = {
            let inner = self.inner_read();
            let mut orphaned = Vec::new();
            for (id, hedge) in inner.hyperedges.iter() {
                if hedge.tx_valid_to.is_none() {
                    let mut active_participant_count = 0usize;
                    for p in hedge.participants.iter() {
                        if let Some(&idx) = inner.id_map.get(&p.entity) {
                            if inner.entity_at(idx).is_some() {
                                active_participant_count += 1;
                            }
                        }
                    }
                    if active_participant_count < 2 {
                        orphaned.push(*id);
                    }
                }
            }
            orphaned
        };

        let mut swept_count = 0usize;
        for hid in candidate_ids {
            if self.tombstone_hyperedge(hid, wal_tx) {
                swept_count += 1;
            }
        }

        Ok(swept_count)
    }
}

/// Report summarizing the cascade invalidation of hyperedges derived from a superseded document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperedgeCascadeReport {
    /// Hyperedges tombstoned synchronously in this cascade run (up to MAX_HYPEREDGE_CASCADE_FANOUT).
    pub invalidated: Vec<crate::hyperedge::HyperEdgeId>,
    /// Hyperedges deferred for background processing due to fan-out limit.
    pub deferred: Vec<crate::hyperedge::HyperEdgeId>,
    /// Number of hyperedges queued for background processing (§6.6 / §6.8).
    pub queued_for_background: usize,
}

/// Alias for [`HyperedgeCascadeReport`] (§6.8 specification alignment).
pub type CascadeReport = HyperedgeCascadeReport;

/// Cascade invalidates hyperedges derived from `superseded_doc_id` with fan-out protection.
///
/// Up to `MAX_HYPEREDGE_CASCADE_FANOUT` hyperedges are tombstoned synchronously.
/// Any excess hyperedges are persisted to LSM storage under `__graph:cascade_queue:...`
/// and queued in `CsrGraph`'s cascade queue for deferred background processing.
///
/// INVARIANTE: Jede hier tombstonierte Hyperkante erhält einen Provenienz-Eintrag
/// mit der WAL-Sequenznummer dieses Aufrufs (INV-GRAPH-PROV-1).
/// Topologically sorts a set of hyperedge IDs in the parent-to-child DAG closure.
///
/// Every parent edge comes BEFORE its children.
/// Tie-breaking is done by `HyperEdgeId` in ascending order for deterministic execution.
///
/// # Crash Recovery Rationale
/// Nach einem Absturz darf nie ein Kind tombstoniert, aber sein Vorfahre noch lebendig sein,
/// weil der Vorfahre danach nicht mehr auffindbar wäre.
pub fn topological_sort_hyperedge_closure(
    graph: &CsrGraph,
    closure_nodes: &[crate::hyperedge::HyperEdgeId],
) -> Vec<crate::hyperedge::HyperEdgeId> {
    if closure_nodes.is_empty() {
        return Vec::new();
    }

    let node_set: std::collections::HashSet<_> = closure_nodes.iter().copied().collect();

    let mut in_degree = std::collections::HashMap::new();
    let mut children_map: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();

    for &node in &node_set {
        in_degree.entry(node).or_insert(0usize);
        let parents = graph.parent_hyperedges_of(node);
        let mut parents_in_closure = 0usize;
        for parent in parents {
            if node_set.contains(&parent) {
                parents_in_closure += 1;
                children_map.entry(parent).or_default().push(node);
            }
        }
        *in_degree.entry(node).or_default() = parents_in_closure;
    }

    let mut ready = std::collections::BinaryHeap::new();
    for (&node, &deg) in &in_degree {
        if deg == 0 {
            ready.push(std::cmp::Reverse(node));
        }
    }

    let mut topo_order = Vec::with_capacity(node_set.len());

    while let Some(std::cmp::Reverse(node)) = ready.pop() {
        topo_order.push(node);
        if let Some(children) = children_map.get(&node) {
            for &child in children {
                if let Some(deg) = in_degree.get_mut(&child) {
                    if *deg > 0 {
                        *deg -= 1;
                        if *deg == 0 {
                            ready.push(std::cmp::Reverse(child));
                        }
                    }
                }
            }
        }
    }

    if topo_order.len() < node_set.len() {
        let placed: std::collections::HashSet<_> = topo_order.iter().copied().collect();
        let mut remaining: Vec<_> = node_set.into_iter().filter(|n| !placed.contains(n)).collect();
        remaining.sort_unstable();
        topo_order.extend(remaining);
    }

    topo_order
}

/// Cascade invalidates hyperedges derived from `superseded_doc_id` with fan-out protection
/// and recursive super-edge ancestor invalidation.
///
/// Up to `MAX_HYPEREDGE_CASCADE_FANOUT` hyperedges (ancestors and document hyperedges combined)
/// are tombstoned synchronously in topological order (parents before children).
/// Any excess hyperedges are persisted to LSM storage under `__graph:cascade_queue:...`
/// and queued in `CsrGraph`'s cascade queue for deferred background processing.
///
/// INVARIANTE: Jede hier tombstonierte Hyperkante erhält einen Provenienz-Eintrag
/// mit der WAL-Sequenznummer dieses Aufrufs (INV-GRAPH-PROV-1).
pub async fn cascade_invalidate_hyperedges_for_superseded_doc(
    graph: &CsrGraph,
    superseded_doc_id: DocId,
    wal_seq: u64,
) -> Result<HyperedgeCascadeReport> {
    let wal_tx = TxId::new(wal_seq);
    let mut candidate_ids = graph.hyperedges_for_doc(superseded_doc_id);
    if candidate_ids.is_empty() {
        return Ok(HyperedgeCascadeReport {
            invalidated: Vec::new(),
            deferred: Vec::new(),
            queued_for_background: 0,
        });
    }
    candidate_ids.sort_unstable();
    candidate_ids.dedup();

    // 1. Upward closure search via DFS to collect doc hyperedges and all their non-tombstoned ancestors.
    let mut closure_nodes = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut on_stack = std::collections::HashSet::new();

    fn dfs_upward(
        graph: &CsrGraph,
        node: crate::hyperedge::HyperEdgeId,
        visited: &mut std::collections::HashSet<crate::hyperedge::HyperEdgeId>,
        on_stack: &mut std::collections::HashSet<crate::hyperedge::HyperEdgeId>,
        closure_nodes: &mut Vec<crate::hyperedge::HyperEdgeId>,
    ) {
        if !visited.insert(node) {
            return;
        }
        on_stack.insert(node);
        closure_nodes.push(node);

        for parent in graph.parent_hyperedges_of(node) {
            if on_stack.contains(&parent) {
                tracing::warn!(
                    child = %node.inner(),
                    parent = %parent.inner(),
                    "Cycle detected in hyperedge child-parent graph during cascade invalidation"
                );
            } else {
                dfs_upward(graph, parent, visited, on_stack, closure_nodes);
            }
        }

        on_stack.remove(&node);
    }

    for &doc_hid in &candidate_ids {
        dfs_upward(graph, doc_hid, &mut visited, &mut on_stack, &mut closure_nodes);
    }

    // 2. Topological sort: ensure parents come before children.
    let topo_order = topological_sort_hyperedge_closure(graph, &closure_nodes);

    // 3. Process up to MAX_HYPEREDGE_CASCADE_FANOUT synchronously; remaining excess goes to deferred queue.
    let mut invalidated = Vec::with_capacity(topo_order.len().min(MAX_HYPEREDGE_CASCADE_FANOUT));
    let mut deferred = Vec::new();

    for (idx, hyperedge_id) in topo_order.into_iter().enumerate() {
        if idx < MAX_HYPEREDGE_CASCADE_FANOUT {
            if graph.tombstone_hyperedge(hyperedge_id, wal_tx) {
                invalidated.push(hyperedge_id);
            }
        } else {
            deferred.push(hyperedge_id);
        }
    }

    if !deferred.is_empty() {
        if let Some(storage) = graph.storage() {
            for hid in &deferred {
                let key = format!(
                    "{CASCADE_QUEUE_PREFIX}{:016x}:{:016x}",
                    superseded_doc_id.inner(),
                    hid.inner()
                );
                storage.put(wal_tx, key.as_bytes(), &[]).await?;
            }
        }
        graph.enqueue_cascade_deferred(superseded_doc_id, &deferred);
    }

    let queued = deferred.len();
    Ok(HyperedgeCascadeReport {
        invalidated,
        deferred,
        queued_for_background: queued,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csr::EdgeType;
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
    use crate::path_rag::PathRAGEngine;
    use contextra_types::{Edge, Entity};
use contextra_ports::GraphIndex;

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
            contextra_types::Entity::new(node1, "n1", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            contextra_types::Entity::new(node2, "n2", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            contextra_types::Entity::new(node3, "n3", "Node"),
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
            contextra_types::Entity::new(node1, "n1", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            contextra_types::Entity::new(node2, "n2", "Node"),
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
            contextra_types::Entity::new(node1, "n1", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            contextra_types::Entity::new(node2, "n2", "Node"),
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
            contextra_types::Entity::new(node1, "n1", "Node"),
        )
        .await
        .unwrap();
        GraphIndex::add_entity(
            graph.as_ref(),
            tx1,
            contextra_types::Entity::new(node2, "n2", "Node"),
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
                contextra_types::Entity::new(n, format!("n{}", n.inner()), "Node"),
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

    #[tokio::test]
    async fn test_sweep_orphans_sweeps_degenerated_hyperedges() {
        let graph = CsrGraph::new();
        let tx = TxId::new(1);

        let e1 = EntityId::new(10);
        let e2 = EntityId::new(20);
        let e3 = EntityId::new(30);

        graph
            .add_entity(tx, Entity::new(e1, "E1", "Node"))
            .await
            .unwrap();
        graph
            .add_entity(tx, Entity::new(e2, "E2", "Node"))
            .await
            .unwrap();
        graph.commit(tx).await.unwrap();

        const ROLE: RoleId = RoleId::new(1);

        // Hyperedge 1: e1, e2 (both active -> valid)
        let h1 = HyperEdge::new(
            HyperEdgeId::new(1),
            EdgeType::Default,
            vec![RoleBinding::new(ROLE, e1), RoleBinding::new(ROLE, e2)],
            1.0,
        );

        // Hyperedge 2: e1, e3 (e3 non-existent -> active count = 1 < 2 -> orphan)
        let h2 = HyperEdge::new(
            HyperEdgeId::new(2),
            EdgeType::Default,
            vec![RoleBinding::new(ROLE, e1), RoleBinding::new(ROLE, e3)],
            1.0,
        );

        // Hyperedge 3: e3, e4 (both non-existent -> active count = 0 < 2 -> orphan)
        let h3 = HyperEdge::new(
            HyperEdgeId::new(3),
            EdgeType::Default,
            vec![
                RoleBinding::new(ROLE, e3),
                RoleBinding::new(ROLE, EntityId::new(40)),
            ],
            1.0,
        );

        graph.insert_hyperedge_direct(h1);
        graph.insert_hyperedge_direct(h2);
        graph.insert_hyperedge_direct(h3);

        let swept = graph.sweep_orphans(TxId::new(10)).unwrap();
        assert_eq!(
            swept, 2,
            "Must sweep exactly 2 orphaned hyperedges (h2 and h3)"
        );

        assert!(graph.get_hyperedge(HyperEdgeId::new(1)).is_some());
        assert!(graph.get_hyperedge(HyperEdgeId::new(2)).is_none());
        assert!(graph.get_hyperedge(HyperEdgeId::new(3)).is_none());
    }
}
