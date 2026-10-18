//! Integrationstests für Auto-Recovery, Atomarität, Determinismus und numerische Stabilität des Bandits.

#![cfg(feature = "bandit-routing")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::needless_range_loop)]

use contextra_adapt::{BanditError, BanditImplementation, BanditProfileState, SeededRng};
use contextra_ports::SeededRng as _;

/// Referenz-RLS in f64 (Discounted Recursive Least Squares) zur Genauigkeitsprüfung.
struct RlsF64 {
    d: usize,
    theta: Vec<f64>,
    inv_a: Vec<f64>,
    gamma: f64,
}

impl RlsF64 {
    fn new(d: usize, gamma: f64) -> Self {
        let mut inv_a = vec![0.0f64; d * d];
        for i in 0..d {
            inv_a[i * d + i] = 1.0;
        }
        Self {
            d,
            theta: vec![0.0f64; d],
            inv_a,
            gamma,
        }
    }

    fn update(&mut self, x: &[f32], r_adj: f64) {
        let d = self.d;
        let gamma_inv = 1.0 / self.gamma;

        // v = gamma^{-1} * inv_a * x
        let mut v = vec![0.0f64; d];
        let mut xt_v = 0.0f64;
        for i in 0..d {
            let mut row_dot = 0.0f64;
            for j in 0..d {
                row_dot += self.inv_a[i * d + j] * (x[j] as f64);
            }
            v[i] = gamma_inv * row_dot;
            xt_v += (x[i] as f64) * v[i];
        }

        let denom = 1.0 + xt_v;
        let mut pred = 0.0f64;
        for j in 0..d {
            pred += self.theta[j] * (x[j] as f64);
        }
        let residual = r_adj - pred;

        let mut new_inv_a = vec![0.0f64; d * d];
        for i in 0..d {
            let k_i = v[i] / denom;
            self.theta[i] += residual * k_i;
            for j in 0..d {
                new_inv_a[i * d + j] = gamma_inv * self.inv_a[i * d + j] - k_i * v[j];
            }
        }
        self.inv_a = new_inv_a;
    }
}

fn generate_seeded_vector(rng: &SeededRng, d: usize) -> Vec<f32> {
    let mut vec = Vec::with_capacity(d);
    let mut norm_sq = 0.0f32;
    for _ in 0..d {
        let val = (rng.next_unit_f64() as f32) * 2.0 - 1.0;
        vec.push(val);
        norm_sq += val * val;
    }
    let norm = norm_sq.sqrt().max(1e-5);
    for v in vec.iter_mut() {
        *v /= norm;
    }
    vec
}

#[test]
fn test_5000_updates_no_err_and_f64_reference_accuracy() {
    let d = 4;
    let implementations = [
        BanditImplementation::DiagonalApproximation,
        BanditImplementation::SketchedProjection { projected_dim: 2 },
        BanditImplementation::ShermanMorrison,
    ];

    for impl_variant in implementations {
        let mut state = BanditProfileState::cold_start(d, 0.5);
        state.implementation = impl_variant;
        state.gamma = 0.999;

        let mut rls_ref = RlsF64::new(d, 0.999);
        let rng = SeededRng::new(42);

        for step in 1..=5000 {
            let x = generate_seeded_vector(&rng, d);
            let cost = 0.1f32;
            let is_cloud = false;
            let reward = 0.8f32;
            let r_adj = reward - state.lambda * cost;

            let res = state.update(&x, reward, cost, is_cloud);
            assert!(
                res.is_ok(),
                "Update at step {step} failed for {:?}: {:?}",
                impl_variant,
                res
            );

            // θ und A⁻¹ / sigma_sq müssen endlich sein
            assert!(
                state.theta.iter().all(|val| val.is_finite()),
                "Non-finite theta at step {step} for {:?}",
                impl_variant
            );
            assert!(
                state.inv_a.iter().all(|val| val.is_finite()),
                "Non-finite inv_a at step {step} for {:?}",
                impl_variant
            );

            if impl_variant == BanditImplementation::ShermanMorrison {
                rls_ref.update(&x, r_adj as f64);
            }
        }

        if impl_variant == BanditImplementation::ShermanMorrison {
            // Vergleiche theta-Vorhersagefehler gegen f64-Referenz
            let test_x = generate_seeded_vector(&rng, d);
            let pred_f32: f32 = state
                .theta
                .iter()
                .zip(test_x.iter())
                .map(|(t, xi)| t * xi)
                .sum();
            let pred_f64: f64 = rls_ref
                .theta
                .iter()
                .zip(test_x.iter())
                .map(|(t, &xi)| t * (xi as f64))
                .sum();

            let diff = (pred_f32 as f64 - pred_f64).abs();
            println!("Prediction error f32 vs f64 RLS at step 5000: {diff:.6e}");
            assert!(
                diff < 0.2,
                "Prediction error vs f64 RLS too high at step 5000: {diff}"
            );
        }
    }
}

#[test]
fn test_state_atomicity_on_errors() {
    let d = 4;
    let mut state = BanditProfileState::cold_start(d, 0.5);
    let x_valid = vec![0.5f32, 0.5, 0.5, 0.5];

    // Perform a valid update first
    state.update(&x_valid, 0.8, 0.1, false).unwrap();

    let state_before = state.clone();

    // 1. Dimension Mismatch Error
    let x_mismatch = vec![0.5f32, 0.5];
    let err_mismatch = state.update(&x_mismatch, 0.8, 0.1, false);
    assert!(matches!(
        err_mismatch,
        Err(BanditError::DimensionMismatch { .. })
    ));
    assert_eq!(state, state_before, "State mutated on DimensionMismatch!");

    // 2. Non-finite input vector (NaN)
    let x_nan = vec![0.5f32, f32::NAN, 0.5, 0.5];
    let err_nan = state.update(&x_nan, 0.8, 0.1, false);
    assert!(err_nan.is_err(), "NaN input must return error");
    assert_eq!(state, state_before, "State mutated on NaN vector input!");

    // 3. Non-finite reward (Inf)
    let err_inf_reward = state.update(&x_valid, f32::INFINITY, 0.1, false);
    assert!(err_inf_reward.is_err(), "Inf reward must return error");
    assert_eq!(state, state_before, "State mutated on Inf reward!");

    // 4. Non-finite cost (NaN)
    let err_nan_cost = state.update(&x_valid, 0.8, f32::NAN, false);
    assert!(err_nan_cost.is_err(), "NaN cost must return error");
    assert_eq!(state, state_before, "State mutated on NaN cost!");
}

#[test]
fn test_determinism_same_seed_inputs_identical_state() {
    let d = 4;
    let mut state1 = BanditProfileState::cold_start(d, 0.5);
    let mut state2 = BanditProfileState::cold_start(d, 0.5);

    let rng1 = SeededRng::new(12345);
    let rng2 = SeededRng::new(12345);

    for _ in 0..500 {
        let x1 = generate_seeded_vector(&rng1, d);
        let x2 = generate_seeded_vector(&rng2, d);

        state1.update(&x1, 0.7, 0.05, false).unwrap();
        state2.update(&x2, 0.7, 0.05, false).unwrap();
    }

    assert_eq!(
        state1, state2,
        "States diverged despite identical seeds and inputs"
    );
}

#[test]
fn test_discounting_f32_overflow_check() {
    // Befund (d): Bei γ = 0.999 und 1000 Updates in unangeregten Richtungen:
    // f(t) = gamma^{-1}^t = (1 / 0.999)^1000 = (1.001001001)^1000 ≈ 2.7169
    // f32::MAX ≈ 3.4e38.
    // Daher läuft f32 bei 1000 Updates ohne Anregung NICHT über.
    let gamma: f32 = 0.999;
    let gamma_inv = 1.0 / gamma;
    let mut factor = 1.0f32;
    for _ in 0..1000 {
        factor *= gamma_inv;
    }
    println!("Discounting growth factor after 1000 steps: {factor}");
    assert!(
        factor < 10.0,
        "Discounting growth factor unexpectedly large: {factor}"
    );
    assert!(factor.is_finite());
}

#[test]
fn test_inconsistent_deserialized_lengths_no_panic() {
    // Befund (e): Test ob inkonsistente Längen in deserialisiertem JSON paniken.
    let inconsistent_json = r#"{
        "theta": [0.1, 0.2],
        "sigma_sq": [1.0, 1.0, 1.0, 1.0],
        "inv_a": [1.0, 0.0, 0.0, 1.0],
        "alpha": 0.5,
        "lambda": 0.1,
        "mu": 0.2,
        "gamma": 0.999,
        "drift_gamma": 0.95,
        "drift_steps_remaining": 0,
        "drift_decay_window": 50,
        "implementation": "DiagonalApproximation"
    }"#;

    let state: BanditProfileState =
        serde_json::from_str(inconsistent_json).expect("Deserialization of state must succeed");

    let x = vec![1.0f32, 0.5f32];
    // Attempting update / score should handle dimension checking safely without panic
    let score_res =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| state.score(&x, 0.0, false)));
    assert!(
        score_res.is_ok(),
        "score() panicked on inconsistent lengths!"
    );

    let update_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut st = state.clone();
        st.update(&x, 1.0, 0.0, false)
    }));
    assert!(
        update_res.is_ok(),
        "update() panicked on inconsistent lengths!"
    );
}
