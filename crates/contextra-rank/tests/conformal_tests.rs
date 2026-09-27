// FILE-CONTEXT
// STAND: 2026-09-27T19:30:00Z
// ZWECK: Integration and invariant verification test suite for Adaptive Conformal Prediction (conformal.rs).
// INVARIANTEN: INV-CALIBRATION-CONFORMAL-1 (marginal coverage guarantee, strict determinism, zero-panic input validation).

use contextra_rank::{AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError};
use contextra_types::ConfigFingerprint;

struct SimpleLcg {
    state: u64,
}

impl SimpleLcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }
}

// ============================================================================
// 1. MARGINAL COVERAGE PROPERTY TEST
// ============================================================================

#[test]
fn test_conformal_marginal_coverage_guarantee() {
    let alpha = 0.10f32; // Target error rate 10% -> 90% coverage target
    let mut calibrator = AdaptiveConformalCalibrator::with_params(alpha, 0.5, 0.01)
        .expect("Valid calibrator creation");

    let mut prng = SimpleLcg::new(42);
    let total_steps = 10_000;
    let warmup_steps = 3_000;

    let mut post_warmup_covered = 0usize;
    let mut post_warmup_total = 0usize;

    for i in 0..total_steps {
        // Generate synthetic non-conformity score in [0, 1]
        let score = prng.next_f32();
        // Propensity weight simulating mild covariate shift adjustment
        let propensity_weight = 0.8 + 0.4 * prng.next_f32();

        let threshold_before = calibrator.threshold();

        let _changed = calibrator
            .update(score, propensity_weight)
            .expect("Valid update");

        if i >= warmup_steps {
            // Check if score was covered by the valid threshold at time t
            if score <= threshold_before {
                post_warmup_covered += 1;
            }
            post_warmup_total += 1;
        }
    }

    let empirical_coverage = post_warmup_covered as f32 / post_warmup_total as f32;
    let target_coverage = 1.0 - alpha;
    let tolerance = 0.04f32; // Documented tolerance +/- 4%

    assert!(
        (empirical_coverage - target_coverage).abs() <= tolerance,
        "Empirical coverage rate {empirical_coverage:.4} deviates from target {target_coverage:.4} beyond tolerance {tolerance}"
    );
}

// ============================================================================
// 2. DETERMINISM TEST
// ============================================================================

#[test]
fn test_conformal_strict_determinism() {
    let alpha = 0.15f32;

    let mut cal1 = AdaptiveConformalCalibrator::new(alpha).unwrap();
    let mut cal2 = AdaptiveConformalCalibrator::new(alpha).unwrap();

    let mut prng1 = SimpleLcg::new(1337);
    let mut prng2 = SimpleLcg::new(1337);

    let steps = 500;
    let mut trajectory1 = Vec::with_capacity(steps);
    let mut trajectory2 = Vec::with_capacity(steps);

    for _ in 0..steps {
        let score1 = prng1.next_f32();
        let weight1 = 0.5 + prng1.next_f32();
        let res1 = cal1.update(score1, weight1).unwrap();
        trajectory1.push((cal1.threshold().to_bits(), res1));

        let score2 = prng2.next_f32();
        let weight2 = 0.5 + prng2.next_f32();
        let res2 = cal2.update(score2, weight2).unwrap();
        trajectory2.push((cal2.threshold().to_bits(), res2));
    }

    assert_eq!(
        trajectory1, trajectory2,
        "Threshold trajectories diverged between two identical seeded runs"
    );
}

// ============================================================================
// 3. ZERO-PANIC & EDGE-CASE HARDENING TEST
// ============================================================================

#[test]
fn test_conformal_zero_panic_invalid_inputs() {
    let mut cal = AdaptiveConformalCalibrator::new(0.1).unwrap();
    let initial_threshold = cal.threshold();

    // 1. Non-finite or non-positive propensity weights MUST return Err
    let invalid_weights = vec![
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        0.0,
        -0.0001,
        -1.0,
        -100.0,
    ];

    for bad_w in invalid_weights {
        let res = cal.update(0.5, bad_w);
        assert!(
            matches!(res, Err(ConformalError::InvalidPropensityWeight(w)) if w.to_bits() == bad_w.to_bits() || bad_w <= 0.0),
            "Expected InvalidPropensityWeight for weight {bad_w}, got {res:?}"
        );
        // Ensure threshold state was NOT modified
        assert_eq!(cal.threshold(), initial_threshold);
    }

    // 2. Non-finite non-conformity scores MUST return Err
    let invalid_scores = vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY];

    for bad_score in invalid_scores {
        let res = cal.update(bad_score, 1.0);
        assert!(
            matches!(res, Err(ConformalError::InvalidNonConformityScore(_))),
            "Expected InvalidNonConformityScore for score {bad_score}, got {res:?}"
        );
        assert_eq!(cal.threshold(), initial_threshold);
    }

    // 3. Invalid alpha in constructor MUST return Err
    let invalid_alphas = vec![0.0, 1.0, -0.1, 1.5, f32::NAN, f32::INFINITY];
    for bad_alpha in invalid_alphas {
        let res = AdaptiveConformalCalibrator::new(bad_alpha);
        assert!(
            matches!(res, Err(ConformalError::InvalidAlpha(_))),
            "Expected InvalidAlpha for alpha {bad_alpha}, got {res:?}"
        );
    }
}

// ============================================================================
// 4. BOUNDS & CONFIG INVALIDATION TESTS
// ============================================================================

#[test]
fn test_conformal_threshold_clamping_and_invalidation() {
    let mut cal = AdaptiveConformalCalibrator::with_params(0.1, 0.5, 0.2)
        .unwrap()
        .with_bounds(0.2, 0.8)
        .unwrap();

    // Drive threshold up with scores above threshold and high propensity weight
    for _ in 0..50 {
        let _ = cal.update(1.0, 5.0);
    }
    assert_eq!(cal.threshold(), 0.8, "Threshold should be clamped to max_threshold 0.8");

    // Drive threshold down
    for _ in 0..50 {
        let _ = cal.update(0.0, 5.0);
    }
    assert_eq!(cal.threshold(), 0.2, "Threshold should be clamped to min_threshold 0.2");

    // Test ConfigFingerprint invalidation (P8 compliance)
    let fp1 = ConfigFingerprint::new("model-a", "Q4_0", "tmpl-1", 0.7);
    cal.invalidate_on_config_change(fp1.clone());

    // Should reset threshold to default initial threshold clamped to [0.2, 0.8] -> 0.5
    assert_eq!(cal.threshold(), 0.5);
    assert_eq!(cal.observation_count(), 0);

    // Identical fingerprint should NOT reset
    let _ = cal.update(1.0, 1.0);
    assert_eq!(cal.observation_count(), 1);
    cal.invalidate_on_config_change(fp1);
    assert_eq!(cal.observation_count(), 1);
}
