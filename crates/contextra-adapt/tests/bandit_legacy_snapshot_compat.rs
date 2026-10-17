//! Test for legacy snapshot compatibility and automatic reconstruction of A from A⁻¹.

#![cfg(feature = "bandit-routing")]

use contextra_adapt::BanditProfileState;

#[test]
fn test_legacy_snapshot_compat() {
    let fixture_str = include_str!("fixtures/legacy_bandit_snapshot.json");
    let mut state: BanditProfileState =
        serde_json::from_str(fixture_str).expect("Legacy snapshot deserialization must succeed");

    assert_eq!(state.theta.len(), 4);
    assert_eq!(state.inv_a.len(), 16);
    assert_eq!(state.drift_threshold, 1e-4);
    assert_eq!(state.max_trace, 1e6);
    assert_eq!(state.rebuild_count(), 0);

    // Reconstruct A from A⁻¹ on load
    state.ensure_a_matrix_reconstructed();

    assert_eq!(state.a_matrix.len(), 16);
    // Identity matrix inverse is identity matrix
    for i in 0..4 {
        for j in 0..4 {
            let expected = if i == j { 1.0f64 } else { 0.0f64 };
            assert!(
                (state.a_matrix[i * 4 + j] - expected).abs() < 1e-6,
                "A matrix reconstruction mismatch at ({}, {}): got {}",
                i,
                j,
                state.a_matrix[i * 4 + j]
            );
        }
    }
}
