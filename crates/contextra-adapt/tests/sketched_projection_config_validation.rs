// FILE-CONTEXT
// ZWECK: Konfigurations- und Validierungstests für SketchedProjection (§13.2).
// STAND: TS:2026-09-26T00:00:00Z

use contextra_adapt::bandit::{
    BanditError, BanditImplementation, BanditProfileState, SketchMatrix,
};

#[test]
fn test_sketch_matrix_from_seed_bounds_validation() {
    let original_dim = 128;

    // projected_dim == 0 -> Err
    let err_zero = SketchMatrix::from_seed(42, original_dim, 0).unwrap_err();
    assert_eq!(
        err_zero,
        BanditError::InvalidConfig("projected_dim must be in (0, original_dim]".into())
    );

    // projected_dim > original_dim -> Err
    let err_exceed = SketchMatrix::from_seed(42, original_dim, original_dim + 1).unwrap_err();
    assert_eq!(
        err_exceed,
        BanditError::InvalidConfig("projected_dim must be in (0, original_dim]".into())
    );

    // Valid boundaries (1 and original_dim) -> Ok
    assert!(SketchMatrix::from_seed(42, original_dim, 1).is_ok());
    assert!(SketchMatrix::from_seed(42, original_dim, original_dim).is_ok());
}

#[test]
fn test_bandit_profile_state_sketched_config_validation() {
    let original_dim = 64;
    let x = vec![0.5f32; original_dim];

    // Invalid config: projected_dim == 0
    let mut state_zero = BanditProfileState::cold_start(original_dim, 0.5);
    state_zero.implementation = BanditImplementation::SketchedProjection { projected_dim: 0 };

    let score_err = state_zero.score(&x, 0.1, false).unwrap_err();
    assert_eq!(
        score_err,
        BanditError::InvalidConfig("projected_dim must be in (0, original_dim]".into())
    );

    let update_err = state_zero.update(&x, 1.0, 0.1, false).unwrap_err();
    assert_eq!(
        update_err,
        BanditError::InvalidConfig("projected_dim must be in (0, original_dim]".into())
    );

    // Invalid config: projected_dim > original_dim
    let mut state_exceed = BanditProfileState::cold_start(original_dim, 0.5);
    state_exceed.implementation = BanditImplementation::SketchedProjection {
        projected_dim: original_dim + 10,
    };

    let score_err2 = state_exceed.score(&x, 0.1, false).unwrap_err();
    assert_eq!(
        score_err2,
        BanditError::InvalidConfig("projected_dim must be in (0, original_dim]".into())
    );

    let update_err2 = state_exceed.update(&x, 1.0, 0.1, false).unwrap_err();
    assert_eq!(
        update_err2,
        BanditError::InvalidConfig("projected_dim must be in (0, original_dim]".into())
    );
}

#[test]
fn test_non_finite_variance_returns_error() {
    let original_dim = 16;
    let projected_dim = 8;
    let x = vec![1.0f32; original_dim];

    let mut state = BanditProfileState::cold_start(original_dim, 0.5);
    state.implementation = BanditImplementation::SketchedProjection { projected_dim };
    state.ensure_sketched_state(projected_dim).expect("valid state setup");

    // Force inv_a matrix entry to NaN or extreme negative
    state.inv_a[0] = f32::NAN;

    let err = state.score(&x, 0.0, false).unwrap_err();
    match err {
        BanditError::NonFiniteVariance(v) => {
            assert!(v.is_nan());
        }
        other => panic!("Expected NonFiniteVariance error, got {:?}", other),
    }

    // Test negative variance
    state.inv_a[0] = -100.0;
    let err_neg = state.score(&x, 0.0, false).unwrap_err();
    match err_neg {
        BanditError::NonFiniteVariance(v) => {
            assert!(v < 0.0);
        }
        other => panic!("Expected NonFiniteVariance error, got {:?}", other),
    }
}
