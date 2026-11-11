//! Differenz- und Äquivalenztest: contextra-engine::decay_controller vs. contextra-adapt::decay_controller
//!
//! Normativ: docs/SPEC.md §5.4 (Differenztests: Regler f64/f32-bitgleich), S-60.
//! Geschützt mit `#![cfg(feature = "adaptive-candidate-pool-sizing")]`.
//! Vergleicht alle gemeinsamen Methoden auf f32-Bitgleichheit bei identischen Eingaben.

#![cfg(not(loom))]
#![cfg(feature = "adaptive-candidate-pool-sizing")]

#[path = "legacy_reference/decay_controller_legacy.rs"]
mod decay_controller_legacy;

use contextra_engine::decay_controller::{
    AdaptiveDecayController as EngineController,
    DecayControllerConfig as EngineConfig,
    DecaySignalInputs as EngineInputs,
};
use contextra_adapt::decay_controller::{
    AdaptiveDecayController as AdaptController,
    DecayControllerConfig as AdaptConfig,
    DecaySignalInputs as AdaptInputs,
};
use proptest::prelude::*;

#[test]
fn test_regulator_deterministic_edge_cases() {
    let configs = vec![
        (0.0, 1, 0.0),
        (2.0, 10_000, 0.01),
        (5.0, 100_000, 0.5),
        (0.5, 500, 0.99),
    ];

    let input_cases = vec![
        (0.0, 0.0, 0.0, 0.0),
        (1.0, 1.0, 0.6, 0.4),
        (0.5, 0.5, 0.5, 0.5),
        (0.0, 1.0, 1.0, 0.0),
        (1.0, 0.0, 0.0, 1.0),
    ];

    for (kappa, half_life_tx, threshold) in configs {
        let eng_cfg = EngineConfig {
            kappa,
            base_half_life_tx: half_life_tx,
            eviction_threshold: threshold,
        };
        let adapt_cfg = AdaptConfig {
            kappa,
            base_half_life_tx: half_life_tx,
            eviction_threshold: threshold,
        };

        let eng_ctrl = EngineController::new(eng_cfg);
        let adapt_ctrl = AdaptController::new(adapt_cfg);

        for &(tomb, query, w_tomb, w_query) in &input_cases {
            let eng_in = EngineInputs {
                tombstone_ratio: tomb,
                query_load_inverse: query,
                w_tombstone: w_tomb,
                w_query,
            };
            let adapt_in = AdaptInputs {
                tombstone_ratio: tomb,
                query_load_inverse: query,
                w_tombstone: w_tomb,
                w_query,
            };

            // 1. system_temperature
            let eng_temp = eng_ctrl.system_temperature(&eng_in);
            let adapt_temp = adapt_ctrl.system_temperature(&adapt_in);
            assert_eq!(
                eng_temp.to_bits(), adapt_temp.to_bits(),
                "system_temperature bit-equality failed: eng={}, adapt={}",
                eng_temp, adapt_temp
            );

            // 2. effective_half_life
            let eng_hl = eng_ctrl.effective_half_life(eng_temp);
            let adapt_hl = adapt_ctrl.effective_half_life(adapt_temp);
            assert_eq!(
                eng_hl.to_bits(), adapt_hl.to_bits(),
                "effective_half_life bit-equality failed: eng={}, adapt={}",
                eng_hl, adapt_hl
            );

            // 3. effective_score and should_evict
            for elapsed_tx in [0, 1, 10, 100, 1_000, 10_000, 100_000, u64::MAX / 2] {
                for base_score in [0.0f32, 0.1, 0.5, 1.0] {
                    let eng_score = eng_ctrl.effective_score(base_score, elapsed_tx, &eng_in);
                    let adapt_score = adapt_ctrl.effective_score(base_score, elapsed_tx, &adapt_in);
                    assert_eq!(
                        eng_score.to_bits(), adapt_score.to_bits(),
                        "effective_score bit-equality failed for elapsed_tx={}, base_score={}",
                        elapsed_tx, base_score
                    );

                    let eng_evict = eng_ctrl.should_evict(base_score, elapsed_tx, &eng_in);
                    let adapt_evict = adapt_ctrl.should_evict(base_score, elapsed_tx, &adapt_in);
                    assert_eq!(
                        eng_evict, adapt_evict,
                        "should_evict parity failed for elapsed_tx={}, base_score={}",
                        elapsed_tx, base_score
                    );
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]
    #[test]
    fn prop_regulator_differential_bit_equality(
        kappa in 0.0f32..10.0f32,
        half_life_tx in 1u64..1_000_000u64,
        threshold in 0.0f32..1.0f32,
        tombstone_ratio in 0.0f32..1.0f32,
        query_load_inverse in 0.0f32..1.0f32,
        w_tombstone in 0.0f32..1.0f32,
        w_query in 0.0f32..1.0f32,
        base_score in 0.0f32..1.0f32,
        elapsed_tx in 0u64..1_000_000u64,
    ) {
        let eng_cfg = EngineConfig {
            kappa,
            base_half_life_tx: half_life_tx,
            eviction_threshold: threshold,
        };
        let adapt_cfg = AdaptConfig {
            kappa,
            base_half_life_tx: half_life_tx,
            eviction_threshold: threshold,
        };

        let eng_ctrl = EngineController::new(eng_cfg);
        let adapt_ctrl = AdaptController::new(adapt_cfg);

        let eng_in = EngineInputs {
            tombstone_ratio,
            query_load_inverse,
            w_tombstone,
            w_query,
        };
        let adapt_in = AdaptInputs {
            tombstone_ratio,
            query_load_inverse,
            w_tombstone,
            w_query,
        };

        let eng_temp = eng_ctrl.system_temperature(&eng_in);
        let adapt_temp = adapt_ctrl.system_temperature(&adapt_in);
        prop_assert_eq!(eng_temp.to_bits(), adapt_temp.to_bits());

        let eng_hl = eng_ctrl.effective_half_life(eng_temp);
        let adapt_hl = adapt_ctrl.effective_half_life(adapt_temp);
        prop_assert_eq!(eng_hl.to_bits(), adapt_hl.to_bits());

        let eng_score = eng_ctrl.effective_score(base_score, elapsed_tx, &eng_in);
        let adapt_score = adapt_ctrl.effective_score(base_score, elapsed_tx, &adapt_in);
        prop_assert_eq!(eng_score.to_bits(), adapt_score.to_bits());

        let eng_evict = eng_ctrl.should_evict(base_score, elapsed_tx, &eng_in);
        let adapt_evict = adapt_ctrl.should_evict(base_score, elapsed_tx, &adapt_in);
        prop_assert_eq!(eng_evict, adapt_evict);
    }
}
