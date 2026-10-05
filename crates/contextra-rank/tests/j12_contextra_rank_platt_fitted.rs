use contextra_rank::calibration::PlattScaler;

#[test]
fn test_platt_scaler_is_fitted_lifecycle() {
    // Default / Identity scaler transform runs is_fitted() check
    let default_scaler = PlattScaler::default();
    assert!(!default_scaler.is_fitted());
    assert!(default_scaler.is_identity());

    // Identity transformation
    let p_uncalibrated = default_scaler.transform(1.5);
    assert!((p_uncalibrated - (1.0 / (1.0 + (-1.5_f32).exp()))).abs() < f32::EPSILON);

    // Fit on positive and negative observations
    let observations = vec![
        (2.5, true),
        (1.8, true),
        (1.2, true),
        (-0.5, false),
        (-1.8, false),
        (-2.2, false),
    ];

    let fitted_scaler = PlattScaler::fit(&observations);

    // Fitted scaler must report is_fitted() == true
    assert!(fitted_scaler.is_fitted());
    assert!(!fitted_scaler.is_identity());

    // Fitted transformation through public transform API
    let p_pos = fitted_scaler.transform(2.0);
    let p_neg = fitted_scaler.transform(-2.0);
    assert!(p_pos > p_neg);
    assert!((0.0..=1.0).contains(&p_pos));
    assert!((0.0..=1.0).contains(&p_neg));
}
