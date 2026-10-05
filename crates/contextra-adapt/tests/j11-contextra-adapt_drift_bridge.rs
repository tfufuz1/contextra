use contextra_adapt::bandit::BanditProfileState;
use contextra_adapt::drift::DriftPolicyBridge;
use contextra_adapt::lyapunov::LyapunovResult;

#[test]
fn test_bandit_observe_and_adapt_drift_wires_observe_and_react() {
    let mut bridge = DriftPolicyBridge::new(10, 2.0, 4.0, 0.95);
    let baseline: Vec<f32> = (0..100).map(|i| (i as f32 / 100.0) * 0.2).collect();
    bridge.watcher.set_baseline(&baseline);

    let mut bandit = BanditProfileState::cold_start(2, 0.5);
    let alpha_before = bandit.alpha;

    let mut last_res = LyapunovResult::InsufficientData;
    for i in 0..25 {
        let shift = (i as f32 / 25.0) * 0.8;
        let current: Vec<f32> = (0..50)
            .map(|j| ((j as f32 / 50.0) * 0.2 + shift).clamp(0.0, 1.0))
            .collect();
        last_res = bandit.observe_and_adapt_drift(&mut bridge, &current);
    }

    assert!(matches!(last_res, LyapunovResult::DriftDetected { .. }));
    assert!(bandit.alpha > alpha_before);
    assert_eq!(bandit.drift_steps_remaining, bandit.drift_decay_window);
}
