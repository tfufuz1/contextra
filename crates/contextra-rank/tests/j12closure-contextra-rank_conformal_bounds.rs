use contextra_rank::{AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError};

#[test]
fn test_j12closure_adaptive_conformal_calibrator_with_bounds() -> Result<(), ConformalError> {
    let calibrator_default = AdaptiveConformalCalibrator::new(0.1)?;
    assert_eq!(calibrator_default.threshold(), 0.5);

    let bounded = AdaptiveConformalCalibrator::bounded(0.1, 0.2, 0.8)?;
    assert_eq!(bounded.threshold(), 0.5);

    let mut calibrator = AdaptiveConformalCalibrator::with_params(0.1, 0.5, 0.2)?
        .with_bounds(0.2, 0.8)?;

    assert_eq!(calibrator.threshold(), 0.5);

    for _ in 0..50 {
        calibrator.update(1.0, 1.0)?;
    }
    assert!(calibrator.threshold() <= 0.8);

    for _ in 0..100 {
        calibrator.update(0.0, 1.0)?;
    }
    assert!(calibrator.threshold() >= 0.2);

    let invalid_res = AdaptiveConformalCalibrator::new(0.1)
        .and_then(|c| c.with_bounds(0.8, 0.2));
    assert!(matches!(invalid_res, Err(ConformalError::InvalidThreshold(_))));

    Ok(())
}
