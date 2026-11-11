#![allow(clippy::unwrap_used, clippy::cast_sign_loss)]
//! Golden-Test und Generator für deterministische Regler- und Bandit-Folgen (WP-1).
//!
//! Strikter Bit-Vergleich aller Berechnungsfolgen mit festen Seeds und Schrittfolgen.

use std::fs;
use std::path::Path;
use std::time::Duration;

use contextra_adapt::decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
use contextra_adapt::drift::{CatoniDriftDetector, DriftDetector, DriftSignal};
use contextra_adapt::lyapunov::LyapunovDriftWatcher;
use contextra_adapt::pid::PidController;
use contextra_adapt::pid_latency_controller::PidLatencyController;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
struct GoldenCorpus {
    pid_outputs: Vec<usize>,
    pid_latency_outputs: Vec<u64>, // f64 as to_bits
    decay_scores: Vec<u32>, // f32 as to_bits
    catoni_signals: Vec<u8>, // signal as discriminant
    lyapunov_results: Vec<String>,
    #[cfg(feature = "bandit-routing")]
    bandit_scores: Vec<u32>,
    #[cfg(feature = "flow-corrected-thompson")]
    fcts_mus: Vec<u32>,
    #[cfg(feature = "rie-greedy-personalization")]
    rie_predictions: Vec<u32>,
}

fn compute_golden_corpus() -> GoldenCorpus {
    // 1. PID-Controller (100 Schritte mit fester Latenz-Kurve)
    let mut pid = PidController::new(100.0, 50, 500, Some(100));
    let mut pid_outputs = Vec::with_capacity(100);
    for i in 0..100 {
        let latency = 80.0 + (i as f32 * 1.5) % 60.0;
        let pool = pid.update(Duration::from_millis(50), latency);
        pid_outputs.push(pool);
    }

    // 2. PidLatencyController
    let mut lat_ctrl = PidLatencyController::with_params(0.5, 0.1, 0.05, 100.0);
    let mut pid_latency_outputs = Vec::with_capacity(100);
    for i in 0..100 {
        let latency = 70.0 + ((i * 7) % 80) as f64;
        let adj = lat_ctrl.compute_adjustment(latency);
        pid_latency_outputs.push(adj.to_bits());
    }

    // 3. AdaptiveDecayController
    let decay_ctrl = AdaptiveDecayController::new(DecayControllerConfig {
        kappa: 2.0,
        base_half_life_tx: 10_000,
        eviction_threshold: 0.01,
    });
    let mut decay_scores = Vec::with_capacity(50);
    for i in 0..50 {
        let inputs = DecaySignalInputs {
            tombstone_ratio: (i as f32 * 0.02).min(1.0),
            query_load_inverse: 0.3 + (i as f32 * 0.01),
            w_tombstone: 0.6,
            w_query: 0.4,
        };
        let score = decay_ctrl.effective_score(1.0, (i * 500) as u64, &inputs);
        decay_scores.push(score.to_bits());
    }

    // 4. CatoniDriftDetector
    let mut catoni = CatoniDriftDetector::new(1.0, 0.5, 0.1);
    let mut catoni_signals = Vec::with_capacity(60);
    for i in 0..60 {
        let val = if i % 10 == 0 { 10.0 } else { 1.0 + (i as f32 * 0.05) };
        let sig = catoni.observe(val);
        let tag = match sig {
            DriftSignal::Stable => 0,
            DriftSignal::Detected => 1,
        };
        catoni_signals.push(tag);
    }

    // 5. LyapunovDriftWatcher
    let mut lyap = LyapunovDriftWatcher::new(10);
    let mut lyapunov_results = Vec::with_capacity(50);
    for i in 0..50 {
        let val = (i as f32 * 0.1) % 1.0;
        let res = lyap.observe_score(val);
        lyapunov_results.push(format!("{res:?}"));
    }

    // 6. Bandits (falls Feature aktiv)
    #[cfg(feature = "bandit-routing")]
    let bandit_scores = {
        use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};
        let mut bandit = BanditProfileState::cold_start(8, 0.5);
        bandit.implementation = BanditImplementation::ShermanMorrison;
        let mut scores = Vec::with_capacity(30);
        let mut rng = contextra_adapt::SeededRng::new(0x12345678);
        for _ in 0..30 {
            use contextra_adapt::Rng;
            let ctx: Vec<f32> = (0..8).map(|_| rng.next_f32()).collect();
            let score = bandit.score(&ctx, 0.05, false).unwrap();
            bandit.update(&ctx, 0.8, 0.05, false).unwrap();
            scores.push(score.to_bits());
        }
        scores
    };

    #[cfg(feature = "flow-corrected-thompson")]
    let fcts_mus = {
        use contextra_adapt::flow_thompson::{FcTsConfig, FlowCorrectedThompsonBandit};
        let config = FcTsConfig {
            dim: 3,
            lambda: 0.1,
            noise_var: 0.01,
            window_capacity: 50,
            ..Default::default()
        };
        let mut fcts = FlowCorrectedThompsonBandit::new(config).unwrap();
        let mut mus = Vec::with_capacity(30);
        for i in 0..30 {
            let ctx = [0.1 * (i as f32), 0.2, 0.3];
            fcts.update_with_flow(&ctx, 1.0, i as u64, 1.0).unwrap();
            for &m in fcts.mu() {
                mus.push(m.to_bits());
            }
        }
        mus
    };

    #[cfg(feature = "rie-greedy-personalization")]
    let rie_predictions = {
        use contextra_adapt::rie_greedy::RieGreedyProfile;
        let mut rie = RieGreedyProfile::new(3, 1.0, 0.9);
        let mut preds = Vec::with_capacity(20);
        for i in 0..20 {
            let ctx = [0.5, 0.1 * (i as f32), 0.2];
            let pred = rie.predict(&ctx).unwrap();
            rie.update(&ctx, 0.9).unwrap();
            preds.push(pred.to_bits());
        }
        preds
    };

    GoldenCorpus {
        pid_outputs,
        pid_latency_outputs,
        decay_scores,
        catoni_signals,
        lyapunov_results,
        #[cfg(feature = "bandit-routing")]
        bandit_scores,
        #[cfg(feature = "flow-corrected-thompson")]
        fcts_mus,
        #[cfg(feature = "rie-greedy-personalization")]
        rie_predictions,
    }
}

#[test]
fn test_golden_conformance() {
    let golden_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/golden_corpus.json");
    let current = compute_golden_corpus();

    if !golden_path.exists() {
        let json = serde_json::to_string_pretty(&current).unwrap();
        fs::write(&golden_path, json).unwrap();
    } else {
        let expected_json = fs::read_to_string(&golden_path).unwrap();
        let expected: GoldenCorpus = serde_json::from_str(&expected_json).unwrap();
        assert_eq!(current.pid_outputs, expected.pid_outputs, "PID outputs mismatch");
        assert_eq!(current.pid_latency_outputs, expected.pid_latency_outputs, "PID Latency outputs mismatch");
        assert_eq!(current.decay_scores, expected.decay_scores, "Decay scores mismatch");
        assert_eq!(current.catoni_signals, expected.catoni_signals, "Catoni signals mismatch");
        assert_eq!(current.lyapunov_results, expected.lyapunov_results, "Lyapunov results mismatch");

        #[cfg(feature = "bandit-routing")]
        assert_eq!(current.bandit_scores, expected.bandit_scores, "Bandit scores mismatch");

        #[cfg(feature = "flow-corrected-thompson")]
        assert_eq!(current.fcts_mus, expected.fcts_mus, "FC-TS mus mismatch");

        #[cfg(feature = "rie-greedy-personalization")]
        assert_eq!(current.rie_predictions, expected.rie_predictions, "RIE predictions mismatch");
    }
}

#[test]
fn test_experimental_feature_flags() {
    #[cfg(feature = "exp-bandit-routing")]
    let _ = "exp-bandit-routing active";
    #[cfg(feature = "exp-flow-thompson")]
    let _ = "exp-flow-thompson active";
    #[cfg(feature = "exp-rie-greedy")]
    let _ = "exp-rie-greedy active";
}

#[test]
fn test_default_feature_flag() {
    #[cfg(feature = "default")]
    let _ = "default active";
}
