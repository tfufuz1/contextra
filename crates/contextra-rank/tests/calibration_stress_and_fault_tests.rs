#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// FILE-CONTEXT
// STAND: 2026-09-15T14:45:00Z (SESSION: 527bbb50)
// ZWECK: Stress, fault-injection, and multi-threading concurrency tests for contextra-rank calibration.
// INVARIANTEN: INV-CAL-1 (no silent fallback before warmup), INV-CAL-2 (fingerprint reset), PID anti-windup & k_min floor >= 50.

use contextra_rank::{IsotonicCalibrator, PlattScaler};
use proptest::prelude::*;

#[test]
fn test_isotonic_fifo_eviction_and_force_rebuild() {
    let max_obs = 20;
    let mut cal = IsotonicCalibrator::new(10, max_obs);

    // Record initial observations up to max_obs
    for i in 0..max_obs {
        cal.record_outcome((i as f32) / (max_obs as f32), true);
    }
    assert_eq!(cal.observation_count(), max_obs);
    let prob_before = cal.calibrated_probability(0.5).unwrap();

    // Push 10 new false outcomes, forcing FIFO eviction of oldest true outcomes
    for _ in 0..10 {
        cal.record_outcome(0.5, false);
    }
    assert_eq!(cal.observation_count(), max_obs);

    cal.force_rebuild();
    let prob_after = cal.calibrated_probability(0.5).unwrap();
    assert!(
        prob_after < prob_before,
        "Evicting true observations and adding false observations must lower probability"
    );
}

#[test]
fn test_platt_scaler_extreme_finite_logits_monotonicity() {
    let mut obs = Vec::new();
    for i in -50..=50 {
        let logit = i as f32 * 0.1;
        obs.push((logit, logit > 0.0));
    }

    let scaler = PlattScaler::fit(&obs);
    assert!(scaler.is_fitted());

    let logits: Vec<f32> = vec![-100.0, -10.0, -1.0, 0.0, 1.0, 10.0, 100.0];
    let probs: Vec<f32> = logits.iter().map(|&l| scaler.predict(l)).collect();

    for w in probs.windows(2) {
        assert!(
            w[0] <= w[1] + 1e-6,
            "PlattScaler output must be monotonically non-decreasing"
        );
        assert!(
            (0.0..=1.0).contains(&w[0]),
            "PlattScaler output must be within [0, 1]"
        );
    }
}

proptest! {
    #[test]
    fn prop_isotonic_ece_non_negative_and_bounded(
        outcomes in prop::collection::vec(prop::bool::ANY, 50..150)
    ) {
        let mut cal = IsotonicCalibrator::new(20, 200);
        for (i, &outcome) in outcomes.iter().enumerate() {
            let score = (i as f32) / (outcomes.len() as f32);
            cal.record_outcome(score, outcome);
        }

        let ece = cal.expected_calibration_error();
        prop_assert!(ece.is_some());
        let val = ece.unwrap();
        prop_assert!((0.0..=1.0).contains(&val), "ECE must be bounded in [0, 1], got {}", val);
    }
}
