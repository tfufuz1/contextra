//! Deterministic tests for adaptive controllers, PID regulators, bandits, Lyapunov watchers, and shadow mode.

use contextra_adapt::flow_thompson::{
    FcTsArmSet, FcTsConfig, FcTsRng, FlowCorrectedThompsonBandit, SplitMix64,
};
use contextra_adapt::lyapunov::LyapunovDriftWatcher;
use contextra_adapt::pid_latency_controller::{
    LatencyBudgetGuard, PidLatencyController, MAX_SCALING_FACTOR, MIN_SCALING_FACTOR,
};
use contextra_adapt::shadow_mode::{ShadowDiscrepancy, ShadowSink};
use contextra_adapt::{
    AdaptiveDecayController, AntiWindupController, BanditProfileState, CatoniDriftDetector,
    DecayControllerConfig, DecaySignalInputs, DriftDetector, DriftPolicyBridge, PidController,
};
use contextra_ports::Clock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
struct MockClock {
    nanos: AtomicU64,
}

impl Clock for MockClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }
}

// --- PID CONTROLLER TESTS ---

#[test]
fn test_pid_anti_windup_bounds_integrator_strictly() {
    let mut pid = PidController::new(150.0, 50, 200, Some(100));
    let dt = Duration::from_millis(100);

    // Drive with high latency to saturate min pool size (50)
    for _ in 0..100 {
        pid.update(dt, 1000.0);
    }

    // Capture pool and integral when saturated
    let pool_saturated = pid.update(dt, 1000.0);
    assert_eq!(pool_saturated, 50);

    // Using AntiWindupController trait explicitly with saturated flag
    let pool_aw = pid.update_with_anti_windup(dt, 1000.0, true);
    assert_eq!(pool_aw, 50);
}

#[test]
fn test_pid_step_response_without_overshoot() {
    let mut pid = PidController::new(100.0, 50, 200, Some(100));
    let dt = Duration::from_millis(100);

    // Apply a step change in latency from target (100ms) to 150ms
    let pool1 = pid.update(dt, 150.0);
    let pool2 = pid.update(dt, 150.0);
    let pool3 = pid.update(dt, 150.0);

    assert!(pool1 <= 100);
    assert!(pool2 <= pool1);
    assert!(pool3 <= pool2);
    assert!(pool3 >= 50);
}

#[test]
fn test_pid_latency_controller_step_response_clamping() {
    let mut controller = PidLatencyController::new(100.0);

    // Step increase in latency to 300ms
    let adj1 = controller.compute_adjustment(300.0);
    let adj2 = controller.compute_adjustment(300.0);

    assert!(adj1 < MAX_SCALING_FACTOR);
    assert!(adj2 <= adj1);
    assert!(adj2 >= MIN_SCALING_FACTOR);
}

#[test]
fn test_pid_resets_properly() {
    let mut pid = PidController::new(150.0, 50, 200, Some(100));
    let dt = Duration::from_millis(100);
    pid.update(dt, 300.0);

    pid.reset();
    assert_eq!(pid.current_pool_size(), None);
}

#[test]
fn test_pid_latency_controller_reset_clears_history() {
    let mut controller = PidLatencyController::new(100.0);
    controller.compute_adjustment(200.0);
    controller.reset();

    let adj_fresh = controller.compute_adjustment(100.0);
    assert_eq!(adj_fresh, MAX_SCALING_FACTOR);
}

// --- LYAPUNOV & DRIFT DETECTOR TESTS ---

#[test]
fn test_lyapunov_drift_monitor_detects_divergence() {
    let mut watcher = LyapunovDriftWatcher::new(10);
    let baseline: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0 * 0.2).collect();
    watcher.set_baseline(&baseline);

    // Inject sharp divergence in non-conformity scores
    let shifted: Vec<f32> = (0..100).map(|i| 0.8 + (i as f32) / 100.0 * 0.2).collect();

    let mut detected = false;
    for _ in 0..15 {
        let res = watcher.update(&shifted);
        if res.is_drift_detected() {
            detected = true;
            break;
        }
    }

    assert!(
        detected,
        "LyapunovDriftWatcher must detect sharp distribution divergence"
    );
}

#[test]
fn test_lyapunov_drift_monitor_status_str() {
    let mut watcher = LyapunovDriftWatcher::new(10);
    assert_eq!(watcher.status_str(), "unbekannt");

    let baseline: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0 * 0.5).collect();
    watcher.set_baseline(&baseline);

    for _ in 0..12 {
        watcher.update(&baseline);
    }
    assert_eq!(watcher.status_str(), "stabil");
}

#[test]
fn test_catoni_drift_detector_reset() {
    let mut detector = CatoniDriftDetector::new(0.5, 1.0, 0.05);
    for _ in 0..20 {
        detector.observe(1.0);
    }
    detector.reset();
    assert_eq!(detector.catoni_mu, 0.5);
    assert_eq!(detector.cusum_sum, 0.0);
    assert!(!detector.is_drifting());
}

#[test]
fn test_drift_policy_bridge_default() {
    let bridge = DriftPolicyBridge::default();
    assert_eq!(bridge.k_drift, 2.0);
    assert_eq!(bridge.alpha_max, 4.0);
    assert_eq!(bridge.gamma, 0.95);
}

// --- BANDIT CONVERGENCE TESTS ---

#[test]
fn test_fcts_bandit_converges_on_synthetic_bernoulli_environment() {
    let config = FcTsConfig {
        dim: 2,
        lambda: 1.0,
        noise_var: 0.1,
        window_capacity: 100,
        explore_scale: 0.1,
        drift_ridge: 10.0,
        max_drift_norm: 5.0,
    };

    let mut arm0 = FlowCorrectedThompsonBandit::new(config.clone()).expect("arm0 ok");
    let mut arm1 = FlowCorrectedThompsonBandit::new(config.clone()).expect("arm1 ok");

    let mut rng = SplitMix64::new(12345);

    // Context x = [1.0, 0.0]
    let ctx = vec![1.0f32, 0.0f32];

    // Synthetic Bernoulli environment:
    // Arm 0 reward mean = 0.8 (optimal)
    // Arm 1 reward mean = 0.2 (suboptimal)
    let steps = 300;
    for t in 0..steps {
        let r0 = if (rng.next_f32()) < 0.8 { 1.0 } else { 0.0 };
        let r1 = if (rng.next_f32()) < 0.2 { 1.0 } else { 0.0 };

        arm0.update_with_flow(&ctx, r0, t as u64, 1.0)
            .expect("update arm0");
        arm1.update_with_flow(&ctx, r1, t as u64, 1.0)
            .expect("update arm1");
    }

    let arm0_mean = arm0.mu()[0];
    let arm1_mean = arm1.mu()[0];

    assert!(
        arm0_mean > arm1_mean,
        "FC-TS arm0 mean ({arm0_mean}) must converge higher than arm1 mean ({arm1_mean})"
    );
    assert!(
        (arm0_mean - 0.8).abs() < 0.2,
        "Arm0 mean should be near 0.8, got {arm0_mean}"
    );
}

#[test]
fn test_linucb_bandit_converges_on_synthetic_environment() {
    let mut state0 = BanditProfileState::cold_start(2, 0.5);
    let mut state1 = BanditProfileState::cold_start(2, 0.5);

    let ctx = vec![1.0f32, 0.0f32];
    let mut rng = SplitMix64::new(54321);

    for _ in 0..200 {
        let r0 = if (rng.next_f32()) < 0.9 { 1.0 } else { 0.0 };
        let r1 = if (rng.next_f32()) < 0.1 { 1.0 } else { 0.0 };

        state0.update(&ctx, r0, 0.0, false).expect("update 0");
        state1.update(&ctx, r1, 0.0, false).expect("update 1");
    }

    let score0 = state0.score(&ctx, 0.0, false).expect("score 0");
    let score1 = state1.score(&ctx, 0.0, false).expect("score 1");

    assert!(
        score0 > score1,
        "LinUCB optimal state score ({score0}) must exceed suboptimal ({score1})"
    );
}

#[test]
fn test_fcts_config_validation_failures() {
    let mut cfg = FcTsConfig::default();
    cfg.lambda = -1.0;
    assert!(cfg.validate().is_err());

    cfg = FcTsConfig::default();
    cfg.noise_var = 0.0;
    assert!(cfg.validate().is_err());

    cfg = FcTsConfig::default();
    cfg.window_capacity = 0;
    assert!(cfg.validate().is_err());

    cfg = FcTsConfig::default();
    cfg.explore_scale = f32::NAN;
    assert!(cfg.validate().is_err());

    cfg = FcTsConfig::default();
    cfg.drift_ridge = -5.0;
    assert!(cfg.validate().is_err());

    cfg = FcTsConfig::default();
    cfg.max_drift_norm = 0.0;
    assert!(cfg.validate().is_err());
}

// --- SHADOW MODE TESTS ---

#[derive(Default)]
struct TestRecordingSink {
    records: Arc<Mutex<Vec<ShadowDiscrepancy<u32>>>>,
}

impl ShadowSink<u32> for TestRecordingSink {
    fn record(&self, discrepancy: ShadowDiscrepancy<u32>) {
        self.records.lock().unwrap().push(discrepancy);
    }
}

#[test]
fn test_shadow_mode_never_alters_productive_decisions() {
    let config = FcTsConfig {
        dim: 2,
        ..Default::default()
    };
    let arm0 = FlowCorrectedThompsonBandit::new(config.clone()).expect("arm0 ok");
    let arm1 = FlowCorrectedThompsonBandit::new(config.clone()).expect("arm1 ok");

    let arm_set_prod = FcTsArmSet {
        arms: vec![arm0.clone(), arm1.clone()],
    };

    let sink = Arc::new(TestRecordingSink::default());
    let arm_set_shadow = FcTsArmSet {
        arms: vec![arm0, arm1],
    }
    .with_shadow_sink(sink.clone());

    let ctx = vec![1.0f32, 0.0f32];

    for seed in 1..=50 {
        let mut rng_prod = SplitMix64::new(seed);
        let mut rng_shadow = SplitMix64::new(seed);

        let choice_prod = arm_set_prod
            .select_arm(&ctx, &mut rng_prod)
            .expect("prod decision");
        let choice_shadow = arm_set_shadow
            .select_arm(&ctx, &mut rng_shadow)
            .expect("shadow decision");

        assert_eq!(
            choice_prod, choice_shadow,
            "Shadow mode must never alter productive decision for seed {seed}"
        );
    }

    assert_eq!(sink.records.lock().unwrap().len(), 50);
}

// --- CLOCK & TIMING TESTS ---

#[test]
fn test_latency_budget_guard_with_mock_clock() {
    let clock = Arc::new(MockClock {
        nanos: AtomicU64::new(1_000_000_000), // 1.0s
    });

    let guard = LatencyBudgetGuard::new_with_clock(100.0, clock.clone());
    assert!(!guard.is_exceeded());
    assert_eq!(guard.elapsed_ms(), 0.0);

    // Advance clock by 50ms (50,000,000 nanos)
    clock
        .nanos
        .store(1_050_000_000, std::sync::atomic::Ordering::Relaxed);
    assert!(!guard.is_exceeded());
    assert_eq!(guard.elapsed_ms(), 50.0);

    // Advance clock by 101ms
    clock
        .nanos
        .store(1_101_000_000, std::sync::atomic::Ordering::Relaxed);
    assert!(guard.is_exceeded());
    assert_eq!(guard.elapsed_ms(), 101.0);
}

#[test]
fn test_latency_budget_guard_default_fallback() {
    let guard = LatencyBudgetGuard::new(f64::NAN);
    assert_eq!(guard.budget_ms(), 100.0);
}

// --- DECAY CONTROLLER ADDITIONAL TESTS ---

#[test]
fn test_decay_controller_config_default() {
    let cfg = DecayControllerConfig::default();
    assert_eq!(cfg.kappa, 2.0);
    assert_eq!(cfg.base_half_life_tx, 10_000);
    assert_eq!(cfg.eviction_threshold, 0.01);
}

#[test]
fn test_decay_signal_inputs_default() {
    let inputs = DecaySignalInputs::default();
    assert_eq!(inputs.tombstone_ratio, 0.0);
    assert_eq!(inputs.query_load_inverse, 0.5);
    assert_eq!(inputs.w_tombstone, 0.6);
    assert_eq!(inputs.w_query, 0.4);
}

#[test]
fn test_adaptive_decay_controller_temperature_extreme_clamp() {
    let controller = AdaptiveDecayController::with_defaults();
    let extreme_inputs = DecaySignalInputs {
        tombstone_ratio: 5.0,
        query_load_inverse: 5.0,
        ..Default::default()
    };
    let temp = controller.system_temperature(&extreme_inputs);
    assert_eq!(temp, 1.0);
}

#[test]
fn test_adaptive_decay_controller_effective_score_zero_elapsed() {
    let controller = AdaptiveDecayController::with_defaults();
    let inputs = DecaySignalInputs::default();
    let score = controller.effective_score(0.85, 0, &inputs);
    assert!((score - 0.85).abs() < 1e-6);
}

#[test]
fn test_sketched_bandit_feature_gated_initialization() {
    #[cfg(feature = "sketched-bandit")]
    {
        let mut state = BanditProfileState::cold_start(4, 0.5);
        state.implementation = contextra_adapt::BanditImplementation::SketchedProjection {
            projected_dim: 2,
        };
        assert!(state.ensure_sketched_state(2).is_ok());
        let x = vec![1.0f32; 4];
        let score = state.score(&x, 0.0, false);
        assert!(score.is_ok());
    }
}

#[test]
fn test_catoni_drift_detector_influence_function_bounds() {
    let mut detector = CatoniDriftDetector::new(0.5, 1.0, 0.05);
    // Observe extreme value
    detector.observe(1000.0);
    // Catoni M-estimator update step psi = 999.5 / (1 + 999.5) < 1.0, so catoni_mu grows boundedly
    assert!(detector.catoni_mu < 0.6);
}

#[test]
fn test_ensemble_drift_watcher_set_baseline_updates_both_detectors() {
    let mut watcher = contextra_adapt::EnsembleDriftWatcher::default();
    let scores = vec![0.2, 0.4, 0.6, 0.8];
    watcher.set_baseline(&scores);
    assert!((watcher.catoni.baseline_reward - 0.5).abs() < 1e-6);
}

#[test]
fn test_pid_latency_controller_with_params() {
    let pid = PidLatencyController::with_params(0.8, 0.1, 0.2, 120.0);
    assert_eq!(pid.kp, 0.8);
    assert_eq!(pid.ki, 0.1);
    assert_eq!(pid.kd, 0.2);
    assert_eq!(pid.target_latency_ms, 120.0);
}

#[test]
fn test_pid_latency_controller_negative_params_fallback() {
    let pid = PidLatencyController::with_params(-1.0, -1.0, -1.0, -50.0);
    assert_eq!(pid.kp, 0.5);
    assert_eq!(pid.ki, 0.05);
    assert_eq!(pid.kd, 0.1);
    assert_eq!(pid.target_latency_ms, 100.0);
}
