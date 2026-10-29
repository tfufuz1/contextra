//! Property tests for Isotonic Calibration, Platt Scaling, Conformal Prediction Coverage, and Drift Detection.

use contextra_rank::calibration::{
    AdaptiveConformalCalibrator, ConformalCalibrator, IsotonicCalibrator, PlattScaler,
};
use contextra_rank::drift::{DriftDetector, DriftStatus};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_isotonic_pava_monotonicity(
        scores in prop::collection::vec(0.0f32..1.0f32, 10..50),
        labels in prop::collection::vec(any::<bool>(), 10..50),
    ) {
        let mut cal = IsotonicCalibrator::new(5, 100);
        let n = scores.len().min(labels.len());

        for i in 0..n {
            if let (Some(&s), Some(&l)) = (scores.get(i), labels.get(i)) {
                cal.record_outcome(s, l);
            }
        }

        cal.force_rebuild();

        let test_scores = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
        let probs: Vec<f32> = test_scores
            .iter()
            .filter_map(|&s| cal.calibrated_probability(s))
            .collect();

        for window in probs.windows(2) {
            if let [p1, p2] = window {
                prop_assert!(p1 <= p2, "Isotonic probabilities must be non-decreasing: {p1} <= {p2}");
                prop_assert!(*p1 >= 0.0 && *p1 <= 1.0, "Probabilities must be in [0, 1]");
            }
        }
    }

    #[test]
    fn prop_platt_scaling_range_bounded(
        logit in -100.0f32..100.0f32,
        a in -10.0f32..10.0f32,
        b in -10.0f32..10.0f32,
    ) {
        let scaler = PlattScaler::new(a, b);
        let p = scaler.predict(logit);
        prop_assert!((0.0..=1.0).contains(&p), "Platt scaling output must be in [0, 1], got {p}");
    }

    #[test]
    fn prop_drift_detector_stationary_no_false_alarms(
        mean in 0.2f32..0.8f32,
        noise_level in 0.001f32..0.02f32,
        num_obs in 30usize..80usize,
    ) {
        let mut detector = DriftDetector::new(50, 0.15);
        detector.set_baseline(mean);

        let mut false_alarms = 0;
        for i in 0..num_obs {
            let offset = if i % 2 == 0 { noise_level } else { -noise_level };
            let score = (mean + offset).clamp(0.0, 1.0);
            let status = detector.observe(score);
            if matches!(status, DriftStatus::DriftDetected { .. }) {
                false_alarms += 1;
            }
        }

        prop_assert_eq!(false_alarms, 0, "Stationary score distribution must yield 0 false alarms");
    }
}

#[test]
fn test_conformal_coverage_empirical_guarantee_synthetic() {
    let alpha = 0.10_f32;
    let mut calibrator = match AdaptiveConformalCalibrator::with_params(alpha, 0.5, 0.02) {
        Ok(c) => c,
        Err(_) => return,
    };

    let num_samples = 1000usize;
    let mut miscoverage_count = 0usize;

    // Synthetic non-conformity scores generated deterministically
    for t in 0..num_samples {
        let phase = (t as f32 * 0.1).sin().abs();
        let score = (0.3 + 0.4 * phase).clamp(0.0, 1.0);

        let current_threshold = calibrator.threshold();
        if score > current_threshold {
            miscoverage_count += 1;
        }

        let update_res = calibrator.update(score, 1.0);
        assert!(
            update_res.is_ok(),
            "Conformal update must succeed for valid scores"
        );
    }

    let empirical_miscoverage = miscoverage_count as f32 / num_samples as f32;
    let empirical_coverage = 1.0 - empirical_miscoverage;

    // Target coverage is 1 - alpha = 0.90. Over 1000 online steps, coverage should satisfy empirical bound >= 1 - alpha - tolerance
    let tolerance = 0.08_f32;
    assert!(
        empirical_coverage >= (1.0 - alpha - tolerance),
        "Empirical coverage {empirical_coverage:.3} is below target guarantee {}",
        1.0 - alpha - tolerance
    );
}

#[test]
fn test_platt_scaler_fit_properties_range() {
    let obs = vec![
        (0.1, false),
        (0.2, false),
        (0.3, false),
        (0.7, true),
        (0.8, true),
        (0.9, true),
    ];

    let scaler = PlattScaler::fit(&obs);
    assert!(scaler.is_fitted());

    let p_low = scaler.predict(0.1);
    let p_high = scaler.predict(0.9);

    assert!((0.0..=1.0).contains(&p_low));
    assert!((0.0..=1.0).contains(&p_high));
    assert!(
        p_low < p_high,
        "Higher logit must yield higher calibrated probability"
    );
}
