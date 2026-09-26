//! Property-Tests und Unit-Tests für Off-Policy-Kompatibilität und Bandit-Default-Flip-Gate (§B.3.2).

use contextra_adapt::{
    BanditDefaultFlipGate, BanditShadowReport, DefaultBanditFlipGate,
    DiagonalApproximationBandit, FcTsSamplingDistribution, OffPolicyCompatibility, OffPolicyError,
    MAX_CUMULATIVE_REGRET_THRESHOLD, MIN_BANDIT_SHADOW_SAMPLES,
};
use contextra_types::{ContextraError, RetrievalStrategy};
use proptest::prelude::*;

const STRATEGIES: [RetrievalStrategy; 4] = [
    RetrievalStrategy::Vector,
    RetrievalStrategy::Text,
    RetrievalStrategy::Graph,
    RetrievalStrategy::Hybrid,
];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property-Test über zufällig generierte Propensity-Verteilungspaare (§B.3.2).
    ///
    /// Generiert zufällige Wahrscheinlichkeiten im Intervall [0.0, 1.0] für Referenz-Policy
    /// und FC-TS-Sampling-Verteilung über alle 4 RetrievalStrategy-Arme.
    #[test]
    fn prop_off_policy_positivity_verification(
        ref_props in proptest::array::uniform4(0.0f32..=1.0f32),
        fcts_probs in proptest::array::uniform4(0.0f32..=1.0f32),
    ) {
        let mut ref_map = std::collections::HashMap::new();
        let mut fcts_map = std::collections::HashMap::new();

        for (i, &strat) in STRATEGIES.iter().enumerate() {
            ref_map.insert(strat, ref_props[i]);
            fcts_map.insert(strat, fcts_probs[i]);
        }

        let ref_policy = DiagonalApproximationBandit::new(ref_map);
        let fcts_dist = FcTsSamplingDistribution::new(fcts_map);

        let result1 = fcts_dist.verify_positivity(&ref_policy);
        let result2 = fcts_dist.verify_positivity(&ref_policy);

        // P28 Invariante: Determinismus — zweimalige Ausführung liefert exakt dasselbe Ergebnis
        prop_assert_eq!(&result1, &result2);

        // Prüfe Positivitäts-Bedingung:
        // Ein Fehler muss genau dann auftreten, wenn mindestens eine Strategie
        // FC-TS-Sampling-Prob > 0 hat, aber Referenz-Propensity <= 0.0 aufweist.
        let expected_violation = STRATEGIES.iter().find(|&&strat| {
            let fcts_p = fcts_dist.probability(strat);
            let ref_p = ref_policy.propensity(strat);
            fcts_p > 0.0 && ref_p <= 0.0
        });

        match (result1, expected_violation) {
            (Err(OffPolicyError::ZeroPropensityViolation(violating_strat)), Some(&expected_strat)) => {
                let fcts_p = fcts_dist.probability(violating_strat);
                let ref_p = ref_policy.propensity(violating_strat);
                prop_assert!(fcts_p > 0.0);
                prop_assert!(ref_p <= 0.0);
                // Der gemeldete verletzende Arm muss tatsächlich die Positivitätsannahme verletzen
                prop_assert_eq!(violating_strat, expected_strat);
            }
            (Ok(()), None) => {
                // Alle Arme mit fcts_p > 0.0 haben ref_p > 0.0
                for &strat in &STRATEGIES {
                    if fcts_dist.probability(strat) > 0.0 {
                        prop_assert!(ref_policy.propensity(strat) > 0.0);
                    }
                }
            }
            (res, exp) => {
                prop_assert!(false, "Mismatch in verify_positivity: got {:?}, expected violation {:?}", res, exp);
            }
        }
    }
}

#[test]
fn test_off_policy_error_conversion_to_contextra_error() {
    let err = OffPolicyError::ZeroPropensityViolation(RetrievalStrategy::Graph);
    let contextra_err: ContextraError = err.clone().into();

    match contextra_err {
        ContextraError::InvalidInput(msg) => {
            assert!(msg.contains("Graph"));
            assert!(msg.contains("zero propensity"));
        }
        other => panic!("Expected InvalidInput, got {:?}", other),
    }
}

#[test]
fn test_bandit_default_flip_gate_decision_boundaries() {
    let gate = DefaultBanditFlipGate;

    // 1. Alle Bedingungen perfekt erfüllt -> should_flip == true
    let valid_report = BanditShadowReport {
        sample_count: MIN_BANDIT_SHADOW_SAMPLES,
        cumulative_regret: MAX_CUMULATIVE_REGRET_THRESHOLD,
        p99_latency_delta_us: 0.0,
        reward_improvement_ratio: Some(0.12),
    };
    assert!(gate.should_flip(&valid_report));

    // 2. Probenanzahl knapp unter Schwelle (9_999 < 10_000) -> should_flip == false
    let insufficient_samples_report = BanditShadowReport {
        sample_count: MIN_BANDIT_SHADOW_SAMPLES - 1,
        cumulative_regret: -10.0,
        p99_latency_delta_us: -5.0,
        reward_improvement_ratio: Some(0.20),
    };
    assert!(!gate.should_flip(&insufficient_samples_report));

    // 3. Kumulatives Regret positiv (> 0.0) -> should_flip == false
    let positive_regret_report = BanditShadowReport {
        sample_count: 20_000,
        cumulative_regret: 0.001,
        p99_latency_delta_us: -1.0,
        reward_improvement_ratio: None,
    };
    assert!(!gate.should_flip(&positive_regret_report));

    // 4. p99 Latenz langsamer als Baseline (> 0.0) -> should_flip == false
    let slower_latency_report = BanditShadowReport {
        sample_count: 15_000,
        cumulative_regret: -2.5,
        p99_latency_delta_us: 0.5,
        reward_improvement_ratio: Some(0.05),
    };
    assert!(!gate.should_flip(&slower_latency_report));

    // 5. Überlegener FC-TS Kandidat (viele Samples, negativer Regret, schnellere Latenz) -> should_flip == true
    let superior_report = BanditShadowReport {
        sample_count: 50_000,
        cumulative_regret: -100.0,
        p99_latency_delta_us: -15.2,
        reward_improvement_ratio: Some(0.25),
    };
    assert!(gate.should_flip(&superior_report));
}
