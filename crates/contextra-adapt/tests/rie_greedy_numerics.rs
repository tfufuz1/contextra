//! Numerische und verhaltensbasierte Testfälle für den RIE-Greedy-Personalismus-Mechanismus (§10.4.1).

#![cfg(feature = "rie-greedy-personalization")]

use contextra_adapt::{RieGreedyError, RieGreedyProfile};

#[test]
fn test_uncertainty_growth_after_discounting_and_update() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 3;
    let lambda = 1.0;
    let gamma = 0.8; // Starkes Vergessen
    let mut profile = RieGreedyProfile::new(dim, lambda, gamma);

    let initial_trace = profile.trace_precision_inv(); // lambda^-1 * 3 = 3.0

    // Kontext mit kleiner Norm
    let context = [0.1f32, 0.0, 0.0];
    let reward = 1.0f32;

    profile.update(&context, reward)?;

    let trace_after_update = profile.trace_precision_inv();

    // Bei Diskontierung VOR Update wächst die Unsicherheit / Spur von Lambda^-1 wegen gamma^-1 > 1,
    // sofern der Beobachtungsgewinn (Rang-1 Reduktion) kleiner als der Diskontierungseffekt ist.
    assert!(
        trace_after_update > initial_trace,
        "Trace of Lambda^-1 must increase when temporal discounting (gamma={gamma}) precedes rank-1 update (before: {initial_trace}, after: {trace_after_update})"
    );

    Ok(())
}

#[test]
fn test_dimension_mismatch_returns_error() {
    let mut profile = RieGreedyProfile::new(3, 1.0, 0.9);
    let invalid_context = [1.0f32, 2.0f32]; // Expected 3, got 2

    let predict_err = profile.predict(&invalid_context).unwrap_err();
    assert_eq!(
        predict_err,
        RieGreedyError::DimensionMismatch {
            expected: 3,
            actual: 2,
        }
    );

    let update_err = profile.update(&invalid_context, 1.0).unwrap_err();
    assert_eq!(
        update_err,
        RieGreedyError::DimensionMismatch {
            expected: 3,
            actual: 2,
        }
    );
}

#[test]
fn test_deterministic_behavior_bit_identical_state() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 4;
    let mut profile1 = RieGreedyProfile::new(dim, 1.0, 0.95);
    let mut profile2 = RieGreedyProfile::new(dim, 1.0, 0.95);

    let interactions = [
        ([0.5f32, -0.2f32, 0.1f32, 0.8f32], 0.7f32),
        ([0.1f32, 0.9f32, -0.4f32, 0.2f32], 0.3f32),
        ([-0.3f32, 0.2f32, 0.6f32, -0.1f32], 1.0f32),
    ];

    for (ctx, r) in &interactions {
        profile1.update(ctx, *r)?;
        profile2.update(ctx, *r)?;
    }

    assert_eq!(
        profile1.precision_inv, profile2.precision_inv,
        "precision_inv states must be bit-identical across identical runs"
    );
    assert_eq!(
        profile1.info_vector, profile2.info_vector,
        "info_vector states must be bit-identical across identical runs"
    );

    let test_ctx = [0.2f32, 0.3f32, 0.4f32, 0.5f32];
    let pred1 = profile1.predict(&test_ctx)?;
    let pred2 = profile2.predict(&test_ctx)?;

    assert_eq!(
        pred1.to_bits(),
        pred2.to_bits(),
        "predict results must be bit-identical"
    );

    Ok(())
}

#[test]
fn test_convergence_on_constant_context_and_reward() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 1;
    let lambda = 1.0;
    let gamma = 1.0; // Kein Vergessen für exakte Konvergenz gegen Mittelwert
    let mut profile = RieGreedyProfile::new(dim, lambda, gamma);

    let context = [1.0f32];
    let target_reward = 2.5f32;

    for _ in 0..500 {
        profile.update(&context, target_reward)?;
    }

    let pred = profile.predict(&context)?;
    let diff = (pred - target_reward).abs();

    assert!(
        diff < 0.05,
        "Prediction {pred} should converge close to target reward {target_reward} (diff={diff})"
    );

    Ok(())
}

#[test]
fn test_non_finite_input_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut profile = RieGreedyProfile::new(2, 1.0, 0.9);

    let nan_context = [f32::NAN, 1.0f32];
    assert_eq!(profile.predict(&nan_context), Err(RieGreedyError::NonFinite));
    assert_eq!(
        profile.update(&nan_context, 1.0),
        Err(RieGreedyError::NonFinite)
    );

    let valid_context = [1.0f32, 0.5f32];
    assert_eq!(
        profile.update(&valid_context, f32::INFINITY),
        Err(RieGreedyError::NonFinite)
    );

    Ok(())
}
