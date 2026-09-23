// Testpflicht AK-15 (docs/specs/CONTEXTRA_SPEC_v2.md)

use contextra_router::OffPolicyEvaluator;

#[test]
fn test_propensity_lower_bound_clamping_and_discard() {
    let mut evaluator = OffPolicyEvaluator::new();

    // Propensity below 0.01 (e.g. 0.001, 0.0) is clamped to lower bound 0.01 for IPS weight calculation
    evaluator.observe(1, 1, 0.001, 1.0);
    assert!((evaluator.cumulative_ips() - 100.0).abs() < 1e-9);
    assert_eq!(evaluator.samples(), 1);
    assert_eq!(evaluator.discarded(), 0);

    // 0.0 is non-negative, so it is also clamped to lower bound 0.01
    evaluator.observe(1, 1, 0.0, 1.0);
    assert!((evaluator.cumulative_ips() - 200.0).abs() < 1e-9);
    assert_eq!(evaluator.samples(), 2);
    assert_eq!(evaluator.discarded(), 0);

    // Invalid propensity values (negative, NaN, or > 1.0) are rejected / discarded
    evaluator.observe(1, 1, -0.05, 1.0);
    evaluator.observe(1, 1, f32::NAN, 1.0);
    evaluator.observe(1, 1, 1.2, 1.0);

    // Cumulative IPS should not increase for discarded samples
    assert!((evaluator.cumulative_ips() - 200.0).abs() < 1e-9);
    assert_eq!(evaluator.samples(), 5);
    assert_eq!(evaluator.discarded(), 3);
    assert_eq!(evaluator.stats().discarded, 3);
}

#[test]
fn test_valid_propensity_accepted() {
    let mut evaluator = OffPolicyEvaluator::new();

    // Boundary case: propensity == 0.01
    evaluator.observe(1, 1, 0.01, 1.0);
    assert!((evaluator.cumulative_ips() - 100.0).abs() < 1e-9);

    // Higher valid case: propensity == 0.5
    evaluator.observe(1, 1, 0.5, 1.0);
    assert!((evaluator.cumulative_ips() - 102.0).abs() < 1e-9);

    // Upper boundary case: propensity == 1.0
    evaluator.observe(1, 1, 1.0, 1.0);
    assert!((evaluator.cumulative_ips() - 103.0).abs() < 1e-9);

    assert_eq!(evaluator.samples(), 3);
    assert_eq!(evaluator.discarded(), 0);
    assert!((evaluator.estimate() - (103.0 / 3.0)).abs() < 1e-9);
}
