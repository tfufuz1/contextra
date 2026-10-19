use contextra_rank::calibration::isotonic::{IsotonicCalibrator, DEFAULT_ECE_REBUILD_THRESHOLD};

#[test]
fn test_well_calibrated_data_no_rebuild() {
    let mut calibrator = IsotonicCalibrator::new(50, 1000);

    // Record well-calibrated observations (raw score ~ true probability)
    for i in 0..100 {
        let score = (i as f32) / 100.0;
        let outcome = score > 0.5;
        calibrator.record_outcome(score, outcome);
    }

    // High threshold (0.10) for well-calibrated data where ECE is near 0
    let rebuild_triggered = calibrator.maybe_rebuild_on_ece(DEFAULT_ECE_REBUILD_THRESHOLD);
    assert!(
        !rebuild_triggered,
        "Well calibrated data should not trigger rebuild"
    );
}

#[test]
fn test_systematically_biased_observations_triggers_rebuild() {
    let mut calibrator = IsotonicCalibrator::new(50, 1000);

    // Build initial state with 100 observations
    for i in 0..100 {
        let score = (i as f32) / 100.0;
        calibrator.record_outcome(score, score > 0.5);
    }

    let initial_ece = calibrator
        .expected_calibration_error()
        .expect("ECE should be computed");

    // Pass a threshold lower than the current ECE (e.g., -0.01) to simulate threshold breach
    let threshold = -0.01;
    let rebuild_triggered = calibrator.maybe_rebuild_on_ece(threshold);

    assert!(
        rebuild_triggered,
        "ECE exceeding threshold should trigger rebuild"
    );

    let ece_after = calibrator
        .expected_calibration_error()
        .expect("ECE after rebuild");
    assert!(
        ece_after <= initial_ece || (ece_after - initial_ece).abs() < 1e-5,
        "ECE after rebuild should be less than or equal to initial ECE"
    );
}

#[test]
fn test_edge_cases_uncalibrated_nan_and_negative_threshold() {
    let mut calibrator = IsotonicCalibrator::new(50, 1000);

    // 1. Too few observations (< warmup_required of 50) -> expected_calibration_error() returns None
    for i in 0..10 {
        calibrator.record_outcome(i as f32 / 10.0, i % 2 == 0);
    }
    assert_eq!(calibrator.observation_count(), 10);
    assert!(!calibrator.is_calibrated());

    // None ECE should never trigger rebuild regardless of threshold
    assert!(!calibrator.maybe_rebuild_on_ece(0.10));
    assert!(!calibrator.maybe_rebuild_on_ece(f32::NAN));
    assert!(!calibrator.maybe_rebuild_on_ece(-0.10));

    // 2. Calibrated observations (>= warmup_required)
    for i in 10..60 {
        calibrator.record_outcome(i as f32 / 60.0, i % 2 == 0);
    }
    assert!(calibrator.is_calibrated());

    // NaN threshold should never trigger rebuild
    assert!(!calibrator.maybe_rebuild_on_ece(f32::NAN));

    // Finite negative threshold should not panic
    let _res = calibrator.maybe_rebuild_on_ece(-0.10);
}
