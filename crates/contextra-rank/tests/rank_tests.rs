#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Comprehensive unit test suite for `contextra-rank` crate modules.

#[cfg(test)]
mod tests {
    use contextra_rank::calibration::{IsotonicCalibrator, PlattScaler};
    use contextra_rank::drift::DriftDetector;
    use contextra_rank::fusion::{
        reciprocal_rank_fusion, BoundedTopK, ProvenanceBuilder, SearchResult,
    };
    use contextra_types::ConfigFingerprint;

    #[test]
    fn test_rrf_k_zero_boundary() {
        let prov = ProvenanceBuilder::new(0.0)
            .vector(0.9, 1, 1.0)
            .source_collection("col")
            .index_type("hnsw")
            .expected_total(1.0)
            .build();
        let contrib = prov
            .signal_contributions
            .get("vector")
            .expect("vector contribution present");
        assert_eq!(contrib.rrf_contribution, 1.0);
    }

    #[test]
    fn test_rrf_dual_signal_higher_than_single_signal() {
        let set1 = vec![SearchResult {
            id: "doc_both".to_string(),
            score: 0.99,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        }];
        let set2 = vec![
            SearchResult {
                id: "doc_both".to_string(),
                score: 0.95,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
            SearchResult {
                id: "doc_single".to_string(),
                score: 0.99,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
        ];

        let fused = reciprocal_rank_fusion(vec![set1, set2], 10);
        assert_eq!(fused.len(), 2);
        assert_eq!(fused[0].id, "doc_both");
        assert_eq!(fused[1].id, "doc_single");
        assert!(fused[0].score > fused[1].score);
    }

    #[test]
    fn test_bounded_top_k() {
        let mut top_k = BoundedTopK::new(2);
        top_k.push(10i32);
        top_k.push(20i32);
        top_k.push(5i32);
        let sorted = top_k.into_sorted_vec();
        assert_eq!(sorted, vec![5, 10]);
    }

    #[test]
    fn test_isotonic_calibrator_warmup() {
        let mut cal = IsotonicCalibrator::new(10, 100);
        for i in 0..9 {
            cal.record_outcome(i as f32 / 10.0, i > 4);
        }
        assert!(cal.calibrated_probability(0.5).is_none());

        cal.record_outcome(0.9, true);
        assert!(cal.calibrated_probability(0.5).is_some());
    }

    #[test]
    fn test_isotonic_p8_invalidation() {
        let mut cal = IsotonicCalibrator::new(5, 100);
        for i in 0..10 {
            cal.record_outcome(i as f32 / 10.0, i > 4);
        }
        let fp1 = ConfigFingerprint::new("model-1", "Q4", "prompt", 0.7);
        cal.invalidate_on_config_change(fp1.clone());
        assert_eq!(cal.observation_count(), 0);

        cal.record_outcome(0.5, true);
        cal.invalidate_on_config_change(fp1);
        assert_eq!(cal.observation_count(), 1);
    }

    #[test]
    fn test_platt_scaler_fit_and_predict() {
        let mut obs = Vec::new();
        for i in -10..=10 {
            let logit = i as f32 * 0.2;
            obs.push((logit, logit > 0.0));
        }

        let scaler = PlattScaler::fit(&obs);
        assert!(scaler.is_fitted());
        assert!(scaler.predict(1.0) > 0.7);
        assert!(scaler.predict(-1.0) < 0.3);
    }

    #[test]
    fn test_drift_detector() {
        let mut detector = DriftDetector::new(10, 0.1);
        detector.set_baseline(0.5);

        for _ in 0..10 {
            detector.observe(0.51);
        }
        assert_eq!(detector.overall_drift_status(), "stabil");

        for _ in 0..10 {
            detector.observe(0.9);
        }
        assert_eq!(detector.overall_drift_status(), "kritisch");
    }

    #[test]
    fn test_g_function_standalone_properties() {
        use contextra_rank::g;

        // (c) g(0) is exactly 0
        assert_eq!(g(0.0), 0.0);
        assert_eq!(g(-0.0), 0.0);

        // g(x) bounded towards 1 for x -> infinity
        assert!((g(1000.0) - 1.0).abs() < 0.01);
        assert!((g(1_000_000.0) - 1.0).abs() < 0.0001);

        // g(x) bounded towards -1 for x -> -infinity
        assert!((g(-1000.0) - (-1.0)).abs() < 0.01);

        // Non-finite float safety
        assert_eq!(g(f32::NAN), 0.0);
        assert_eq!(g(f32::INFINITY), 0.0);
        assert_eq!(g(f32::NEG_INFINITY), 0.0);
    }

    #[test]
    fn test_mrrf_determinism_baseline() {
        use contextra_rank::fusion::{
            weighted_reciprocal_rank_fusion_mrrf, weighted_reciprocal_rank_fusion_with_options,
            MetadataMergePriority, SearchResult, SignalCalibrationContext,
        };
        use contextra_rank::drift::DriftStatus;

        let res_a = vec![
            SearchResult {
                id: "doc1".to_string(),
                score: 0.9,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
            SearchResult {
                id: "doc2".to_string(),
                score: 0.8,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
        ];
        let res_b = vec![
            SearchResult {
                id: "doc2".to_string(),
                score: 0.85,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
            SearchResult {
                id: "doc1".to_string(),
                score: 0.75,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
        ];

        let sets = vec![
            ("signal_a".to_string(), res_a.clone(), 1.0),
            ("signal_b".to_string(), res_b.clone(), 1.0),
        ];

        // Baseline standard weighted RRF
        let standard_fused = weighted_reciprocal_rank_fusion_with_options(
            sets.clone(),
            10,
            MetadataMergePriority::default(),
            true,
            None,
        );

        // mRRF with beta = 0.0
        let mrrf_fused_beta0 = weighted_reciprocal_rank_fusion_mrrf(
            sets.clone(),
            10,
            MetadataMergePriority::default(),
            true,
            None,
            None,
            0.0,
        );

        // mRRF with beta = 1.0 but uncalibrated/unstable signal context (so m_s defaults to 0)
        let uncalibrated_ctxs = vec![
            SignalCalibrationContext::new("signal_a", false, DriftStatus::Stable { mean_shift: 0.0 }),
            SignalCalibrationContext::new("signal_b", true, DriftStatus::DriftDetected { mean_shift: 0.3, threshold: 0.1 }),
        ];
        let mrrf_fused_uncalibrated = weighted_reciprocal_rank_fusion_mrrf(
            sets,
            10,
            MetadataMergePriority::default(),
            true,
            None,
            Some(&uncalibrated_ctxs),
            1.0,
        );

        assert_eq!(standard_fused, mrrf_fused_beta0);
        assert_eq!(standard_fused, mrrf_fused_uncalibrated);
    }

    #[test]
    fn test_mrrf_margin_modulation_relative_weight_effect() {
        use contextra_rank::fusion::{
            modulate_and_renormalize_weights, SearchResult, SignalCalibrationContext,
        };
        use contextra_rank::drift::DriftStatus;

        // Signal A: Close tie (m_s = 0.80 - 0.79 = 0.01)
        let res_a = vec![
            SearchResult {
                id: "docA1".to_string(),
                score: 0.80,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
            SearchResult {
                id: "docA2".to_string(),
                score: 0.79,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
        ];

        // Signal B: Clear margin (m_s = 0.90 - 0.40 = 0.50)
        let res_b = vec![
            SearchResult {
                id: "docB1".to_string(),
                score: 0.90,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
            SearchResult {
                id: "docB2".to_string(),
                score: 0.40,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
        ];

        let sets = vec![
            ("signal_a".to_string(), res_a, 1.0),
            ("signal_b".to_string(), res_b, 1.0),
        ];

        let contexts = vec![
            SignalCalibrationContext::new("signal_a", true, DriftStatus::Stable { mean_shift: 0.0 }),
            SignalCalibrationContext::new("signal_b", true, DriftStatus::Stable { mean_shift: 0.0 }),
        ];

        let beta = 1.0;
        let modulated_weights = modulate_and_renormalize_weights(&sets, Some(&contexts), beta);

        let w_a = modulated_weights[0];
        let w_b = modulated_weights[1];

        // Verify total weight sum is preserved (sum = 2.0, within [0.999, 1.001] factor)
        let total_weight = w_a + w_b;
        assert!((total_weight - 2.0).abs() < 0.001, "Weight sum invariant violated: {total_weight}");

        // Signal B has a much larger margin, so after modulation and renormalization, w_B > w_A
        assert!(w_b > w_a, "Expected Signal B weight ({w_b}) to be strictly greater than Signal A weight ({w_a})");
    }
}
