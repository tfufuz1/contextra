//! Integration tests for PPR strategy top-k result equivalence and tie-breaking by EntityId ascending.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_graph::csr::CsrGraph;
use contextra_ports::graph_index::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, PprAlgorithm, PprConfig, TxId};

#[tokio::test]
async fn test_ppr_strategy_top_k_equivalence_and_tie_breaking() {
    let graph = CsrGraph::new();

    // Create a 4-node ring graph: 1 -> 2 -> 3 -> 4 -> 1
    // and symmetric cross edges: 1 <-> 3
    let e1 = EntityId(1);
    let e2 = EntityId(2);
    let e3 = EntityId(3);
    let e4 = EntityId(4);

    let tx = TxId::new(1);

    graph
        .add_entity(tx, Entity::new(e1, "N1", "Node"))
        .await
        .expect("add_entity");
    graph
        .add_entity(tx, Entity::new(e2, "N2", "Node"))
        .await
        .expect("add_entity");
    graph
        .add_entity(tx, Entity::new(e3, "N3", "Node"))
        .await
        .expect("add_entity");
    graph
        .add_entity(tx, Entity::new(e4, "N4", "Node"))
        .await
        .expect("add_entity");

    let mut edge1 = Edge::new(e1, e2, "link");
    edge1.weight = 1.0;
    let mut edge2 = Edge::new(e2, e3, "link");
    edge2.weight = 1.0;
    let mut edge3 = Edge::new(e3, e4, "link");
    edge3.weight = 1.0;
    let mut edge4 = Edge::new(e4, e1, "link");
    edge4.weight = 1.0;
    let mut edge5 = Edge::new(e1, e3, "link");
    edge5.weight = 1.0;
    let mut edge6 = Edge::new(e3, e1, "link");
    edge6.weight = 1.0;

    graph.add_edge(tx, edge1).await.expect("add_edge");
    graph.add_edge(tx, edge2).await.expect("add_edge");
    graph.add_edge(tx, edge3).await.expect("add_edge");
    graph.add_edge(tx, edge4).await.expect("add_edge");
    graph.add_edge(tx, edge5).await.expect("add_edge");
    graph.add_edge(tx, edge6).await.expect("add_edge");

    graph.commit(tx).await.expect("commit");

    let cfg_fp = PprConfig {
        algorithm: PprAlgorithm::ForwardPush,
        convergence_epsilon: 1e-6,
        ..Default::default()
    };

    let cfg_pi = PprConfig {
        algorithm: PprAlgorithm::DensePowerIteration,
        convergence_epsilon: 1e-6,
        ..Default::default()
    };

    let seeds = vec![e1];

    let results_fp = graph
        .personalized_page_rank(&seeds, &cfg_fp)
        .await
        .expect("ppr fp");
    let results_pi = graph
        .personalized_page_rank(&seeds, &cfg_pi)
        .await
        .expect("ppr pi");

    assert!(!results_fp.is_empty(), "FP results non-empty");
    assert_eq!(results_fp.len(), results_pi.len(), "Same result count");

    // Verify top-k entity set match within epsilon
    for ((id_fp, score_fp), (id_pi, score_pi)) in results_fp.iter().zip(results_pi.iter()) {
        assert_eq!(id_fp, id_pi, "Entity order match");
        assert!(
            (score_fp - score_pi).abs() < 1e-2,
            "Scores match within epsilon for entity {:?}: FP {}, PI {}",
            id_fp,
            score_fp,
            score_pi
        );
    }

    // Verify tie-breaking: if two elements have identical score, EntityId MUST be strictly ascending
    for window in results_fp.windows(2) {
        let (e_a, score_a) = window[0];
        let (e_b, score_b) = window[1];

        if (score_a - score_b).abs() < f32::EPSILON {
            assert!(
                e_a < e_b,
                "Tie break failed: score_a == score_b but {:?} >= {:?}",
                e_a,
                e_b
            );
        }
    }
}
