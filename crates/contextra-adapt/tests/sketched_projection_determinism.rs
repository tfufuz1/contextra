// FILE-CONTEXT
// ZWECK: Determinismus-Tests für SketchedProjection (§13.2 / P28).
// STAND: TS:2026-09-26T00:00:00Z

use contextra_adapt::bandit::{
    BanditImplementation, BanditProfileState, SketchMatrix,
};
use contextra_types::TenantId;

#[test]
fn test_sketched_projection_matrix_determinism_100_runs() {
    let seed = 987654321u64;
    let original_dim = 384;
    let projected_dim = 64;

    let base_matrix = SketchMatrix::from_seed(seed, original_dim, projected_dim)
        .expect("valid sketch matrix construction");

    for i in 0..100 {
        let current_matrix = SketchMatrix::from_seed(seed, original_dim, projected_dim)
            .expect("valid sketch matrix construction");

        assert_eq!(
            base_matrix.data, current_matrix.data,
            "SketchMatrix data mismatch at run {i}"
        );
        assert_eq!(base_matrix.seed, current_matrix.seed);
        assert_eq!(base_matrix.original_dim, current_matrix.original_dim);
        assert_eq!(base_matrix.projected_dim, current_matrix.projected_dim);
    }
}

#[test]
fn test_sketched_projection_vector_projection_determinism() {
    let seed = 424242u64;
    let original_dim = 128;
    let projected_dim = 32;

    let matrix = SketchMatrix::from_seed(seed, original_dim, projected_dim)
        .expect("valid sketch matrix construction");

    let x: Vec<f32> = (0..original_dim)
        .map(|i| (i as f32 * 0.013).sin())
        .collect();

    let expected_projection = matrix.project(&x);

    for run in 0..100 {
        let proj = matrix.project(&x);
        assert_eq!(
            expected_projection, proj,
            "Projection result mismatch at run {run}"
        );
    }
}

#[test]
fn test_sketched_projection_score_and_update_determinism() {
    let original_dim = 64;
    let projected_dim = 16;
    let seed = 123456789u64;
    let x: Vec<f32> = (0..original_dim)
        .map(|i| ((i + 1) as f32 * 0.05).cos())
        .collect();

    let mut state1 = BanditProfileState::cold_start(original_dim, 0.5);
    state1.seed = seed;
    state1.implementation = BanditImplementation::SketchedProjection { projected_dim };

    let initial_score = state1.score(&x, 0.1, false).expect("valid score");

    for _ in 0..10 {
        state1.update(&x, 0.9, 0.1, false).expect("valid update");
    }
    let updated_score = state1.score(&x, 0.1, false).expect("valid score");

    // Repeat identical sequence from fresh cold-start and verify bit-identity
    for run in 0..100 {
        let mut state_run = BanditProfileState::cold_start(original_dim, 0.5);
        state_run.seed = seed;
        state_run.implementation = BanditImplementation::SketchedProjection { projected_dim };

        let score_run = state_run.score(&x, 0.1, false).expect("valid score");
        assert_eq!(
            initial_score.to_bits(),
            score_run.to_bits(),
            "Initial score mismatch at run {run}"
        );

        for _ in 0..10 {
            state_run.update(&x, 0.9, 0.1, false).expect("valid update");
        }

        let updated_score_run = state_run.score(&x, 0.1, false).expect("valid score");
        assert_eq!(
            updated_score.to_bits(),
            updated_score_run.to_bits(),
            "Updated score mismatch at run {run}"
        );
    }
}

#[test]
fn test_sketched_projection_tenant_seed_isolation() {
    let tenant_a = TenantId::try_new(100).expect("valid tenant_id");
    let tenant_b = TenantId::try_new(200).expect("valid tenant_id");
    let base_seed = 77777u64;

    let seed_a = SketchMatrix::derive_seed(&tenant_a, base_seed);
    let seed_b = SketchMatrix::derive_seed(&tenant_b, base_seed);

    assert_ne!(
        seed_a, seed_b,
        "Derived seeds for different tenant IDs must be distinct"
    );

    let matrix_a = SketchMatrix::from_seed(seed_a, 128, 32).expect("valid matrix_a");
    let matrix_b = SketchMatrix::from_seed(seed_b, 128, 32).expect("valid matrix_b");

    assert_ne!(
        matrix_a.data, matrix_b.data,
        "Sketch matrices for distinct tenants must not collide"
    );
}
