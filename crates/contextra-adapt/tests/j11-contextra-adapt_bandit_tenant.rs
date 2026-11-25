use contextra_adapt::bandit::{BanditImplementation, BanditProfileState, SketchMatrix};
use contextra_types::TenantId;

#[test]
fn test_bandit_tenant_sketch_initialization_wires_derive_seed() {
    let tenant_a = TenantId::try_new(1001).unwrap();
    let tenant_b = TenantId::try_new(1002).unwrap();
    let base_seed = 42;
    let d = 16;
    let k = 4;

    // Direct derive_seed verification
    let seed_a = SketchMatrix::derive_seed(&tenant_a, base_seed);
    let seed_b = SketchMatrix::derive_seed(&tenant_b, base_seed);
    assert_ne!(seed_a, seed_b, "Tenant seeds must be isolated");

    // Production API verification via BanditProfileState::init_tenant_sketch
    let mut state_a = BanditProfileState::cold_start(d, 0.5);
    state_a.implementation = BanditImplementation::SketchedProjection { projected_dim: k };
    state_a
        .init_tenant_sketch(&tenant_a, base_seed, k)
        .expect("tenant sketch init succeeded");

    let mut state_b = BanditProfileState::cold_start(d, 0.5);
    state_b.implementation = BanditImplementation::SketchedProjection { projected_dim: k };
    state_b
        .init_tenant_sketch(&tenant_b, base_seed, k)
        .expect("tenant sketch init succeeded");

    assert_eq!(state_a.seed, seed_a);
    assert_eq!(state_b.seed, seed_b);

    // Verify scoring and updating work on the initialized tenant states
    let x = vec![0.5f32; d];
    let score_a = state_a.score(&x, 0.1, false).expect("valid score");
    let score_b = state_b.score(&x, 0.1, false).expect("valid score");
    assert!(score_a.is_finite());
    assert!(score_b.is_finite());

    state_a.update(&x, 1.0, 0.1, false).expect("valid update");
    state_b.update(&x, 1.0, 0.1, false).expect("valid update");
}
