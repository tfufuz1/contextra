//! Graph-Sink Implementation (`CsrGraphSuperEdgeSink`) for LeanRAG Stage 3 Aggregation (§21.4).
//!
//! Provides a buffered [`SuperEdgeSink`] implementation against [`CsrGraph`].
//! Mutations (tombstones and new superedges) are held locally until [`SuperEdgeSink::commit`]
//! is called, executing a single atomic RCU publish on the underlying graph.

use crate::aggregation_phase::{SuperEdgeDraft, SuperEdgeSink};
use contextra_types::{ContextraError, Result, TxId};
use contextra_graph::csr::EdgeType;
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_graph::CsrGraph;

/// Buffered sink implementation writing synthetic superedges and tombstone requests to [`CsrGraph`].
///
/// # Invariant
/// Assumes a single active writer per `CsrGraph` during consolidation (enforced externally via
/// `ConsolidationLockGuard` in `consolidation_executor.rs`).
pub struct CsrGraphSuperEdgeSink<'a> {
    graph: &'a CsrGraph,
    wal_tx: TxId,
    pending_tombstones: Vec<HyperEdgeId>,
    pending_edges: Vec<HyperEdge>,
}

impl<'a> CsrGraphSuperEdgeSink<'a> {
    /// Creates a new `CsrGraphSuperEdgeSink` instance attached to the given graph and WAL transaction.
    pub fn new(graph: &'a CsrGraph, wal_tx: TxId) -> Self {
        Self {
            graph,
            wal_tx,
            pending_tombstones: Vec::new(),
            pending_edges: Vec::new(),
        }
    }
}

impl<'a> SuperEdgeSink for CsrGraphSuperEdgeSink<'a> {
    fn tombstone_edge(&mut self, id: HyperEdgeId) -> Result<()> {
        self.pending_tombstones.push(id);
        Ok(())
    }

    /// Allocates a provisional ID and buffers a synthetic superedge draft.
    ///
    /// # Role Binding Simplification
    /// Synthesizes generic role bindings using [`RoleId::new(0)`] for each participant entity,
    /// as `SuperEdgeDraft` participant lists do not carry explicit semantic role mappings and
    /// synthetic superedges do not enforce role interner constraints.
    fn write_super_edge(&mut self, draft: SuperEdgeDraft) -> Result<HyperEdgeId> {
        let graph_max_id = self.graph.max_hyperedge_id();
        let pending_max_id = self.pending_edges.iter().map(|e| e.id.inner()).max().unwrap_or(0);
        let next_id = HyperEdgeId::new(graph_max_id.max(pending_max_id) + 1);

        let participants: Vec<RoleBinding> = draft
            .participants
            .iter()
            .copied()
            .map(|entity| RoleBinding::new(RoleId::new(0), entity))
            .collect();

        let hyperedge = HyperEdge::new(next_id, EdgeType::Default, participants, draft.weight)
            .with_source_doc_id(None)
            .with_child_edge_ids(draft.child_edge_ids);

        self.pending_edges.push(hyperedge);
        Ok(next_id)
    }

    /// Atomically applies all buffered tombstone requests and new superedges in a single batch.
    fn commit(&mut self) -> Result<()> {
        let pending_edges = std::mem::take(&mut self.pending_edges);
        let result = self.graph.commit_super_edge_batch(
            &self.pending_tombstones,
            pending_edges,
            self.wal_tx,
        );

        if let Err(e) = result {
            return Err(ContextraError::from(e));
        }

        self.pending_tombstones.clear();
        Ok(())
    }
}
