use std::sync::Arc;
use std::time::Duration;

use contextra_adapt::drift::{DriftDetector, DriftSignal};
use contextra_adapt::pid_latency_controller::LatencyBudgetGuard;
use contextra_adapt::shadow_mode::{ShadowDiscrepancy, ShadowSink};
use contextra_adapt::wiring::{AdaptWiring, AdaptWiringDeps};
use contextra_ports::ManualClock;

#[test]
fn test_adapt_wiring_comprehensive_production_path() {
    let clock = Arc::new(ManualClock::new(1_000_000_000));
    let deps = AdaptWiringDeps {
        clock: clock.clone(),
    };

    let wiring = AdaptWiring::build(deps).expect("AdaptWiring build must succeed");

    // 1. PID Controller & AntiWindupController Trait Object
    {
        let mut pid = wiring.pid_controller.lock().unwrap();
        let pool_size = pid.update(Duration::from_millis(100), 150.0);
        assert_eq!(pool_size, 50);
    }
    {
        let mut anti_windup = wiring.anti_windup_controller.lock().unwrap();
        let pool_size =
            anti_windup.update_with_anti_windup(Duration::from_millis(100), 200.0, true);
        assert!(pool_size >= 50);
    }

    // 2. PID Latency Controller & Latency Budget Guard
    {
        let mut pid_latency = wiring.pid_latency_controller.lock().unwrap();
        let guard = LatencyBudgetGuard::new_with_clock(100.0, clock.clone());

        // Baseline observation under target budget
        let adj_normal = pid_latency.observe_and_adjust(&guard);
        assert_eq!(adj_normal, 1.0);

        // Advance clock to simulate over-budget execution
        clock.advance(Duration::from_millis(150));
        assert!(guard.is_exceeded());

        let adj_over = pid_latency.observe_and_adjust(&guard);
        assert!(adj_over < 1.0);

        let scaled = pid_latency.scale_pool_size(guard.elapsed_ms(), 200);
        assert!(scaled < 200);
        assert!(scaled >= 60); // 200 * 0.3 = 60 floor
    }

    // 3. Decay Controller
    {
        let inputs = contextra_adapt::decay_controller::DecaySignalInputs::default();
        let base_score = 0.8f32;
        let elapsed_tx = 100u64;
        let should_evict = wiring
            .decay_controller
            .should_evict(base_score, elapsed_tx, &inputs);
        assert!(!should_evict);
    }

    // 4. Homeostat PID Controller
    {
        let mut homeostat = wiring.homeostat.lock().unwrap();
        let pool = homeostat.update(Duration::from_millis(100), 100.0);
        assert!(pool >= 50);
    }

    // 5. Drift Detector & Catoni Drift Detector
    {
        let mut detector = wiring.drift_detector.lock().unwrap();
        let sig = detector.observe(0.5);
        assert_eq!(sig, DriftSignal::Stable);
        assert!(!detector.is_drifting());
    }
    {
        let mut catoni = wiring.catoni_drift_detector.lock().unwrap();
        let sig = catoni.observe(0.5);
        assert_eq!(sig, DriftSignal::Stable);
    }

    // 6. Ensemble Drift Watcher
    {
        let mut ensemble = wiring.ensemble_drift_watcher.lock().unwrap();
        let drift = ensemble.observe_and_decide(0.5);
        assert!(!drift);
    }

    // 7. Bandit Policy & Bandit Profile State
    #[cfg(feature = "bandit-routing")]
    {
        let bandit_state = wiring.bandit_profile_state.lock().unwrap();
        assert!(bandit_state.alpha > 0.0);

        let mut bandit_policy = wiring.bandit_policy.lock().unwrap();
        bandit_policy.apply_drift_penalty(2.0, 4.0, 0.95);
    }

    // 8. Flip Gate
    #[cfg(feature = "flow-corrected-thompson")]
    {
        use contextra_adapt::flow_thompson::BanditShadowReport;
        let report = BanditShadowReport {
            sample_count: 5_000,
            cumulative_regret: 0.0,
            p99_latency_delta_us: -10.0,
            reward_improvement_ratio: None,
        };
        let should_flip = wiring.flip_gate.should_flip(&report);
        assert!(!should_flip);
    }

    // 9. Shadow Sink
    {
        wiring.shadow_sink.record(ShadowDiscrepancy {
            baseline: "A",
            candidate: "B",
            context_id: 42,
        });
    }
}
