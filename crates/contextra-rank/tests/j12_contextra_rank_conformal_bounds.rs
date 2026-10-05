use contextra_rank::{AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError};

#[test]
fn test_adaptive_conformal_calibrator_with_bounds_clamping() -> Result<(), ConformalError> {
    // Standard constructor AdaptiveConformalCalibrator::new internally executes with_bounds
    let calibrator_default = AdaptiveConformalCalibrator::new(0.1)?;
    assert_eq!(calibrator_default.threshold(), 0.5);

    // Custom configuration using with_bounds
    let mut calibrator = AdaptiveConformalCalibrator::with_params(0.1, 0.5, 0.2)?
        .with_bounds(0.3, 0.7)?;

    assert_eq!(calibrator.threshold(), 0.5);

    // Drive updates with high non-conformity scores through ConformalCalibrator::update
    for _ in 0..50 {
        calibrator.update(1.0, 1.0)?;
    }

    // Assert that threshold is clamped at upper bound 0.7
    assert_eq!(calibrator.threshold(), 0.7);

    // Drive updates with low non-conformity scores through ConformalCalibrator::update
    for _ in 0..50 {
        calibrator.update(0.0, 1.0)?;
    }

    // Assert that threshold is clamped at lower bound 0.3
    assert_eq!(calibrator.threshold(), 0.3);

    Ok(())
}

#[test]
fn test_adaptive_conformal_calibrator_with_invalid_bounds() {
    let calibrator_res = AdaptiveConformalCalibrator::new(0.1)
        .unwrap()
        .with_bounds(0.8, 0.2); // Min > Max

    assert!(matches!(calibrator_res, Err(ConformalError::InvalidThreshold(_))));
}
