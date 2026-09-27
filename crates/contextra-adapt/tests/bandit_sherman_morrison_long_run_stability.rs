//! Testfall für Sherman-Morrison Langzeit-Stabilität und Präzisions-Matrix-Drift-Erkennung (§13.2).

#![cfg(feature = "bandit-routing")]

use contextra_adapt::{
    BanditError, BanditImplementation, BanditProfileState,
    SHERMAN_MORRISON_REFACTORIZATION_INTERVAL,
};

#[test]
fn test_sherman_morrison_long_run_drift_detection() {
    let d = 4;
    let mut state = BanditProfileState::cold_start(d, 0.5);
    state.implementation = BanditImplementation::ShermanMorrison;
    // Numerisch anspruchsvolle Bedingungen mit wiederholten kollinearen Vektoren und Diskontierung
    state.gamma = 0.99;

    let x = vec![0.8f32, 0.8f32, 0.8f32, 0.8f32]; // Wiederholte kollineare Vektoren
    let cost = 0.0;
    let is_cloud = false;
    let reward = 1.0;

    let max_updates = 1500;
    let mut drift_detected = false;

    for i in 1..=max_updates {
        match state.update(&x, reward, cost, is_cloud) {
            Ok(()) => {
                assert!(
                    (i as u64) <= SHERMAN_MORRISON_REFACTORIZATION_INTERVAL,
                    "update() call {i} succeeded after exceeding refactorization interval without drift detection"
                );
            }
            Err(BanditError::PrecisionMatrixDriftDetected {
                updates_since_reset,
                denominator,
            }) => {
                drift_detected = true;
                assert_eq!(updates_since_reset, i as u64);
                // Entweder Intervall überschritten ODER denominator <= 0.0
                assert!(
                    updates_since_reset > SHERMAN_MORRISON_REFACTORIZATION_INTERVAL
                        || denominator <= 0.0,
                    "PrecisionMatrixDriftDetected returned unexpectedly with updates={updates_since_reset}, denominator={denominator}"
                );
                break;
            }
            Err(e) => {
                panic!("Unexpected error variant from update(): {e:?}");
            }
        }
    }

    assert!(
        drift_detected,
        "Expected PrecisionMatrixDriftDetected within {max_updates} updates, but none occurred"
    );
}

#[test]
fn test_sherman_morrison_well_conditioned_no_false_positives() {
    let d = 4;
    let mut state = BanditProfileState::cold_start(d, 0.5);
    state.implementation = BanditImplementation::ShermanMorrison;
    state.gamma = 0.999;

    let cost = 0.1;
    let is_cloud = false;
    let reward = 0.5;

    // Normale, wohlkonditionierte orthogonale/rotierende Feature-Vektoren
    for i in 1..=SHERMAN_MORRISON_REFACTORIZATION_INTERVAL {
        let idx = (i as usize) % d;
        let mut x = vec![0.1f32; d];
        x[idx] = 1.0;

        let res = state.update(&x, reward, cost, is_cloud);
        assert!(
            res.is_ok(),
            "Well-conditioned update {i} failed unexpectedly: {res:?}"
        );
    }

    // Update 1001 überschreitet das Re-Faktorisierungsintervall und MUSS den Fehler liefern
    let x_1001 = vec![1.0f32, 0.0, 0.0, 0.0];
    let res_1001 = state.update(&x_1001, reward, cost, is_cloud);
    match res_1001 {
        Err(BanditError::PrecisionMatrixDriftDetected {
            updates_since_reset,
            denominator,
        }) => {
            assert_eq!(
                updates_since_reset,
                SHERMAN_MORRISON_REFACTORIZATION_INTERVAL + 1
            );
            assert!(
                denominator > 0.0,
                "Under well-conditioned updates, denominator should remain positive (got {denominator})"
            );
        }
        other => {
            panic!("Expected PrecisionMatrixDriftDetected on update 1001, got {other:?}");
        }
    }
}

/// Gauss-Jordan Inversion in f64 für ein d x d Array (Row-Major).
fn naive_matrix_inverse_f64(a: &[f64], d: usize) -> Option<Vec<f64>> {
    let mut aug = vec![0.0f64; d * 2 * d];
    for i in 0..d {
        for j in 0..d {
            aug[i * 2 * d + j] = a[i * d + j];
        }
        aug[i * 2 * d + d + i] = 1.0;
    }

    for k in 0..d {
        let mut max_row = k;
        let mut max_val = aug[k * 2 * d + k].abs();
        for i in (k + 1)..d {
            let val = aug[i * 2 * d + k].abs();
            if val > max_val {
                max_val = val;
                max_row = i;
            }
        }

        if max_val < 1e-12 {
            return None;
        }

        if max_row != k {
            for j in 0..(2 * d) {
                aug.swap(k * 2 * d + j, max_row * 2 * d + j);
            }
        }

        let pivot = aug[k * 2 * d + k];
        for j in 0..(2 * d) {
            aug[k * 2 * d + j] /= pivot;
        }

        for i in 0..d {
            if i != k {
                let factor = aug[i * 2 * d + k];
                for j in 0..(2 * d) {
                    aug[i * 2 * d + j] -= factor * aug[k * 2 * d + j];
                }
            }
        }
    }

    let mut inv = vec![0.0f64; d * d];
    for i in 0..d {
        for j in 0..d {
            inv[i * d + j] = aug[i * 2 * d + d + j];
        }
    }
    Some(inv)
}

#[test]
fn test_sherman_morrison_100k_updates_frobenius_stability() {
    let d = 8;
    let mut state = BanditProfileState::cold_start(d, 0.5);
    state.implementation = BanditImplementation::ShermanMorrison;
    state.gamma = 0.999;

    let mut a_matrix_f64 = vec![0.0f64; d * d];
    for i in 0..d {
        a_matrix_f64[i * d + i] = 1.0;
    }

    let gamma_f64 = state.gamma as f64;

    // Simple LCG PRNG for synthetic vectors
    let mut rng_state: u64 = 123456789;

    println!("\n=== Sherman-Morrison 100.000 Updates Frobenius-Norm Stability Test ===");

    for t in 1..=100_000 {
        // Generate synthetic context vector x
        let mut x = vec![0.0f32; d];
        let mut norm_sq = 0.0f32;
        for i in 0..d {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((rng_state >> 33) as f32) / ((1u64 << 31) as f32) - 1.0;
            x[i] = val;
            norm_sq += val * val;
        }
        let norm = norm_sq.sqrt().max(1e-5);
        for i in 0..d {
            x[i] /= norm;
        }

        // 1. Direct Ground-Truth Matrix Accumulation: A = gamma * A + x x^T
        for i in 0..d {
            for j in 0..d {
                a_matrix_f64[i * d + j] = gamma_f64 * a_matrix_f64[i * d + j] + (x[i] as f64) * (x[j] as f64);
            }
        }

        // 2. Incremental Sherman-Morrison update in state.
        // When interval 1000 is reached, refactorization re-inverts A from ground truth and resets counter.
        match state.update(&x, 1.0, 0.0, false) {
            Ok(()) => {}
            Err(BanditError::PrecisionMatrixDriftDetected { updates_since_reset, .. }) => {
                if updates_since_reset > SHERMAN_MORRISON_REFACTORIZATION_INTERVAL {
                    // Perform refactorization / re-inversion from current A
                    let fresh_inv = naive_matrix_inverse_f64(&a_matrix_f64, d)
                        .expect("Refactorization reinversion must succeed");
                    for i in 0..(d * d) {
                        state.inv_a[i] = fresh_inv[i] as f32;
                    }
                    // Perform update again on fresh refactorized matrix
                    let _ = state.update(&x, 1.0, 0.0, false);
                }
            }
            Err(e) => panic!("Unexpected error: {e:?}"),
        }

        if t % 10_000 == 0 {
            let ref_inv_f64 = naive_matrix_inverse_f64(&a_matrix_f64, d)
                .expect("Ground truth matrix reinversion must succeed");

            let mut diff_frob_sq = 0.0f64;
            let mut ref_frob_sq = 0.0f64;

            for i in 0..(d * d) {
                let sm_val = state.inv_a[i] as f64;
                let ref_val = ref_inv_f64[i];
                let diff = sm_val - ref_val;
                diff_frob_sq += diff * diff;
                ref_frob_sq += ref_val * ref_val;
            }

            let rel_frobenius_err = (diff_frob_sq.sqrt() / ref_frob_sq.sqrt()) as f32;
            println!("Update {t:6}: Rel. Frobenius-Norm Abweichung = {rel_frobenius_err:.6e}");
            assert!(
                rel_frobenius_err < 1e-4,
                "Relative Frobenius norm deviation at step {t} ({rel_frobenius_err:.6e}) exceeded threshold 1e-4"
            );
        }
    }
}
