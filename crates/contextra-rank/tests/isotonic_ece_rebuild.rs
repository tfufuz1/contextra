use contextra_rank::calibration::isotonic::{
    IsotonicCalibrator, DEFAULT_ECE_REBUILD_THRESHOLD,
};

#[test]
fn test_well_calibrated_data_returns_false_and_no_rebuild() {
    let mut calibrator = IsotonicCalibrator::new(50, 1000);

    // Well calibrated data: low raw scores get false outcomes, high raw scores get true outcomes
    for i in 0..50 {
        let score = (i as f32) / 50.0;
        let outcome = score >= 0.5;
        calibrator.record_outcome(score, outcome);
    }

    // Force an initial model rebuild so that model is clean
    calibrator.force_rebuild();

    let initial_last_cal = calibrator.last_calibration_at();
    let initial_ece = calibrator
        .expected_calibration_error()
        .expect("should have ECE");

    // ECE should be very low (close to 0.0) for perfectly monotonic well-calibrated data
    assert!(
        initial_ece <= DEFAULT_ECE_REBUILD_THRESHOLD,
        "Expected ECE <= threshold, got {}",
        initial_ece
    );

    // Call maybe_rebuild_on_ece
    let rebuilt = calibrator.maybe_rebuild_on_ece(DEFAULT_ECE_REBUILD_THRESHOLD);

    assert!(!rebuilt, "maybe_rebuild_on_ece should return false for well-calibrated data");
    assert_eq!(
        calibrator.last_calibration_at(),
        initial_last_cal,
        "Calibration timestamp should not change when rebuild is not triggered"
    );
}

#[test]
fn test_biased_observations_trigger_rebuild_and_ece_decreases_or_remains() {
    let mut calibrator = IsotonicCalibrator::new(50, 1000);

    // Record initial observations where all scores predict outcome = false
    for i in 0..50 {
        let score = (i as f32) / 50.0;
        calibrator.record_outcome(score, false);
    }

    // Force initial build
    calibrator.force_rebuild();

    // Now record biased new observations: high scores (e.g. 0.9) suddenly get true outcomes
    // This introduces a model_dirty state and significant miscalibration relative to cached predictions
    for _ in 0..20 {
        calibrator.record_outcome(0.9, true);
    }

    // Before rebuild, calculate ECE on dirty model (which rebuilds model for calculation)
    let ece_before = calibrator
        .expected_calibration_error()
        .expect("ECE should be present");

    // Use a small threshold below ece_before to guarantee rebuild trigger
    let custom_threshold = (ece_before / 2.0).min(DEFAULT_ECE_REBUILD_THRESHOLD);

    let rebuilt = calibrator.maybe_rebuild_on_ece(custom_threshold);

    assert!(rebuilt, "maybe_rebuild_on_ece should return true when ECE exceeds threshold");

    let ece_after = calibrator
        .expected_calibration_error()
        .expect("ECE should be present after rebuild");

    assert!(
        ece_after <= ece_before,
        "ECE after rebuild ({}) should be <= ECE before rebuild ({})",
        ece_after,
        ece_before
    );
}

#[test]
fn test_edge_cases_insufficient_obs_nan_and_negative_thresholds() {
    // 1. Insufficient observations (below warmup requirement of 50)
    let mut calibrator = IsotonicCalibrator::new(50, 1000);
    for i in 0..10 {
        calibrator.record_outcome((i as f32) / 10.0, true);
    }

    assert_eq!(
        calibrator.expected_calibration_error(),
        None,
        "ECE should be None when below warmup"
    );

    // maybe_rebuild_on_ece should return false and not panic when ECE is None
    let rebuilt_none = calibrator.maybe_rebuild_on_ece(DEFAULT_ECE_REBUILD_THRESHOLD);
    assert!(!rebuilt_none, "Should return false when ECE is None");

    // 2. Warm up the calibrator
    for i in 10..50 {
        calibrator.record_outcome((i as f32) / 50.0, i % 2 == 0);
    }
    assert!(calibrator.is_calibrated());

    // 3. NaN threshold -> should return false and not panic
    let rebuilt_nan = calibrator.maybe_rebuild_on_ece(f32::NAN);
    assert!(!rebuilt_nan, "Should return false when threshold is NaN");

    // 4. Positive Infinity threshold -> should return false
    let rebuilt_inf = calibrator.maybe_rebuild_on_ece(f32::INFINITY);
    assert!(!rebuilt_inf, "Should return false when threshold is positive infinity");

    // 5. Negative threshold (e.g. -0.10) with valid positive ECE -> should trigger rebuild safely
    let rebuilt_neg = calibrator.maybe_rebuild_on_ece(-0.10);
    assert!(rebuilt_neg, "Negative finite threshold should trigger rebuild if ECE > threshold");
}
