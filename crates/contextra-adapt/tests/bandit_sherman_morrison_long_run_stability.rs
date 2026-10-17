//! Testfall für Sherman-Morrison 10^5 Random Updates Stabilität und Cholesky Reference Tolerance.

#![cfg(feature = "bandit-routing")]

use contextra_adapt::{cholesky_invert_f64, BanditImplementation, BanditProfileState};

#[test]
fn test_sherman_morrison_100k_updates_stability_against_cholesky_reference() {
    let d = 8;
    let mut state = BanditProfileState::cold_start(d, 0.5);
    state.implementation = BanditImplementation::ShermanMorrison;
    state.gamma = 0.999;

    let mut a_ref_f64 = vec![0.0f64; d * d];
    for i in 0..d {
        a_ref_f64[i * d + i] = 1.0;
    }

    let gamma_f64 = state.gamma as f64;
    let mut rng_state: u64 = 987654321;

    let total_updates = 100_000;

    for t in 1..=total_updates {
        // Generate random context vector x using deterministic SplitMix PRNG
        let mut x = vec![0.0f32; d];
        let mut norm_sq = 0.0f32;
        for i in 0..d {
            rng_state = rng_state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = rng_state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            let val = (((z ^ (z >> 31)) >> 11) as f64 / ((1u64 << 53) as f64)) as f32 - 0.5;
            x[i] = val;
            norm_sq += val * val;
        }
        let norm = norm_sq.sqrt().max(1e-5);
        for i in 0..d {
            x[i] /= norm;
        }

        // Update ground truth f64 A_ref: A_ref = gamma * A_ref + x x^T
        for i in 0..d {
            for j in 0..d {
                a_ref_f64[i * d + j] =
                    gamma_f64 * a_ref_f64[i * d + j] + (x[i] as f64) * (x[j] as f64);
            }
        }

        // Update bandit profile state. Must never return error or stop learning!
        let update_res = state.update(&x, 1.0, 0.1, false);
        assert!(
            update_res.is_ok(),
            "Update {t} failed unexpectedly: {update_res:?}"
        );

        // Every 25,000 steps, check tolerance against f64 Cholesky reference
        if t % 25_000 == 0 {
            let inv_a_ref =
                cholesky_invert_f64(&a_ref_f64, d).expect("f64 Cholesky inversion must succeed");

            // Evaluate variance sqrt(x^T A^{-1} x) under Sherman-Morrison state vs Cholesky reference
            let mut var_sm = 0.0f64;
            let mut var_ref = 0.0f64;
            for i in 0..d {
                for j in 0..d {
                    var_sm += (x[i] as f64) * (state.inv_a[i * d + j] as f64) * (x[j] as f64);
                    var_ref += (x[i] as f64) * inv_a_ref[i * d + j] * (x[j] as f64);
                }
            }

            let std_sm = var_sm.sqrt();
            let std_ref = var_ref.sqrt();
            let abs_diff = (std_sm - std_ref).abs();

            assert!(
                abs_diff < 1e-3,
                "Standard deviation tolerance exceeded at step {t}: SM={std_sm:.6}, Ref={std_ref:.6}, diff={abs_diff:.6e}"
            );
        }
    }

    assert!(
        state.rebuild_count() > 0,
        "Rebuilds must occur periodically to prevent matrix drift across 10^5 updates"
    );
}
