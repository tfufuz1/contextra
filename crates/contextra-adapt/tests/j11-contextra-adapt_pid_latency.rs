use contextra_adapt::pid_latency_controller::{LatencyBudgetGuard, PidLatencyController};

#[test]
fn test_pid_latency_controller_observe_and_adjust_and_scale_pool_size() {
    let guard = LatencyBudgetGuard::new(100.0);
    let mut pid = PidLatencyController::new(100.0);

    // Test observe_and_adjust
    let factor = pid.observe_and_adjust(&guard);
    assert!(factor >= 0.3 && factor <= 1.0);

    // Test scale_pool_size under target latency (50ms)
    let base_k = 200;
    let scaled_k_low = pid.scale_pool_size(50.0, base_k);
    assert_eq!(scaled_k_low, 200);

    // Test scale_pool_size under high latency (500ms)
    let scaled_k_high = pid.scale_pool_size(500.0, base_k);
    assert!(scaled_k_high < base_k);
    assert!(scaled_k_high >= (base_k as f64 * 0.3) as usize);
}
