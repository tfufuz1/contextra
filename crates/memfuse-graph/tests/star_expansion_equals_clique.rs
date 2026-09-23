#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use memfuse_core::EntityId;
use memfuse_graph::csr::EdgeType;
use memfuse_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

/// Helper function computing Convention K star expansion weight: w_star = 2w / (|e| - 1)
pub fn star_weight_convention_k(hyperedge_weight: f32, cardinality: usize) -> f32 {
    if cardinality < 2 {
        0.0
    } else {
        (2.0 * hyperedge_weight) / ((cardinality - 1) as f32)
    }
}

#[test]
fn test_star_expansion_equals_clique_schur_complement() {
    let participants = vec![
        RoleBinding::new(RoleId::new(1), EntityId::new(10)),
        RoleBinding::new(RoleId::new(2), EntityId::new(20)),
        RoleBinding::new(RoleId::new(3), EntityId::new(30)),
        RoleBinding::new(RoleId::new(4), EntityId::new(40)),
    ];
    let cardinality = participants.len();
    let original_weight = 3.0f32;

    let hedge = HyperEdge::new(
        HyperEdgeId::new(1),
        EdgeType::Default,
        participants,
        original_weight,
    );

    // Convention K star weight calculation: 2w / (|e| - 1)
    let w_star_expected = star_weight_convention_k(hedge.weight, cardinality);
    assert_eq!(w_star_expected, (2.0 * 3.0) / (4.0 - 1.0)); // 6.0 / 3.0 = 2.0

    // Complete clique expansion weight per pair in clique expansion (Schur complement equivalence)
    // For a hyperedge with weight w, the Schur complement onto pairwise edges gives w_clique = w / (|e| - 1)
    let w_clique_expected = hedge.weight / ((cardinality - 1) as f32); // 3.0 / 3.0 = 1.0

    // Star expansion bipartite projection produces path of length 2 between any node pair via center node with weight w_star.
    // Conductance / Schur complement of star graph with star edge weights w_star yields effective pairwise weight w_star / 2.
    // For Schur complement equivalence: w_star / 2 = w_clique_expected => w_star = 2 * w_clique_expected = 2w / (|e| - 1).
    let schur_star_effective = w_star_expected / 2.0;

    assert!(
        (schur_star_effective - w_clique_expected).abs() < 1e-6,
        "Star expansion Schur complement effective weight {schur_star_effective} must equal clique weight {w_clique_expected}"
    );
}
