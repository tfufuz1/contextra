//! Compile-time verification of all public re-exports from contextra-graph.
#[allow(unused_imports)]
use contextra_graph::*;

#[test]
fn test_api_surface_types() {
    let _ = CsrGraph::new();
}
