//! Comprehensive unit test suite for `memfuse-rank` crate modules.

#[cfg(test)]
mod tests {
    use memfuse_rank::calibration::{IsotonicCalibrator, PlattScaler};
    use memfuse_rank::drift::DriftDetector;
    use memfuse_rank::fusion::{
        reciprocal_rank_fusion, BoundedTopK, ProvenanceBuilder, SearchResult,
    };
    use memfuse_types::ConfigFingerprint;

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
}
