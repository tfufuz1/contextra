//! Test suite verifying zero-panic and deterministic behavior for NaN and Infinity float inputs.

use contextra_rank::calibration::{
    AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError, IsotonicCalibrator,
    PlattScaler,
};
use contextra_rank::drift::{DriftDetector, DriftStatus};
use contextra_rank::fusion::{
    apply_resonance_bonus, g, modulate_and_renormalize_weights, reciprocal_rank_fusion,
    ProvenanceBuilder, ResonanceConfig, SearchResult,
};

// ANCHOR[contextra-rank:SAFE-TEST-001]
#[test]
fn test_g_function_nan_inf_safety() {
    assert_eq!(g(f32::NAN), 0.0);
    assert_eq!(g(f32::INFINITY), 0.0);
    assert_eq!(g(f32::NEG_INFINITY), 0.0);
    assert_eq!(g(0.0), 0.0);
}

    // Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_isotonic_calibrator_nan_inf_safety() {
    let mut cal = IsotonicCalibrator::new(2, 10);
    cal.record_outcome(f32::NAN, true);
    cal.record_outcome(f32::INFINITY, false);
    cal.record_outcome(f32::NEG_INFINITY, true);

    assert_eq!(cal.observation_count(), 0);

    cal.record_outcome(0.1, false);
    cal.record_outcome(0.9, true);

    assert!(cal.calibrated_probability(f32::NAN).is_some());
    assert!(cal.calibrated_probability(f32::INFINITY).is_some());
    assert!(cal.calibrated_probability(f32::NEG_INFINITY).is_some());
    assert!(cal.calibrated_probability(0.5).is_some());
}

    // Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_platt_scaler_nan_inf_safety() {
    let bad_scaler = PlattScaler::new(f32::NAN, 1.0);
    assert!(bad_scaler.is_identity());

    let scaler = PlattScaler::new(2.0, -1.0);
    assert_eq!(scaler.predict(f32::NAN), 0.5);
    assert_eq!(scaler.predict(f32::INFINITY), 0.5);
    assert_eq!(scaler.predict(f32::NEG_INFINITY), 0.5);
    assert_eq!(scaler.transform(f32::NAN), 0.5);
}

    // Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_conformal_calibrator_nan_inf_safety() {
    assert!(matches!(
        AdaptiveConformalCalibrator::new(f32::NAN),
        Err(ConformalError::Alpha(_))
    ));
    assert!(matches!(
        AdaptiveConformalCalibrator::new(f32::INFINITY),
        Err(ConformalError::Alpha(_))
    ));

    if let Ok(mut cal) = AdaptiveConformalCalibrator::new(0.1) {
        assert!(matches!(
            cal.update(f32::NAN, 1.0),
            Err(ConformalError::NonConformityScore(_))
        ));
        assert!(matches!(
            cal.update(0.5, f32::NAN),
            Err(ConformalError::PropensityWeight(_))
        ));
        assert!(matches!(
            cal.update(0.5, -1.0),
            Err(ConformalError::PropensityWeight(_))
        ));
    }
}

    // Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_drift_detector_nan_inf_safety() {
    let mut detector = DriftDetector::new(10, f32::NAN);
    detector.set_baseline(f32::NAN);
    detector.set_baseline(f32::INFINITY);

    let status = detector.observe(f32::NAN);
    assert_eq!(status, DriftStatus::InsufficientData);

    detector.set_baseline(0.5);
    for _ in 0..10 {
        detector.observe(0.5);
    }

    assert_eq!(detector.overall_drift_status(), "stabil");
}

    // Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_provenance_builder_nan_inf_safety() {
    let builder = ProvenanceBuilder::new(f32::NAN)
        .vector(f32::NAN, 1, None)
        .bm25(f32::INFINITY, 1, None)
        .graph(f32::NEG_INFINITY, 1, None)
        .rerank_score(f32::NAN);

    let record = builder.build();
    assert_eq!(record.vector_distance, None);
    assert_eq!(record.bm25_score, None);
    assert_eq!(record.graph_score, None);
    assert_eq!(record.rerank_score, None);
}

    // Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_resonance_config_nan_inf_safety() {
    let bad_cfg = ResonanceConfig {
        beta: f32::NAN,
        gamma: 0.3,
    };
    let docs = vec![SearchResult {
        id: "doc1".to_string(),
        score: 0.8,
        metadata: None,
        matched_signals: vec!["vector".to_string()],
        provenance: None,
    }];

    let res = apply_resonance_bonus(docs.clone(), 1, &bad_cfg);
    assert_eq!(res.first().map(|d| d.score), Some(0.8));
}

    // Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_modulate_weights_nan_beta() {
    let sets = vec![("vec".to_string(), Vec::new(), 1.0)];
    let weights = modulate_and_renormalize_weights(&sets, None, f32::NAN);
    assert_eq!(weights, vec![1.0]);
}

    // Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_fusion_nan_doc_scores() {
    let docs = vec![
        SearchResult {
            id: "nan_doc".to_string(),
            score: f32::NAN,
            metadata: None,
            matched_signals: vec!["vec".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "valid_doc".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vec".to_string()],
            provenance: None,
        },
    ];

    let fused = reciprocal_rank_fusion(vec![docs], 10);
    assert_eq!(fused.len(), 2);
    assert!(fused.iter().all(|r| r.score.is_finite()));
}
