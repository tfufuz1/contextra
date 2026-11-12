// SECTION Lib Core Implementation
#![deny(dead_code)]
//! Contextra Graph — CSR-Graph for Entity-Relation Traversal & Session DAG.
//!
//! This crate provides the graph signal (Signal 3) for the 5-Signal Fusion
//! architecture (optional: Edge-Reinforcement-Signal hinter `edge-reinforcement-learning`-Flag).
//! It implements a Compressed Sparse Row (CSR) graph for
//! memory-efficient BFS traversal with score-decay, and a Session-DAG for
//! agent state branching (Contextra Session-DAG Pattern).
//!
//! # Architecture Role (Triebwerk — Layer 1)
//!
//! Peer to `contextra-store` and `contextra-index`. Provides the `GraphIndex`
//! trait implementation via [`CsrGraph`] and conversation branching via [`SessionBranchTree`].
//!
//! # Concurrency and Lock Discipline
//!
//! - Lock Hierarchy in [`CsrGraph`]: Locks are managed internally via `parking_lot::RwLock`.
//!   `inner` holds graph topology (offsets, targets, weights, maps, pending edges). Lock scopes
//!   are minimal and strictly contained within single methods without holding locks across
//!   `.await` points.
//! - Lock Hierarchy in [`SessionBranchTree`]: Synchronizes `nodes`, `edges`, and `active_head`
//!   independently using `parking_lot::RwLock`. When acquiring multiple locks concurrently,
//!   `nodes` MUST be acquired before `edges` or `active_head` to prevent deadlocks. No locks are
//!   held across `.await` storage calls.

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Crate-Wurzel und öffentliche Schnittstellen von contextra-graph.
// INVARIANTEN: #![forbid(unsafe_code)], Zero-Panic Invariante.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

// INVARIANT: CSR-Graph für 5-Signal Fusion (WP-6.1)

#![forbid(unsafe_code)]
#![deny(clippy::await_holding_lock, clippy::await_holding_invalid_type)]

// REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-10T19:23:40Z) (SESSION: bd6ff800)
// PRÜFER-KONTEXT: FRESH - Verified zero-unsafe invariant, zero unhandled panics, AGT-GRAPH-001 TxId origin assertions, and 133/133 tests green.

#[cfg(feature = "apprh-diffusion")]
pub mod apprh;
pub mod arc_slice;
pub mod cascade;
pub mod community;
#[cfg(any(feature = "types-compat", feature = "ports-compat"))]
pub mod compat;
pub mod consistency_enforcement;
pub mod csr;
#[cfg(feature = "edge-reinforcement-learning")]
pub mod edge_reinforcement;
#[cfg(feature = "edge-reinforcement-learning")]
pub mod edge_reinforcement_buffer;
pub mod entity_extraction;
pub mod error;
pub mod hyperedge;
pub mod hyperedge_suggest;
pub mod path_rag;
#[cfg(feature = "graph-connectivity-health")]
pub mod percolation;
pub mod ppr;
pub mod ppr_stream;
pub mod provenance;
pub mod provider;
pub mod purger;
#[cfg(feature = "experimental-graph")]
pub mod session_dag;
pub mod test_doubles;
pub mod types;
pub mod tl_hfd;
pub mod storage;
pub mod graph_view;
pub mod text_gen;

#[cfg(feature = "apprh-diffusion")]
pub use apprh::{
    apprh_local, forward_push_apprh, shadow_compare_forward_push_vs_apprh, ApprhError,
    ApprhFlipGate, ApprhMode, ApprhParams, ApprhSelector, ApprhSelectorConfig,
    ApprhShadowComparison, DefaultApprhFlipGate,
};
pub use arc_slice::ArcSlice;
pub use cascade::{
    cascade_invalidate_edges_for_superseded_doc, cascade_invalidate_hyperedges_for_superseded_doc,
    CascadeDeletionRecord, CascadeInvalidationReport, CascadeReport, CascadeStatus, CascadeTicket,
    GraphGarbageCollection, HyperedgeCascadeReport, MAX_HYPEREDGE_CASCADE_FANOUT,
};
pub use community::{
    detect_communities, CommunityAssignment, CommunityDetectionConfig, StarExpansionIterator,
    VirtualHyperedgeNode,
};
pub use consistency_enforcement::{
    ConflictPattern, ConsistencyEnforcer, ContradictionDetector, EdgeAssertion, EdgeId,
    ExactPredicateConflictDetector,
};
pub use csr::CsrGraph;
#[cfg(feature = "edge-reinforcement-learning")]
pub use edge_reinforcement::{
    apply_cooccurrence_reinforcement, apply_traversal_reinforcement, apply_weight_normalization,
    compute_edge_weight, EdgeReinforcementConfig,
};
#[cfg(feature = "edge-reinforcement-learning")]
pub use edge_reinforcement_buffer::{CooccurrenceSignal, EdgeReinforcementBuffer, TraversalSignal};
pub use entity_extraction::{
    ingest_extracted_relations, EntityExtractor, ExtractedRelation, RuleBasedEntityExtractor,
};
pub use error::GraphError as Error;
pub use error::{GraphError, GraphMutationError, Result};
pub use hyperedge::{
    sort_dedup_entities, star_weight, ConsolidationNodesGuard, HyperEdge, HyperEdgeId,
    HyperEdgeView, RoleBinding, RoleId, RoleInterner, HYPEREDGE_BY_ENTITY_PREFIX, HYPEREDGE_PREFIX,
};
pub use hyperedge_suggest::{
    compute_co_occurrence_candidates, validate_candidates_with_llm, HyperEdgeCandidate,
    HyperedgeSuggestError, ValidatedHyperEdgeCandidate, MAX_RELATE_PARTICIPANTS,
};
pub use path_rag::{GraphPath, PathGraph, PathRAGConfig, PathRAGEngine};
#[cfg(feature = "k-path-diffusion")]
pub use path_rag::{KPathConfig, KPathDiffusion, KPathResult};
#[cfg(feature = "graph-connectivity-health")]
pub use percolation::{
    compute_percolation_health, find_rebonding_candidates, should_trigger_rebonding,
    PercolationConfig,
};
#[cfg(feature = "apprh-diffusion")]
pub use ppr::compute_ppr_with_apprh_selector;
pub use ppr::{compute_ppr, compute_ppr_at, DeletedView, PprContext};
pub use ppr_stream::{PprCandidateStream, DEFAULT_PPR_STREAM_BATCH_SIZE};
pub use provenance::{DocEdgeIndex, EdgeProvenance};
#[cfg(any(feature = "types-compat", feature = "ports-compat"))]
pub use provider::{
    compute_pagerank, detect_communities_on_provider, GraphProvider, InMemoryGraphProvider,
};
#[cfg(feature = "ports-compat")]
pub use purger::{graph_purger, GraphPurger};
#[cfg(feature = "experimental-graph")]
pub use session_dag::{
    AgentStateNode, DagEdge, NodeIdx, NodesGuard, NodesWriteGuard, SessionBranchTree,
};
pub use tl_hfd::{
    shadow_compare_forward_push_vs_tl_hfd, tl_hfd_local, DefaultFlipGate, ShadowComparison,
    ShadowDiscrepancyReport, TlHfdError, TlHfdFlipGate, TlHfdParams, MIN_AGREEMENT_THRESHOLD,
    MIN_SHADOW_SAMPLES,
};

/// Extension trait for [`contextra_ports::GraphIndex`] providing entity removal functionality.
// ANCHOR[contextra-graph:SRC-LIB-001]
#[cfg(feature = "ports-compat")]
pub trait GraphIndexExt: contextra_ports::GraphIndex {
    /// Removes an entity node and all its incident (outgoing and incoming) edges from the graph.
    fn remove_entity<'a>(
        &'a self,
        tx: contextra_types::TxId,
        entity: EntityId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>>;
}

// !SECTION Lib Core Implementation

pub use storage::{GraphFuture, GraphStorage, InMemoryGraphStorage};
pub use graph_view::GraphView;
pub use text_gen::TextGenerator;
pub use types::{EntityId, PprAlgorithm, PprConfig};
