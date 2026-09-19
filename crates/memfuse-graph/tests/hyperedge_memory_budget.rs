use memfuse_core::EntityId;
use memfuse_graph::csr::{CsrGraph, EdgeType};
use memfuse_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

#[test]
fn test_capacity_based_memory_budgeting() {
    let graph = CsrGraph::new();

    let initial_est = graph.estimate_memory(false);
    let initial_bytes = graph.estimate_memory_bytes();
    assert_eq!(initial_est.total_bytes(), initial_bytes);

    // Insert entities and hyperedges
    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);

    for i in 1..=100 {
        let id = HyperEdgeId::new(i);
        let bindings = vec![
            RoleBinding::new(ROLE_1, EntityId::new(i * 2)),
            RoleBinding::new(ROLE_2, EntityId::new(i * 2 + 1)),
        ];
        let he = HyperEdge::new(id, EdgeType::Default, bindings, 1.0);
        graph.insert_hyperedge_direct(he);
    }

    let loaded_est = graph.estimate_memory(false);
    assert!(
        loaded_est.shared_payload_bytes > 0,
        "Hyperedges shared payload bytes must be > 0"
    );
    assert!(
        loaded_est.private_bytes > initial_est.private_bytes,
        "Private structural memory must increase with elements"
    );

    // Presized estimate calculates exact len bytes
    let presized_est = graph.estimate_memory(true);
    assert!(
        presized_est.private_bytes <= loaded_est.private_bytes,
        "Presized private bytes must be <= capacity-based private bytes"
    );

    // Compaction peak memory is residence total_bytes + presized rebuild private_bytes
    let peak_bytes = graph.estimate_compaction_peak_bytes();
    assert_eq!(
        peak_bytes,
        loaded_est.total_bytes() + presized_est.private_bytes
    );
    assert!(
        peak_bytes > loaded_est.total_bytes(),
        "Compaction peak must exceed current residence bytes"
    );
}
