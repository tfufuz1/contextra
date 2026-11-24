//! Differential test comparing canonical types with legacy reference (GR6.3).

mod legacy_reference;

use contextra_ids::{DocId, EntityId, TxId};
use contextra_types::{PprAlgorithm, PprConfig, TlHfdParams};
use legacy_reference::{
    LegacyDocId, LegacyEntityId, LegacyPprAlgorithm, LegacyPprConfig, LegacyTlHfdParams, LegacyTxId,
};

#[test]
fn test_id_types_representation_equivalence() {
    let id_val = 123456789u64;

    let e = EntityId::new(id_val);
    let leg_e = LegacyEntityId(id_val);
    assert_eq!(e.inner(), leg_e.0);
    assert_eq!(
        serde_json::to_string(&e).unwrap(),
        serde_json::to_string(&leg_e).unwrap()
    );
    assert_eq!(
        bincode::serialize(&e).unwrap(),
        bincode::serialize(&leg_e).unwrap()
    );

    let d = DocId::new(id_val);
    let leg_d = LegacyDocId(id_val);
    assert_eq!(d.as_u64(), leg_d.0);
    assert_eq!(
        serde_json::to_string(&d).unwrap(),
        serde_json::to_string(&leg_d).unwrap()
    );
    assert_eq!(
        bincode::serialize(&d).unwrap(),
        bincode::serialize(&leg_d).unwrap()
    );

    let t = TxId::new(id_val);
    let leg_t = LegacyTxId(id_val);
    assert_eq!(t.inner(), leg_t.0);
    assert_eq!(
        serde_json::to_string(&t).unwrap(),
        serde_json::to_string(&leg_t).unwrap()
    );
    assert_eq!(
        bincode::serialize(&t).unwrap(),
        bincode::serialize(&leg_t).unwrap()
    );
}

#[test]
fn test_ppr_config_serialization_equivalence() {
    let params = TlHfdParams {
        sigma: 0.2,
        delta: 2.5,
        gamma: 1.2,
        max_iterations: 40,
        max_top_k_expansion: 15,
        max_hyperedge_sort_size: 32,
    };
    let leg_params = LegacyTlHfdParams {
        sigma: 0.2,
        delta: 2.5,
        gamma: 1.2,
        max_iterations: 40,
        max_top_k_expansion: 15,
        max_hyperedge_sort_size: 32,
    };

    let cfg = PprConfig {
        damping_factor: 0.85,
        max_iterations: 50,
        convergence_epsilon: 1e-5,
        algorithm: PprAlgorithm::TlHfd(params),
        warn_on_non_convergence: true,
    };
    let leg_cfg = LegacyPprConfig {
        damping_factor: 0.85,
        max_iterations: 50,
        convergence_epsilon: 1e-5,
        algorithm: LegacyPprAlgorithm::TlHfd(leg_params),
        warn_on_non_convergence: true,
    };

    let json_cfg = serde_json::to_string(&cfg).unwrap();
    let json_leg = serde_json::to_string(&leg_cfg).unwrap();
    assert_eq!(json_cfg, json_leg);
}
