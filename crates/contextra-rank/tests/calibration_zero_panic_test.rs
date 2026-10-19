use contextra_rank::calibration::{IsotonicCalibrator, PlattScaler};
use contextra_rank::drift::{DriftDetector, DriftStatus};
use contextra_types::ConfigFingerprint;

#[test]
fn test_isotonic_calibrator_zero_panic_edge_cases() {
    let mut calib = IsotonicCalibrator::new(5, 100);
    assert!(!calib.is_calibrated());

    // Record non-finite inputs
    calib.record_outcome(f32::NAN, true);
    calib.record_outcome(f32::INFINITY, false);
    calib.record_outcome(f32::NEG_INFINITY, true);
    calib.record_outcome(0.5, true);
    calib.record_outcome(0.8, false);

    assert!(calib.is_calibrated());

    // Calibrated probability with NaN or Inf should not panic
    let p1 = calib.calibrated_probability(f32::NAN);
    assert!(p1.is_none() || p1.unwrap().is_finite());

    let p2 = calib.calibrated_probability(f32::INFINITY);
    assert!(p2.is_none() || p2.unwrap().is_finite());

    let p3 = calib.calibrated_probability(f32::NEG_INFINITY);
    assert!(p3.is_none() || p3.unwrap().is_finite());

    // Invalidation
    let fp = ConfigFingerprint::new("model_v1", "none", "template", 0.0);
    calib.invalidate_on_config_change(fp);
    assert!(!calib.is_calibrated());
}

#[test]
fn test_platt_scaler_zero_panic_edge_cases() {
    // Identity scaler
    let scaler = PlattScaler::identity();
    assert_eq!(scaler.apply(0.0), 0.5);
    assert!(!scaler.apply(f32::NAN).is_nan());

    // Single label dataset (all true)
    let single_label_data = vec![(1.0, true), (2.0, true), (3.0, true)];
    let fitted_true = PlattScaler::fit(&single_label_data);
    assert!(fitted_true.apply(2.0) >= 0.0 && fitted_true.apply(2.0) <= 1.0);

    // Single label dataset (all false)
    let single_label_false = vec![(1.0, false), (2.0, false), (3.0, false)];
    let fitted_false = PlattScaler::fit(&single_label_false);
    assert!(fitted_false.apply(2.0) >= 0.0 && fitted_false.apply(2.0) <= 1.0);

    // Non-finite logits in dataset
    let nan_data = vec![(f32::NAN, true), (f32::INFINITY, false), (0.5, true)];
    let fitted_nan = PlattScaler::fit(&nan_data);
    assert!(fitted_nan.apply(0.5) >= 0.0 && fitted_nan.apply(0.5) <= 1.0);
}

#[test]
fn test_drift_detector_zero_panic_edge_cases() {
    let mut detector = DriftDetector::new(5, 0.1);
    detector.set_baseline(0.5);

    // Record NaN and Inf scores
    let st1 = detector.observe(f32::NAN);
    assert_eq!(st1, DriftStatus::InsufficientData);

    let _ = detector.observe(f32::INFINITY);
    let _ = detector.observe(f32::NEG_INFINITY);

    for _ in 0..5 {
        let _ = detector.observe(0.5);
    }

    // Status check should be panic-free
    let status = detector.analyze();
    assert!(matches!(status, DriftStatus::Stable { .. } | DriftStatus::DriftDetected { .. }));
    assert_ne!(detector.overall_drift_status(), "");

    detector.clear_observations();
    assert_eq!(detector.analyze(), DriftStatus::InsufficientData);
}
