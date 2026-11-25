use contextra_rank::PlattScaler;

#[test]
fn test_j12closure_platt_scaler_is_fitted() {
    let uncalibrated = PlattScaler::default();
    assert!(!uncalibrated.is_fitted());
    assert!(uncalibrated.is_identity());

    let identity_scaler = PlattScaler::identity();
    assert!(!identity_scaler.is_fitted());

    let fitted_scaler = PlattScaler::new(2.5, -0.5);
    assert!(fitted_scaler.is_fitted());
    assert!(!fitted_scaler.is_identity());

    let ctx = fitted_scaler.calibration_context(
        "vector",
        contextra_rank::drift::DriftStatus::Stable { mean_shift: 0.01 },
    );
    assert!(ctx.is_calibrated);

    let observations = vec![(0.1, false), (0.2, false), (0.8, true), (0.9, true)];
    let trained_scaler = PlattScaler::fit(&observations);
    assert!(trained_scaler.is_fitted());
}
