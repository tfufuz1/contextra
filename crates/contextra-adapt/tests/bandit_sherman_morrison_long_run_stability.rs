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
