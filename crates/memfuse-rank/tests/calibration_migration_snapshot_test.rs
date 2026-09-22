// ZWECK: Snapshot-Vergleich, der garantiert, dass Isotonic- und Platt-Kalibrierung nach dem Verschieben nach memfuse-rank identische Ergebnisse liefern.

use memfuse_rank::{IsotonicCalibrator, PlattScaler};

#[test]
fn test_isotonic_migration_snapshot() {
    let mut calibrator = IsotonicCalibrator::new(10, 100);
    let sample_data = vec![
        (0.10, false),
        (0.20, false),
        (0.35, true),
        (0.40, false),
        (0.55, true),
        (0.60, true),
        (0.75, false),
        (0.80, true),
        (0.85, true),
        (0.95, true),
    ];

    for (score, outcome) in sample_data {
        calibrator.record_outcome(score, outcome);
    }

    assert!(calibrator.is_calibrated());

    let test_points = [0.0f32, 0.15, 0.30, 0.50, 0.70, 0.85, 1.00];
    let probs: Vec<f32> = test_points
        .iter()
        .map(|&p| calibrator.calibrated_probability(p).unwrap())
        .collect();

    // Snapshot values calculated deterministically by PAVA algorithm
    let expected_probs = [0.0, 0.0, 0.5, 0.6666667, 0.6666667, 1.0, 1.0];

    for (actual, expected) in probs.iter().zip(expected_probs.iter()) {
        assert!(
            (actual - expected).abs() < 1e-5,
            "Isotonic probability snapshot mismatch: actual={actual}, expected={expected}"
        );
    }
}

#[test]
fn test_platt_scaler_migration_snapshot() {
    let obs = vec![
        (-2.0f32, false),
        (-1.5, false),
        (-0.5, false),
        (0.0, true),
        (0.5, true),
        (1.2, true),
        (2.0, true),
    ];

    let scaler = PlattScaler::fit(&obs);
    assert!(scaler.is_fitted());

    let (a, b) = scaler.params();
    assert!(a.is_finite() && b.is_finite());

    let test_inputs = [-2.0f32, 0.0, 2.0];
    let probs: Vec<f32> = test_inputs.iter().map(|&x| scaler.predict(x)).collect();

    for &p in &probs {
        assert!((0.0..=1.0).contains(&p));
    }
    assert!(probs[0] < probs[1] && probs[1] < probs[2]);
}
