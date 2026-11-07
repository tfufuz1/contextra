//! # Graph-Senke für Hyperkanten (`graph_sink.rs`)
//!
//! Stellt eine gepufferte Senke ([`CsrGraphSuperEdgeSink`]) für die LeanRAG Stage 3 Aggregation
//! (§21.4) bereit. Tombstone-Markierungen und neu synthetisierte Hyperkanten werden lokal gesammelt
//! und erst beim Aufruf von `commit()` atomar via RCU (Read-Copy-Update) in den unterliegenden
//! [`CsrGraph`] publiziert.
//!
//! ## Ring-Zugehörigkeit
//! Teil der Kognitionsschicht (Ring 3), greift auf `contextra-graph` (Ring 0) zu.
//!
//! ## Invarianten
//! - **Single Writer:** Es wird von einem einzelnen aktiven Schreiber während der Konsolidierung
//!   ausgegangen (extern garantiert durch `ConsolidationNodesGuard`).
//! - **Atomare Batch-Veröffentlichung:** Alle Änderungen werden gemeinsam im Graphen publiziert.
//! - **Generische Rollenbindung:** Bei der Erzeugung synthetischer Hyperkanten wird ein Standard-Rollen-Binding
//!   mit `RoleId::new(0)` verwendet.

use crate::aggregation_phase::{SuperEdgeDraft, SuperEdgeSink};
use crate::graph_local::csr::EdgeType;
use crate::graph_local::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use crate::graph_local::CsrGraph;
use crate::types_local::{ContextraError, Result, TxId};

/// Buffered sink implementation writing synthetic superedges and invalidation requests to [`CsrGraph`].
///
/// # Invariant
/// Assumes a single active writer per `CsrGraph` during consolidation (enforced externally via
/// `ConsolidationLockGuard` in `consolidation_executor.rs`).
// ANCHOR[contextra-cognition:SINK-001]
/// Gepufferte Senke zum Schreiben synthetischer Hyperkanten und Tombstone-Anfragen in den CsrGraph.
pub struct CsrGraphSuperEdgeSink<'a> {
    graph: &'a CsrGraph,
    wal_tx: TxId,
    pending_invalidations: Vec<HyperEdgeId>,
    pending_edges: Vec<HyperEdge>,
}

impl<'a> CsrGraphSuperEdgeSink<'a> {
    /// Creates a new `CsrGraphSuperEdgeSink` instance attached to the given graph and WAL transaction.
    // ANCHOR[contextra-cognition:SINK-002]
    /// Erstellt eine neue CsrGraphSuperEdgeSink gebunden an den Graphen und die WAL-Transaktion.
    pub fn new(graph: &'a CsrGraph, wal_tx: TxId) -> Self {
        Self {
            graph,
            wal_tx,
            pending_invalidations: Vec::new(),
            pending_edges: Vec::new(),
        }
    }
}

impl<'a> SuperEdgeSink for CsrGraphSuperEdgeSink<'a> {
    fn invalidate_edge(&mut self, id: HyperEdgeId) -> Result<()> {
        self.pending_invalidations.push(id);
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
        let pending_max_id = self
            .pending_edges
            .iter()
            .map(|e| e.id.inner())
            .max()
            .unwrap_or(0);
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

    /// Atomically applies all buffered invalidation requests and new superedges in a single batch.
    fn commit(&mut self) -> Result<()> {
        let pending_edges = std::mem::take(&mut self.pending_edges);
        let result = self.graph.commit_super_edge_batch(
            &self.pending_invalidations,
            pending_edges,
            self.wal_tx,
        );

        if let Err(e) = result {
            return Err(ContextraError::from(e));
        }

        self.pending_invalidations.clear();
        Ok(())
    }
}
