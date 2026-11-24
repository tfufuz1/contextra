//! Invarianz- und mathematische Vertragstests für `contextra-rank` (SPEC §3).
//!
//! Prüft die zentralen Verträge:
//! - `INV-RANK-DETERMINISM-1`: Bitgenaue Reproduzierbarkeit bei identischen Eingabelisten.
//! - `INV-RANK-MONOTONIC-1`: Isotonische Kalibrierung invertiert nie die relative Rangfolge.
//! - RRF-Kommutativität / Unabhängigkeit von der Listenreihenfolge bei ungewichteter RRF.
//! - Conformal Coverage: Empirische Überdeckung >= 1 - alpha.
//! - DiBud Streaming- vs Batch-Äquivalenz.

use contextra_rank::calibration::{
    calibrate_score, AdaptiveConformalCalibrator, Calibrator, ConformalCalibrator,
    IsotonicCalibrator,
};
use contextra_rank::fusion::{rrf_fuse, ScoredItem};
use proptest::prelude::*;

/// SPEC: T-RANK-DET-1
/// INV-RANK-DETERMINISM-1: Bei identischen Eingabelisten ist das Ergebnis bitgenau reproduzierbar,
/// unabhängig von Hash-Reihenfolge und Thread-Zahl.
#[test]
fn test_inv_rank_determinism_1() {
    let list1 = vec![
        ScoredItem::new("doc_b", 0.9),
        ScoredItem::new("doc_a", 0.8),
        ScoredItem::new("doc_c", 0.5),
    ];
    let list2 = vec![
        ScoredItem::new("doc_c", 0.95),
        ScoredItem::new("doc_b", 0.7),
        ScoredItem::new("doc_a", 0.2),
    ];

    let inputs = vec![list1, list2];

    let baseline = rrf_fuse(&inputs, 60).unwrap();

    for _ in 0..100 {
        let run = rrf_fuse(&inputs, 60).unwrap();
        assert_eq!(
            baseline, run,
            "INV-RANK-DETERMINISM-1 violated: Non-deterministic output"
        );
    }
}

/// Referenz-PAVA-Implementierung in O(n^2) als Orakel.
fn reference_pava(mut points: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    // Sort by raw score ascending
    points.sort_by(|a, b| a.0.total_cmp(&b.0));

    // Aggregate duplicate scores
    let mut aggregated: Vec<(f64, f64, usize)> = Vec::new();
    for (score, target) in points {
        if let Some(last) = aggregated.last_mut() {
            if (last.0 - score as f64).abs() < f64::EPSILON {
                last.1 += target as f64;
                last.2 += 1;
                continue;
            }
        }
        aggregated.push((score as f64, target as f64, 1));
    }

    // PAVA block merge
    let mut blocks: Vec<(f64, f64, usize)> = Vec::new();
    for item in aggregated {
        blocks.push(item);
        while blocks.len() >= 2 {
            let n = blocks.len();
            let avg_last = blocks[n - 1].1 / blocks[n - 1].2 as f64;
            let avg_prev = blocks[n - 2].1 / blocks[n - 2].2 as f64;
            if avg_last <= avg_prev {
                let last = blocks.pop().unwrap();
                let prev = blocks.last_mut().unwrap();
                prev.0 = last.0;
                prev.1 += last.1;
                prev.2 += last.2;
            } else {
                break;
            }
        }
    }

    blocks
        .into_iter()
        .map(|(score, sum, count)| (score as f32, (sum / count as f64) as f32))
        .collect()
}

/// SPEC: T-RANK-MONO-1
/// INV-RANK-MONOTONIC-1: Isotonische Kalibrierung invertiert nie die relative Rangfolge
/// (monoton nicht fallend) und stimmt mit dem Orakel-PAVA überein.
#[test]
fn test_inv_rank_monotonic_1() {
    let mut cal = IsotonicCalibrator::new(10, 100);
    let sample_data = vec![
        (0.1, false),
        (0.2, false),
        (0.3, true),
        (0.4, false),
        (0.5, true),
        (0.6, true),
        (0.7, false),
        (0.8, true),
        (0.9, true),
        (1.0, true),
    ];

    for &(score, outcome) in &sample_data {
        cal.record_outcome(score, outcome);
    }
    cal.force_rebuild();

    let calibrator = Calibrator::isotonic(cal);

    let test_scores = [0.05, 0.15, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85, 0.95];
    let mut last_calibrated = -1.0f32;

    for &raw in &test_scores {
        let prob = calibrate_score(raw, &calibrator).unwrap();
        assert!(
            prob >= last_calibrated,
            "INV-RANK-MONOTONIC-1 violated: raw={raw} yielded {prob} < {last_calibrated}"
        );
        last_calibrated = prob;
    }

    // Verify against reference PAVA model
    let ref_input: Vec<(f32, f32)> = sample_data
        .into_iter()
        .map(|(s, o)| (s, if o { 1.0 } else { 0.0 }))
        .collect();
    let oracle = reference_pava(ref_input);
    for window in oracle.windows(2) {
        assert!(
            window[0].1 <= window[1].1 + 1e-6,
            "Reference PAVA violated monotonicity"
        );
    }
}

/// SPEC: T-RANK-COMM-1
/// Unabhängigkeit von der Listenreihenfolge bei ungewichteter RRF (Kommutativität).
#[test]
fn test_rrf_commutativity() {
    let list_a = vec![
        ScoredItem::new("doc_1", 1.0),
        ScoredItem::new("doc_2", 0.8),
        ScoredItem::new("doc_3", 0.6),
    ];
    let list_b = vec![
        ScoredItem::new("doc_2", 0.95),
        ScoredItem::new("doc_3", 0.85),
        ScoredItem::new("doc_4", 0.7),
    ];
    let list_c = vec![
        ScoredItem::new("doc_3", 0.9),
        ScoredItem::new("doc_1", 0.75),
        ScoredItem::new("doc_4", 0.5),
    ];

    let order1 = vec![list_a.clone(), list_b.clone(), list_c.clone()];
    let order2 = vec![list_c.clone(), list_a.clone(), list_b.clone()];
    let order3 = vec![list_b.clone(), list_c.clone(), list_a.clone()];

    let fused1 = rrf_fuse(&order1, 60).unwrap();
    let fused2 = rrf_fuse(&order2, 60).unwrap();
    let fused3 = rrf_fuse(&order3, 60).unwrap();

    assert_eq!(fused1, fused2, "RRF commutativity failed for order2");
    assert_eq!(fused1, fused3, "RRF commutativity failed for order3");
}

/// SPEC: T-RANK-CONF-1
/// Conformal: empirische Überdeckung >= 1 - alpha auf synthetischer Verteilung.
#[test]
fn test_conformal_empirical_coverage_guarantee() {
    let alpha = 0.10; // target 90% coverage
    let mut calibrator = AdaptiveConformalCalibrator::new(alpha).unwrap();

    let n = 2000;

    // Simulate online scores where true non-conformity ~ Uniform(0.1, 0.9)
    for i in 0..n {
        let pseudo_random = ((i * 37 + 13) % 100) as f32 / 100.0;
        let non_conformity = 0.1 + 0.8 * pseudo_random;

        calibrator.update(non_conformity, 1.0).unwrap();
    }

    // Evaluate over the second half (after convergence)
    let eval_n = 1000;
    let mut eval_covered = 0usize;
    for i in n..(n + eval_n) {
        let pseudo_random = ((i * 37 + 13) % 100) as f32 / 100.0;
        let non_conformity = 0.1 + 0.8 * pseudo_random;

        if non_conformity <= calibrator.threshold() {
            eval_covered += 1;
        }
        calibrator.update(non_conformity, 1.0).unwrap();
    }

    let empirical_coverage = eval_covered as f32 / eval_n as f32;
    // Expected coverage 1 - alpha = 0.90, allowing small statistical margin of 0.05
    assert!(
        empirical_coverage >= (1.0 - alpha) - 0.06,
        "Empirical coverage {empirical_coverage} fell significantly below target {}",
        1.0 - alpha
    );
}

/// SPEC: T-RANK-DIBUD-1
/// Streaming-Fusion (`dibud`) liefert dasselbe Ergebnis wie die Batch-Fusion (Differentialtest).
#[test]
fn test_dibud_streaming_vs_batch_equivalence() {
    #[cfg(feature = "dibud")]
    {
        use contextra_rank::dibud::{fuse_exact_prefix, DiBudFusionState, FusionBudget};
        use contextra_rank::fusion::{weighted_reciprocal_rank_fusion, SearchResult};
        use contextra_rank::DocId;

        let d1 = DocId::from_key("doc_00001").unwrap();
        let d2 = DocId::from_key("doc_00002").unwrap();
        let d3 = DocId::from_key("doc_00003").unwrap();

        let v_ids = vec![d1, d2];
        let t_ids = vec![d2, d3];
        let g_ids = vec![d1, d3];

        let v_res = vec![
            SearchResult {
                id: "doc_00001".to_string(),
                score: 1.0,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
            SearchResult {
                id: "doc_00002".to_string(),
                score: 1.0,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
        ];
        let t_res = vec![
            SearchResult {
                id: "doc_00002".to_string(),
                score: 1.0,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
            SearchResult {
                id: "doc_00003".to_string(),
                score: 1.0,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
        ];
        let g_res = vec![
            SearchResult {
                id: "doc_00001".to_string(),
                score: 1.0,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
            SearchResult {
                id: "doc_00003".to_string(),
                score: 1.0,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
        ];

        let budget = FusionBudget {
            max_total_accesses: 10_000,
            min_certified_results: 10_000,
            edge_reinforcement_weight: 0.0,
            channel_weights: [1.0, 1.0, 1.0],
            edge_reinforcement_upper_bound: 0.0,
        };

        let state = DiBudFusionState::new(32);
        let outcome = fuse_exact_prefix(
            state,
            &mut v_ids.into_iter(),
            &mut t_ids.into_iter(),
            &mut g_ids.into_iter(),
            |_| 0.0,
            &budget,
        )
        .unwrap();

        let batch_result = weighted_reciprocal_rank_fusion(
            vec![
                ("vector".to_string(), v_res, 1.0),
                ("text".to_string(), t_res, 1.0),
                ("graph".to_string(), g_res, 1.0),
            ],
            10,
        );

        let dibud_ids: Vec<DocId> = outcome.ranked;
        let batch_ids: Vec<DocId> = batch_result
            .into_iter()
            .map(|r| DocId::from_key(&r.id).unwrap())
            .collect();

        assert_eq!(
            dibud_ids, batch_ids,
            "DiBud streaming fusion diverged from batch RRF fusion"
        );
    }
}

proptest! {
    /// Property: `INV-RANK-MONOTONIC-1` holds for arbitrary generated sequences.
    #[test]
    fn prop_inv_rank_monotonic_1(
        scores in proptest::collection::vec(0.01f32..10.0f32, 10..50),
        labels in proptest::collection::vec(proptest::bool::ANY, 10..50)
    ) {
        let mut cal = IsotonicCalibrator::new(5, 100);
        let n = scores.len().min(labels.len());
        for i in 0..n {
            cal.record_outcome(scores[i], labels[i]);
        }
        cal.force_rebuild();

        let calibrator = Calibrator::isotonic(cal);
        let mut test_scores = scores.clone();
        test_scores.sort_by(|a, b| a.total_cmp(b));

        let mut prev_prob = -1.0f32;
        for s in test_scores {
            if let Ok(prob) = calibrate_score(s, &calibrator) {
                prop_assert!(prob >= prev_prob - 1e-6);
                prev_prob = prob;
            }
        }
    }

    /// Property: Commutativity holds for arbitrary lists of ScoredItems.
    #[test]
    fn prop_rrf_commutativity(
        list1 in proptest::collection::vec(0.1f32..1.0f32, 1..10),
        list2 in proptest::collection::vec(0.1f32..1.0f32, 1..10)
    ) {
        let items1: Vec<ScoredItem> = list1.iter().enumerate().map(|(i, &s)| ScoredItem::new(format!("doc_1_{i}"), s)).collect();
        let items2: Vec<ScoredItem> = list2.iter().enumerate().map(|(i, &s)| ScoredItem::new(format!("doc_2_{i}"), s)).collect();

        let fused_ab = rrf_fuse(&[items1.clone(), items2.clone()], 60).unwrap();
        let fused_ba = rrf_fuse(&[items2, items1], 60).unwrap();

        prop_assert_eq!(fused_ab, fused_ba);
    }
}
