//! Local abstractions for graph components to decouple contextra-cognition.

pub use contextra_graph as graph;
pub use graph::cascade::{self, CascadeInvalidationReport};
pub use graph::cascade_invalidate_edges_for_superseded_doc;
pub use graph::csr::{self, EdgeType};
pub use graph::hyperedge::{self, HyperEdge, HyperEdgeId, RoleBinding, RoleId};
pub use graph::{detect_communities, CommunityDetectionConfig, CsrGraph};

#[cfg(feature = "graph-connectivity-health")]
pub use graph::percolation::PercolationConfig;
