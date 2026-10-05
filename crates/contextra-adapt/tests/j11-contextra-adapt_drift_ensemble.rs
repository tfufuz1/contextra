use contextra_adapt::bandit::BanditProfileState;
use contextra_adapt::drift::EnsembleDriftWatcher;

#[test]
fn test_ensemble_drift_watcher_observe_and_decide_and_react_policy() {
    let mut ensemble = EnsembleDriftWatcher::default();
    let baseline: Vec<f32> = (0..100).map(|i| (i as f32 / 100.0) * 0.2).collect();
    ensemble.set_baseline(&baseline);

    let mut bandit = BanditProfileState::cold_start(2, 0.5);

    // Stable inputs - observe_and_decide should return false
    assert!(!ensemble.observe_and_decide(0.1));

    // Sustained high shift - both detectors will detect drift
    let mut detected = false;
    for _ in 0..50 {
        if ensemble.observe_and_react_policy(0.9, &mut bandit, 2.0, 4.0, 0.95) {
            detected = true;
            break;
        }
    }

    assert!(detected, "Ensemble drift should trigger policy penalty");
    assert_eq!(bandit.drift_steps_remaining, bandit.drift_decay_window);
}
