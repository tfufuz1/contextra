//! Numerische und verhaltensbasierte Testfälle für Flow-Corrected Thompson Sampling (§21.3).

#![cfg(feature = "flow-corrected-thompson")]

use contextra_adapt::flow_thompson::{
    ring3_background_task_token, FcTsArmSet, FcTsConfig, FcTsError, FcTsRng,
    FlowCorrectedThompsonBandit, SplitMix64,
};

/// Hilfsfunktion für Gauß-Elimination zur Inversion einer 3x3 Matrix.
fn gaussian_elimination_invert_3x3(m: &[f32; 9]) -> Option<[f32; 9]> {
    let mut aug = [
        [m[0], m[1], m[2], 1.0, 0.0, 0.0],
        [m[3], m[4], m[5], 0.0, 1.0, 0.0],
        [m[6], m[7], m[8], 0.0, 0.0, 1.0],
    ];

    for i in 0..3 {
        let mut pivot = i;
        for j in (i + 1)..3 {
            if aug[j][i].abs() > aug[pivot][i].abs() {
                pivot = j;
            }
        }
        if aug[pivot][i].abs() < 1e-8 {
            return None;
        }
        aug.swap(i, pivot);

        let div = aug[i][i];
        for j in 0..6 {
            aug[i][j] /= div;
        }

        for k in 0..3 {
            if k != i {
                let factor = aug[k][i];
                for j in 0..6 {
                    aug[k][j] -= factor * aug[i][j];
                }
            }
        }
    }

    let mut inv = [0.0f32; 9];
    for i in 0..3 {
        for j in 0..3 {
            inv[i * 3 + j] = aug[i][j + 3];
        }
    }
    Some(inv)
}

#[test]
fn test_reproducible_arm_selection_same_seed() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = FcTsConfig {
        dim: 4,
        ..Default::default()
    };
    let arm0 = FlowCorrectedThompsonBandit::new(cfg.clone())?;
    let arm1 = FlowCorrectedThompsonBandit::new(cfg)?;

    let arm_set1 = FcTsArmSet {
        arms: vec![arm0.clone(), arm1.clone()],
    };
    let arm_set2 = FcTsArmSet {
        arms: vec![arm0, arm1],
    };

    let mut rng1 = SplitMix64::new(12345);
    let mut rng2 = SplitMix64::new(12345);

    let ctx = vec![0.5f32, -0.2f32, 0.8f32, 0.1f32];

    for _ in 0..100 {
        let sel1 = arm_set1.select_arm(&ctx, &mut rng1)?;
        let sel2 = arm_set2.select_arm(&ctx, &mut rng2)?;
        assert_eq!(
            sel1, sel2,
            "Arm selection must be bit-identical given identical RNG seed"
        );
    }
    Ok(())
}

#[test]
fn test_parameter_mu_convergence_linear_target() -> Result<(), Box<dyn std::error::Error>> {
    // Zielgewicht w* = [1.5, -0.8, 2.0]
    let w_star = [1.5f32, -0.8f32, 2.0f32];
    let cfg = FcTsConfig {
        dim: 3,
        lambda: 0.1,
        noise_var: 0.01,
        window_capacity: 500,
        ..Default::default()
    };

    let mut bandit = FlowCorrectedThompsonBandit::new(cfg)?;
    let mut rng = SplitMix64::new(999);

    for t in 0..300 {
        // Generiere synthetischen Kontext
        let x = [
            rng.next_f32() * 2.0 - 1.0,
            rng.next_f32() * 2.0 - 1.0,
            rng.next_f32() * 2.0 - 1.0,
        ];
        let clean_reward = x[0] * w_star[0] + x[1] * w_star[1] + x[2] * w_star[2];
        let noise = (rng.next_f32() - 0.5) * 0.1;
        let r = clean_reward + noise;

        bandit.update_with_flow(&x, r, t as u64, 1.0)?;
    }

    let mu = bandit.mu();
    for i in 0..3 {
        let diff = (mu[i] - w_star[i]).abs();
        assert!(
            diff < 0.15,
            "Dimension {i}: mu[{i}] = {:.4} did not converge sufficiently to w*[{i}] = {:.4} (diff={:.4})",
            mu[i], w_star[i], diff
        );
    }
    Ok(())
}

#[test]
fn test_sherman_morrison_matches_direct_matrix_inversion() -> Result<(), Box<dyn std::error::Error>>
{
    let d = 3;
    let cfg = FcTsConfig {
        dim: d,
        lambda: 1.0,
        noise_var: 1.0,
        ..Default::default()
    };

    let mut bandit = FlowCorrectedThompsonBandit::new(cfg)?;

    // Akkumuliere A = λI + sum(x xᵀ) direkt für exakte Matrix-Inversionsreferenz
    let mut exact_a = [1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    let contexts = [
        [0.8f32, 0.2, -0.5],
        [0.1f32, -0.9, 0.4],
        [-0.6f32, 0.3, 0.7],
    ];

    for (t, x) in contexts.iter().enumerate() {
        for i in 0..d {
            for j in 0..d {
                exact_a[i * d + j] += x[i] * x[j];
            }
        }
        bandit.update_with_flow(x, 1.0, t as u64, 1.0)?;
    }

    let direct_inv_a = gaussian_elimination_invert_3x3(&exact_a)
        .ok_or("Direct Gaussian elimination inversion failed")?;

    let sm_inv_a = bandit.inv_a();

    for i in 0..9 {
        let diff = (sm_inv_a[i] - direct_inv_a[i]).abs();
        assert!(
            diff < 1e-3,
            "Sherman-Morrison inverse mismatched direct inversion at index {i}: SM={:.6}, Direct={:.6}, diff={:.6}",
            sm_inv_a[i], direct_inv_a[i], diff
        );
    }
    Ok(())
}

#[test]
fn test_error_handling_and_state_non_mutation() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = FcTsConfig {
        dim: 3,
        ..Default::default()
    };
    let mut bandit = FlowCorrectedThompsonBandit::new(cfg)?;

    // 1. Initiales Valids Update
    bandit.update_with_flow(&[1.0, 0.0, 0.0], 1.0, 0, 1.0)?;
    let mu_before = bandit.mu().to_vec();
    let inv_a_before = bandit.inv_a().to_vec();

    // 2. Dimension Mismatch
    let err_dim = match bandit.update_with_flow(&[1.0, 0.0], 1.0, 1, 1.0) {
        Err(e) => e,
        Ok(()) => return Err("Expected dimension mismatch error".into()),
    };
    assert_eq!(
        err_dim,
        FcTsError::DimensionMismatch {
            expected: 3,
            actual: 2
        }
    );
    assert_eq!(bandit.mu(), &mu_before[..]);
    assert_eq!(bandit.inv_a(), &inv_a_before[..]);

    // 3. NaN in Context
    let err_nan = match bandit.update_with_flow(&[f32::NAN, 0.0, 0.0], 1.0, 2, 1.0) {
        Err(e) => e,
        Ok(()) => return Err("Expected non-finite error".into()),
    };
    assert_eq!(err_nan, FcTsError::NonFinite);
    assert_eq!(bandit.mu(), &mu_before[..]);
    assert_eq!(bandit.inv_a(), &inv_a_before[..]);

    // 4. Negative Konfidenz
    let err_neg_w = match bandit.update_with_flow(&[1.0, 0.0, 0.0], 1.0, 3, -0.5) {
        Err(e) => e,
        Ok(()) => return Err("Expected non-finite error".into()),
    };
    assert_eq!(err_neg_w, FcTsError::NonFinite);
    assert_eq!(bandit.mu(), &mu_before[..]);
    assert_eq!(bandit.inv_a(), &inv_a_before[..]);

    Ok(())
}

#[test]
fn test_ring_buffer_fixed_capacity_overwriting() -> Result<(), Box<dyn std::error::Error>> {
    let cap = 5;
    let cfg = FcTsConfig {
        dim: 2,
        window_capacity: cap,
        ..Default::default()
    };
    let mut bandit = FlowCorrectedThompsonBandit::new(cfg)?;

    // Schreibe 10 Einträge in einen 5er Ringpuffer
    for t in 0..10 {
        let x = [t as f32, (t * 2) as f32];
        bandit.update_with_flow(&x, 1.0, t as u64, 1.0)?;
    }

    // Stelle sicher, dass die Kapazität eingehalten wurde
    let token = ring3_background_task_token();
    bandit.recompute_drift_rate_from_window(&token);

    assert!(bandit.drift_rate().iter().all(|&v| v.is_finite()));
    Ok(())
}

#[test]
fn test_drift_rate_recovery_linear_drift() -> Result<(), Box<dyn std::error::Error>> {
    let d = 2;
    let cfg = FcTsConfig {
        dim: d,
        window_capacity: 50,
        drift_ridge: 0.1,
        max_drift_norm: 10.0,
        ..Default::default()
    };

    let mut bandit = FlowCorrectedThompsonBandit::new(cfg)?;
    let true_drift_rate = [0.5f32, -0.3f32];

    for t in 0..30 {
        let x = [1.0f32, 0.5f32];
        let dt = (30 - t) as f32;
        // Synthetische Belohnung mit konstantem zeitlichen Drift
        let r = dt * (x[0] * true_drift_rate[0] + x[1] * true_drift_rate[1]);

        bandit.update_with_flow(&x, r, t as u64, 1.0)?;
    }

    let token = ring3_background_task_token();
    bandit.recompute_drift_rate_from_window(&token);

    let estimated_drift = bandit.drift_rate();
    // Drift-Vektor sollte in die Richtung von true_drift_rate zeigen (Skalarprodukt > 0)
    let dot = estimated_drift[0] * true_drift_rate[0] + estimated_drift[1] * true_drift_rate[1];
    assert!(
        dot > 0.0,
        "Estimated drift_rate ({:?}) should align with true_drift_rate ({:?}), dot = {}",
        estimated_drift,
        true_drift_rate,
        dot
    );
    Ok(())
}
