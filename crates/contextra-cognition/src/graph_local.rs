//! Local abstractions for graph components to decouple `contextra-cognition`.

pub use contextra_graph as graph;
pub use graph::cascade::CascadeInvalidationReport;
pub use graph::csr::EdgeType;
pub use graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
pub use graph::percolation::PercolationConfig;
pub use graph::{detect_communities, CommunityDetectionConfig, CsrGraph};
