// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Compressed Sparse Row (CSR) Graph-Engine Kern.
// INVARIANTEN: Bitemporale Sichtbarkeit, lock-freie Lesepfade.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

// ANCHOR[contextra-graph:SRC-CSR-001]
//! CSR-Graph-Implementierung für Entity-Relation-Traversal.


// SECTION Csr Core Implementation
pub(crate) mod graph_index;
pub(crate) mod graph_persist;
pub(crate) mod graph_read;
pub(crate) mod graph_write;
pub(crate) mod inner;
pub(crate) mod path_graph;
pub(crate) mod types;
pub(crate) mod visibility;

#[cfg(test)]
mod tests;

pub use graph_write::CsrGraph;
pub use inner::{GraphInner, MemoryEstimate};
pub use types::{
    CsrEdge, CsrGraphConfig, EdgeType, PersistedEdgePayload, MAX_TRAVERSAL_HOPS, MAX_VISITED_NODES,
    SCORE_DECAY,
};
pub use visibility::{
    is_edge_visible, is_edge_visible_bitemporal, is_edge_visible_business, is_suspicious_tx_id,
};

// !SECTION Csr Core Implementation
