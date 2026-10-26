//! Integration tests for PPR Cost Model Strategy Selection.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_graph::ppr::cost::{estimate_ppr_cost, GraphStats, PprCostEstimate};
use contextra_types::{PprAlgorithm, PprConfig};

#[test]
fn test_ppr_cost_model_selection_sparse_vs_dense() {
    let config = PprConfig::default();

    // 1. Small/sparse graph: Power iteration cost (100 * (10 + 5) = 1500) < ForwardPush cost (6,666,666)
    let small_stats = GraphStats::new(5, 10);
    let (est_small, algo_small) = estimate_ppr_cost(Some(&small_stats), 2, &config);
    assert_eq!(algo_small, PprAlgorithm::DensePowerIteration);
    assert!(est_small.power_iteration < est_small.forward_push);

    // 2. Large graph with high edge count: ForwardPush cost (6,666,666) < Power iteration cost (100 * (500,000 + 50,000) = 55,000,000)
    let large_stats = GraphStats::new(50_000, 500_000);
    let (est_large, algo_large) = estimate_ppr_cost(Some(&large_stats), 2, &config);
    assert_eq!(algo_large, PprAlgorithm::ForwardPush);
    assert!(est_large.forward_push < est_large.power_iteration);
}

#[test]
fn test_ppr_cost_model_tie_break() {
    // When forward_push == power_iteration, ForwardPush MUST be chosen deterministically
    let est = PprCostEstimate {
        forward_push: 5000.0,
        power_iteration: 5000.0,
    };
    assert_eq!(est.cheapest_strategy(), PprAlgorithm::ForwardPush);
}

#[test]
fn test_ppr_cost_model_fallback_when_stats_missing() {
    let config = PprConfig::default();

    // Stats missing (None) and seed_count <= 100 -> ForwardPush fallback
    let (_, algo_fallback_small) = estimate_ppr_cost(None, 50, &config);
    assert_eq!(algo_fallback_small, PprAlgorithm::ForwardPush);

    // Stats missing (None) and seed_count > 100 -> DensePowerIteration fallback
    let (_, algo_fallback_large) = estimate_ppr_cost(None, 150, &config);
    assert_eq!(algo_fallback_large, PprAlgorithm::DensePowerIteration);
}

#[test]
fn test_ppr_cost_model_forward_push_cost_monotonicity() {
    let config = PprConfig::default();
    let stats = GraphStats::new(100, 500);

    // Adding more seeds does not increase ForwardPush estimated cost
    let (est1, _) = estimate_ppr_cost(Some(&stats), 1, &config);
    let (est10, _) = estimate_ppr_cost(Some(&stats), 10, &config);

    assert_eq!(est1.forward_push, est10.forward_push);
}
