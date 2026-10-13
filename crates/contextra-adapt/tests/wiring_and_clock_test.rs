use std::sync::Arc;
use std::time::Duration;

use contextra_adapt::pid_latency_controller::LatencyBudgetGuard;
use contextra_adapt::wiring::{AdaptWiring, AdaptWiringDeps};
use contextra_ports::ManualClock;

#[test]
fn test_latency_budget_guard_with_manual_clock() {
    let clock = Arc::new(ManualClock::new(1_000_000_000));
    let guard = LatencyBudgetGuard::new_with_clock(100.0, clock.clone());

    assert_eq!(guard.elapsed_ms(), 0.0);
    assert!(!guard.is_exceeded());

    // Advance manual clock by 50ms
    clock.advance(Duration::from_millis(50));
    assert_eq!(guard.elapsed_ms(), 50.0);
    assert!(!guard.is_exceeded());

    // Advance manual clock by 60ms (total 110ms)
    clock.advance(Duration::from_millis(60));
    assert_eq!(guard.elapsed_ms(), 110.0);
    assert!(guard.is_exceeded());
}

#[test]
fn test_adapt_wiring_build_and_control_cycle() {
    let clock = Arc::new(ManualClock::new(1_000_000_000));
    let deps = AdaptWiringDeps {
        clock: clock.clone(),
    };

    let wiring = AdaptWiring::build(deps).expect("AdaptWiring build should succeed");

    // Perform a control cycle with pid_latency_controller
    let mut pid_latency = wiring.pid_latency_controller.lock().unwrap();
    let guard = LatencyBudgetGuard::new_with_clock(100.0, clock.clone());
    let adjustment = pid_latency.observe_and_adjust(&guard);
    assert!(adjustment >= 0.3 && adjustment <= 1.0);

    // Verify pool scaling
    let base_pool = 100;
    let scaled = pid_latency.scale_pool_size(150.0, base_pool);
    assert!(scaled < base_pool);
}
