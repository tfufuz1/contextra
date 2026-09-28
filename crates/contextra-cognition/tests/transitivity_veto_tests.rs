// FILE-CONTEXT
// ZWECK: Tests für Transitivity Veto Inspektion, Orchestrierung & Budget-Steuerung.
// INVARIANTEN: P24-Lokalitätsnachweis; Zero-Panic; Budget-Grenzprüfung.
// STAND: TS:2026-09-27T00:00:00Z

use contextra_cognition::aggregation_phase::{AggregationConfig, AggregationNode};
use contextra_cognition::memory_consolidation::{run_consolidation_pass, ConsolidationConfig};
use contextra_cognition::transitivity_veto::{
    filter_candidates_with_transitivity_veto, validate_transitivity_veto,
};
use contextra_types::{DocId, EntityId};
use std::time::Instant;

#[test]
fn test_classic_transitivity_veto_case() {
    // A ≈ B, B ≈ C, A ≉ C
    // Node A: [1.0, 0.0, 0.0, 0.0]
    // Node B: [0.7071, 0.7071, 0.0, 0.0]  -> sim(A, B) = 0.7071
    // Node C: [0.0, 1.0, 0.0, 0.0]         -> sim(B, C) = 0.7071, sim(A, C) = 0.0
    let emb_a = vec![1.0f32, 0.0, 0.0, 0.0];
    let emb_b = vec![0.70710678f32, 0.70710678, 0.0, 0.0];
    let emb_c = vec![0.0f32, 1.0, 0.0, 0.0];

    let node_a = AggregationNode {
        entity: EntityId::new(1),
        embedding: emb_a.clone(),
        type_id: 0,
    };
    let node_b = AggregationNode {
        entity: EntityId::new(2),
        embedding: emb_b.clone(),
        type_id: 0,
    };
    let node_c = AggregationNode {
        entity: EntityId::new(3),
        embedding: emb_c.clone(),
        type_id: 0,
    };

    let threshold = 0.70f32;

    // Direct unit check on validate_transitivity_veto
    let is_valid = validate_transitivity_veto(&node_a, &node_b, &node_c, threshold);
    assert!(
        !is_valid,
        "Transitivity veto must trigger when A≈B and B≈C but A≉C"
    );

    // Test through orchestration
    let turns = vec![
        (DocId::new(1), emb_a),
        (DocId::new(2), emb_b),
        (DocId::new(3), emb_c),
    ];
    let agg_cfg = AggregationConfig::default();

    let filtered = filter_candidates_with_transitivity_veto(&turns, threshold, &agg_cfg);
    assert!(
        filtered.is_empty(),
        "Both pairs must be vetoed due to endpoint dissimilarity (A ≉ C)"
    );
}

#[test]
fn test_negative_case_high_similarity_throughout() {
    // A ≈ B ≈ C with high pairwise similarity across all pairs
    let emb_a = vec![1.0f32, 0.05, 0.0, 0.0];
    let emb_b = vec![1.0f32, 0.06, 0.0, 0.0];
    let emb_c = vec![1.0f32, 0.07, 0.0, 0.0];

    let node_a = AggregationNode {
        entity: EntityId::new(1),
        embedding: emb_a.clone(),
        type_id: 0,
    };
    let node_b = AggregationNode {
        entity: EntityId::new(2),
        embedding: emb_b.clone(),
        type_id: 0,
    };
    let node_c = AggregationNode {
        entity: EntityId::new(3),
        embedding: emb_c.clone(),
        type_id: 0,
    };

    let threshold = 0.95f32;

    let is_valid = validate_transitivity_veto(&node_a, &node_b, &node_c, threshold);
    assert!(
        is_valid,
        "No veto should be raised when all pairwise similarities exceed threshold"
    );

    let turns = vec![
        (DocId::new(1), emb_a),
        (DocId::new(2), emb_b),
        (DocId::new(3), emb_c),
    ];
    let agg_cfg = AggregationConfig::default();

    let filtered = filter_candidates_with_transitivity_veto(&turns, threshold, &agg_cfg);
    assert_eq!(
        filtered.len(),
        3,
        "All 3 pairs (A-B, A-C, B-C) must be retained when all similarities are high"
    );
}

#[test]
fn test_budget_boundary_exceeded_skips_veto_and_continues() {
    let emb_a = vec![1.0f32, 0.0, 0.0, 0.0];
    let emb_b = vec![0.70710678f32, 0.70710678, 0.0, 0.0];
    let emb_c = vec![0.0f32, 1.0, 0.0, 0.0];

    let turns = vec![
        (DocId::new(1), emb_a),
        (DocId::new(2), emb_b),
        (DocId::new(3), emb_c),
    ];

    // Set memory budget artificially to 0 MB (exceeded)
    let agg_cfg = AggregationConfig {
        max_compaction_peak_memory_mb: 0,
        ..Default::default()
    };

    let threshold = 0.70f32;
    let filtered = filter_candidates_with_transitivity_veto(&turns, threshold, &agg_cfg);

    // When budget is exceeded, transitivity veto check is skipped and raw candidate pairs are returned
    assert_eq!(
        filtered.len(),
        2,
        "When budget is exceeded, raw candidate pairs should be returned without failing consolidation"
    );

    // Consolidation pass with budget exceeded runs normally
    let config = ConsolidationConfig {
        min_turns_per_segment: 1,
        max_turns_per_segment: 20,
        segment_cohesion_threshold: 0.50,
        near_duplicate_cosine_threshold: 0.70,
        aggregation_config: agg_cfg,
    };

    let result = run_consolidation_pass(&turns, &config);
    assert_eq!(result.segments_created, 2);
    assert!(
        !result.duplicates_tombstoned.is_empty(),
        "Consolidation continues normally even when transitivity check budget is exceeded"
    );
}

#[test]
fn test_p24_locality_bounded_runtime_on_large_graph() {
    let mut turns = Vec::new();
    let dim = 128;

    // Create 100 distinct, non-duplicate background turns (orthogonal basis/sparse)
    for i in 1..=100 {
        let mut emb = vec![0.0f32; dim];
        emb[(i % dim) as usize] = 1.0;
        turns.push((DocId::new(i as u64), emb));
    }

    // Add 3 near-duplicate candidates at the end
    let emb_cand_a = vec![1.0f32; dim];
    let mut emb_cand_b = vec![1.0f32; dim];
    emb_cand_b[0] = 0.99;
    let mut emb_cand_c = vec![1.0f32; dim];
    emb_cand_c[0] = 0.98;

    turns.push((DocId::new(101), emb_cand_a));
    turns.push((DocId::new(102), emb_cand_b));
    turns.push((DocId::new(103), emb_cand_c));

    let agg_cfg = AggregationConfig::default();
    let start = Instant::now();

    let filtered = filter_candidates_with_transitivity_veto(&turns, 0.95, &agg_cfg);

    let duration = start.elapsed();

    // Verify candidates were evaluated and transitivity passed
    assert!(!filtered.is_empty());

    // P24 locality guarantee: runtime must stay extremely small (< 100ms) despite 103 nodes total,
    // because transitivity check only ran on the prefiltered candidates (N_cand = 3)
    assert!(
        duration.as_millis() < 100,
        "Transitivity check runtime should be bounded by prefiltered candidate set size, took: {:?}",
        duration
    );
}
