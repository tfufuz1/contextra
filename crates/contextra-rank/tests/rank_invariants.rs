//! Property-based and deterministic invariant tests for contextra-rank.
//!
//! Specifically validates:
//! - `INV-RANK-DETERMINISM-1`: Identical input lists yield bit-exact identical rankings.
//! - `INV-RANK-MONOTONIC-1`: Isotonic calibration never inverts relative ranking orders.
//! - RRF Commutativity: Permuting ranked lists produces identical ranking outputs.
//! - Conformal Coverage: Empirical coverage rate on synthetic distributions >= 1 - alpha.
//! - DiBud Equivalence: Streaming fusion yields identical output to batch fusion.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
use contextra_rank::calibration::conformal::ConformalCalibrator;
use contextra_rank::calibration::{
    calibrate_score, AdaptiveConformalCalibrator, Calibrator, IsotonicCalibrator,
};
use contextra_rank::dibud::{fuse_exact_prefix, DiBudFusionState, FusionBudget};
use contextra_rank::fusion::{rrf_fuse, weighted_reciprocal_rank_fusion, ScoredItem, SearchResult};
use contextra_rank::DocId;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Invariant 1: INV-RANK-DETERMINISM-1
// ---------------------------------------------------------------------------

#[test]
fn test_inv_rank_determinism_1() {
    let list1 = vec![
        ScoredItem::new("doc_1".to_string(), 0.9),
        ScoredItem::new("doc_2".to_string(), 0.8),
        ScoredItem::new("doc_3".to_string(), 0.7),
    ];
    let list2 = vec![
        ScoredItem::new("doc_2".to_string(), 0.95),
        ScoredItem::new("doc_1".to_string(), 0.85),
        ScoredItem::new("doc_4".to_string(), 0.65),
    ];

    let inputs = vec![list1, list2];

    let baseline = rrf_fuse(&inputs, 60).unwrap();

    for _ in 0..100 {
        let run = rrf_fuse(&inputs, 60).unwrap();
        assert_eq!(
            baseline, run,
            "INV-RANK-DETERMINISM-1 violated: non-deterministic output"
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant 2: INV-RANK-MONOTONIC-1
// ---------------------------------------------------------------------------

#[test]
fn test_inv_rank_monotonic_1() {
    let mut calibrator = IsotonicCalibrator::new(10, 200);

    let training_data = vec![
        (0.1, false),
        (0.2, false),
        (0.25, true),
        (0.3, false),
        (0.4, false),
        (0.5, true),
        (0.6, false),
        (0.7, true),
        (0.8, true),
        (0.9, true),
        (0.95, true),
        (1.0, true),
    ];

    for (x, y) in training_data {
        calibrator.record_outcome(x, y);
    }
    calibrator.force_rebuild();

    let cal_container = Calibrator::isotonic(calibrator);

    let test_scores: Vec<f32> = (0..=100).map(|i| i as f32 / 100.0).collect();
    let mut last_prob = -1.0f32;

    for &score in &test_scores {
        let prob = calibrate_score(score, &cal_container).unwrap();
        assert!(
            prob >= last_prob - 1e-6,
            "INV-RANK-MONOTONIC-1 violated: score {} gave prob {}, previous was {}",
            score,
            prob,
            last_prob
        );
        last_prob = prob;
    }
}

proptest! {
    #[test]
    fn prop_inv_rank_monotonic_1(
        raw_scores in prop::collection::vec(0.0f32..10.0f32, 20..50),
        labels in prop::collection::vec(prop::bool::ANY, 20..50),
        eval_scores in prop::collection::vec(0.0f32..10.0f32, 10..30)
    ) {
        let n = raw_scores.len().min(labels.len());
        let mut calibrator = IsotonicCalibrator::new(5, 100);

        for i in 0..n {
            calibrator.record_outcome(raw_scores[i], labels[i]);
        }
        calibrator.force_rebuild();

        let mut sorted_eval = eval_scores.clone();
        sorted_eval.sort_by(|a, b| a.total_cmp(b));

        let cal_container = Calibrator::isotonic(calibrator);
        let mut prev = -1.0;
        for &raw in &sorted_eval {
            let prob = calibrate_score(raw, &cal_container).unwrap();
            prop_assert!(prob >= prev - 1e-6, "Monotonicity inversion detected!");
            prev = prob;
        }
    }
}

// ---------------------------------------------------------------------------
// Commutativity: Unweighted RRF is list-order invariant
// ---------------------------------------------------------------------------

#[test]
fn test_rrf_commutativity() {
    let list_a = vec![
        ScoredItem::new("doc_A".to_string(), 0.9),
        ScoredItem::new("doc_B".to_string(), 0.8),
        ScoredItem::new("doc_C".to_string(), 0.7),
    ];
    let list_b = vec![
        ScoredItem::new("doc_B".to_string(), 0.99),
        ScoredItem::new("doc_C".to_string(), 0.88),
        ScoredItem::new("doc_D".to_string(), 0.77),
    ];
    let list_c = vec![
        ScoredItem::new("doc_C".to_string(), 0.95),
        ScoredItem::new("doc_A".to_string(), 0.85),
        ScoredItem::new("doc_D".to_string(), 0.65),
    ];

    let order1 = vec![list_a.clone(), list_b.clone(), list_c.clone()];
    let order2 = vec![list_b.clone(), list_c.clone(), list_a.clone()];
    let order3 = vec![list_c.clone(), list_a.clone(), list_b.clone()];

    let fused1 = rrf_fuse(&order1, 60).unwrap();
    let fused2 = rrf_fuse(&order2, 60).unwrap();
    let fused3 = rrf_fuse(&order3, 60).unwrap();

    assert_eq!(
        fused1, fused2,
        "RRF commutativity failed between order1 and order2"
    );
    assert_eq!(
        fused2, fused3,
        "RRF commutativity failed between order2 and order3"
    );
}

proptest! {
    #[test]
    fn prop_rrf_commutativity(
        items1 in prop::collection::vec("[a-z0-9]{3}", 5..15),
        items2 in prop::collection::vec("[a-z0-9]{3}", 5..15),
    ) {
        let list1: Vec<ScoredItem> = items1.into_iter().enumerate().map(|(i, id)| ScoredItem::new(id, 100.0 - i as f32)).collect();
        let list2: Vec<ScoredItem> = items2.into_iter().enumerate().map(|(i, id)| ScoredItem::new(id, 100.0 - i as f32)).collect();

        let fused_ab = rrf_fuse(&[list1.clone(), list2.clone()], 60).unwrap();
        let fused_ba = rrf_fuse(&[list2, list1], 60).unwrap();

        prop_assert_eq!(fused_ab, fused_ba, "RRF commutativity failed under proptest permutation");
    }
}

// ---------------------------------------------------------------------------
// Conformal Coverage: Empirical coverage >= 1 - alpha
// ---------------------------------------------------------------------------

#[test]
fn test_conformal_empirical_coverage_guarantee() {
    let alpha = 0.10; // Target 90% coverage
    let mut calibrator = AdaptiveConformalCalibrator::new(alpha).unwrap();

    let mut rng_state: u64 = 42;
    let mut lcg = || -> f32 {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((rng_state >> 33) as f32) / ((1u64 << 31) as f32)
    };

    for _ in 0..1000 {
        let non_conformity = lcg().powf(2.0);
        calibrator.update(non_conformity, 1.0).unwrap();
    }

    let threshold = calibrator.threshold();
    let mut covered_count = 0;
    let test_samples = 2000;

    for _ in 0..test_samples {
        let non_conformity = lcg().powf(2.0);
        if non_conformity <= threshold {
            covered_count += 1;
        }
    }

    let empirical_coverage = (covered_count as f64) / (test_samples as f64);
    let target = 1.0 - (alpha as f64);
    assert!(
        empirical_coverage >= target - 0.03,
        "Conformal coverage empirical guarantee failed: empirical={}, target={}",
        empirical_coverage,
        target
    );
}

// ---------------------------------------------------------------------------
// Streaming Fusion: DiBud streaming yields identical output to batch
// ---------------------------------------------------------------------------

#[test]
fn test_dibud_streaming_vs_batch_equivalence() {
    let doc_ids = [
        DocId::from_key("doc_00001").unwrap(),
        DocId::from_key("doc_00002").unwrap(),
        DocId::from_key("doc_00003").unwrap(),
    ];

    let vec_ids = vec![doc_ids[0], doc_ids[1]];
    let text_ids = vec![doc_ids[1], doc_ids[2]];
    let graph_ids = vec![doc_ids[2], doc_ids[0]];

    let budget = FusionBudget {
        max_total_accesses: 10_000,
        min_certified_results: 10_000,
        edge_reinforcement_weight: 0.0,
        channel_weights: [1.0, 1.0, 1.0],
        edge_reinforcement_upper_bound: 0.0,
    };

    let state = DiBudFusionState::new(128);
    let mut v_iter = vec_ids.clone().into_iter();
    let mut t_iter = text_ids.clone().into_iter();
    let mut g_iter = graph_ids.clone().into_iter();

    let outcome = fuse_exact_prefix(
        state,
        &mut v_iter,
        &mut t_iter,
        &mut g_iter,
        |_| 0.0,
        &budget,
    )
    .expect("fuse_exact_prefix succeeds");

    let result_sets = vec![
        (
            "vector".to_string(),
            vec_ids
                .into_iter()
                .map(|id| SearchResult {
                    id: id.to_string(),
                    score: 1.0,
                    metadata: None,
                    matched_signals: vec![],
                    provenance: None,
                })
                .collect(),
            1.0,
        ),
        (
            "text".to_string(),
            text_ids
                .into_iter()
                .map(|id| SearchResult {
                    id: id.to_string(),
                    score: 1.0,
                    metadata: None,
                    matched_signals: vec![],
                    provenance: None,
                })
                .collect(),
            1.0,
        ),
        (
            "graph".to_string(),
            graph_ids
                .into_iter()
                .map(|id| SearchResult {
                    id: id.to_string(),
                    score: 1.0,
                    metadata: None,
                    matched_signals: vec![],
                    provenance: None,
                })
                .collect(),
            1.0,
        ),
    ];

    let rrf_full = weighted_reciprocal_rank_fusion(result_sets, 10_000);
    let rrf_doc_ids: Vec<DocId> = rrf_full
        .iter()
        .map(|r| DocId::from_key(&r.id).unwrap())
        .collect();

    assert_eq!(outcome.ranked, rrf_doc_ids);
}
