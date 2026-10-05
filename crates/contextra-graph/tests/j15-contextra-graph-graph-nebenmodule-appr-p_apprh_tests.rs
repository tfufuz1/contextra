//! Integration tests for APPRH symbols: `apprh_diffusion`, `apprh_shadow_compare`, and `with_default_monitor`.

#![cfg(feature = "apprh-diffusion")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_graph::apprh::{ApprhParams, ApprhSelector, ApprhSelectorConfig};
use contextra_graph::csr::CsrGraph;
use contextra_graph::path_rag::PprParams;
use contextra_ports::{Edge, GraphIndex};
use contextra_types::{EntityId, TxId};

#[tokio::test]
async fn test_apprh_diffusion_and_shadow_compare_on_csr_graph() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    let e1 = EntityId::new(10);
    let e2 = EntityId::new(20);
    let e3 = EntityId::new(30);

    graph
        .add_edge(tx, Edge::new(e1, e2, "rel").with_weight(1.0))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(e2, e3, "rel").with_weight(1.0))
        .await
        .unwrap();

    let seeds = vec![e1];
    let apprh_params = ApprhParams::default();

    let diff_res = graph.apprh_diffusion(&seeds, &apprh_params).unwrap();
    assert!(!diff_res.is_empty());
    assert!(diff_res.contains_key(&e1));

    let ppr_params = PprParams::default();
    let comparison = graph
        .apprh_shadow_compare(&seeds, &ppr_params, &apprh_params, 5)
        .unwrap();

    assert!(!comparison.forward_push_results.is_empty());
    assert!(!comparison.apprh_results.is_empty());
}

#[test]
fn test_apprh_selector_with_default_monitor() {
    let config = ApprhSelectorConfig {
        sample_interval: 2,
        max_seeds: 50,
        shadow_top_k: 10,
    };

    let selector = ApprhSelector::with_default_monitor(config).unwrap();
    assert_eq!(selector.mode(), contextra_graph::apprh::ApprhMode::Shadow);
    assert_eq!(selector.monitor().snapshot().total_observations, 0);
}
