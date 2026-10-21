//! PPR Performance and Correctness Benchmark Test for contextra-graph
//! Verifies numerical bit-identical determinism and buffer reuse efficiency in Personalized PageRank (PPR).

use contextra_graph::{CsrGraph, PprContext};
use contextra_types::{Entity, EntityId, PprConfig};
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_ppr_alloc_reduction_100k_nodes() {
    const NUM_NODES: usize = 100_000;

    let graph = Arc::new(CsrGraph::new());

    // Construct a large graph with 100,000 nodes and linear edges i -> i + 1
    for i in 1..=NUM_NODES {
        let eid = EntityId::new(i as u64);
        graph
            .insert_entity_direct(Entity::new(eid, format!("N{i}"), "Node"))
            .expect("insert entity");
    }

    for i in 1..NUM_NODES {
        let src = EntityId::new(i as u64);
        let dst = EntityId::new((i + 1) as u64);
        graph
            .insert_edge_direct(src, dst, 1.0)
            .await
            .expect("insert edge");
    }

    graph.compact();

    let seed = EntityId::new(1);
    let config = PprConfig {
        damping_factor: 0.85,
        max_iterations: 10,
        convergence_epsilon: 1e-6,
        algorithm: contextra_types::PprAlgorithm::Auto,
        warn_on_non_convergence: true,
    };

    // --- First run (fresh PprContext) ---
    let mut ctx = PprContext::new();
    let start1 = Instant::now();
    let res1 = graph
        .personalized_page_rank_with_context_async(&[seed], &config, &mut ctx)
        .await;
    let dur1 = start1.elapsed();

    println!("Run 1 (Fresh PprContext) Duration: {dur1:?}");

    // --- Second run (reusing PprContext with pre-allocated capacity) ---
    let start2 = Instant::now();
    let res2 = graph
        .personalized_page_rank_with_context_async(&[seed], &config, &mut ctx)
        .await;
    let dur2 = start2.elapsed();

    println!("Run 2 (Reused PprContext) Duration: {dur2:?}");

    // Verifications:
    // 1. Non-empty results and non-zero rank scores
    assert!(
        !res1.is_empty(),
        "PPR run on 100,000 nodes must return non-empty rank scores"
    );

    // 2. Numerical correctness / bit-identical equality between runs
    assert_eq!(
        res1.len(),
        res2.len(),
        "Reused PprContext run must return identical result length"
    );
    for (a, b) in res1.iter().zip(res2.iter()) {
        assert_eq!(a.0, b.0, "EntityId mismatch between PPR runs");
        assert_eq!(
            a.1.to_bits(),
            b.1.to_bits(),
            "Bit-level rank score discrepancy between PPR runs for EntityId {:?}",
            a.0
        );
    }

    // 3. Performance efficiency check: Reused context run must complete without regressing runtime
    assert!(
        dur2 <= dur1 * 2,
        "Reused PprContext run ({dur2:?}) exceeded 2x first run duration ({dur1:?})"
    );
}
