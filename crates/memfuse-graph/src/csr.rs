//! CSR-Graph-Implementierung für Entity-Relation-Traversal.

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
pub use types::{EdgeType, Edge, PersistedEdgePayload, CsrGraphConfig, SCORE_DECAY, MAX_TRAVERSAL_HOPS, MAX_VISITED_NODES};
pub(crate) use types::{EdgePayload, StagedEdgePayload, InternalIndex};
pub use inner::{GraphInner, MemoryEstimate};
pub(crate) use inner::InnerWriteGuard;
pub use visibility::{is_suspicious_tx_id, is_edge_visible, is_edge_visible_bitemporal, is_edge_visible_business};
pub use graph_persist::*;
pub use graph_read::*;
pub use graph_index::*;
pub use path_graph::*;
