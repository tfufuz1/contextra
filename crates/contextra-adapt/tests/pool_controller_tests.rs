// FILE-CONTEXT
// STAND: 2026-10-17T00:00:00Z
// ZWECK: Unit, Integration, Plant-Model Step Response, Anti-Windup, and Proptest for PoolController.

use contextra_adapt::{
    ManualControlClock, PoolController, PoolControllerConfig,
};
use proptest::prelude::*;

#[test]
fn test_config_validation() {
    // Valid config
    let valid = PoolControllerConfig::default();
    assert!(valid.validate().is_ok());

    // Invalid kp
    let mut invalid = valid;
    invalid.kp = -0.1;
    assert!(invalid.validate().is_err());

    // Invalid min_pool == 0
    let mut invalid = valid;
    invalid.min_pool = 0;
    assert!(invalid.validate().is_err());

    // Invalid max_pool < min_pool
    let mut invalid = valid;
    invalid.max_pool = 40;
    assert!(invalid.validate().is_err());

    // Invalid pool_nominal out of bounds
    let mut invalid = valid;
    invalid.pool_nominal = 300;
    assert!(invalid.validate().is_err());

    // Invalid target latency
    let mut invalid = valid;
    invalid.target_latency_ms = -10.0;
    assert!(invalid.validate().is_err());

    // Invalid filter time constant
    let mut invalid = valid;
    invalid.filter_time_constant_s = 0.0;
    assert!(invalid.validate().is_err());
}

#[test]
fn test_step_response_against_plant_model() {
    // Plant model: latency = a * pool + noise
    // Target latency = 100.0 ms. Parameter a = 0.8 ms/candidate
    let config = PoolControllerConfig {
        kp: 0.5,
        ki: 0.2,
        kd: 0.01,
        min_pool: 20,
        max_pool: 300,
        pool_nominal: 100,
        target_latency_ms: 100.0,
        filter_time_constant_s: 0.05,
    };

    let mut controller = PoolController::new(config).expect("valid config");
    let mut clock = ManualControlClock::new(0);

    let a = 0.8f32; // 0.8 ms per pool item
    let noise = 1.0f32; // small measurement noise
    let mut pool = controller.current_pool();

    let mut pool_history = Vec::new();

    // Simulate 150 control steps (dt = 0.05s = 50ms per step)
    for i in 0..150 {
        clock.advance_nanos(50_000_000);
        // Plant output
        let measured_latency = a * (pool as f32) + if i % 2 == 0 { noise } else { -noise };
        pool = controller.update_with_clock(&clock, measured_latency);
        pool_history.push(pool);
    }

    // Steady state target pool size = 100.0 / 0.8 = 125
    let steady_state_target = (100.0 / a).round() as usize;

    // Check settling time: after 80 steps (4.0s), pool should be within 10% of steady state target (112..138)
    let settled_pool = pool_history[80];
    let diff = (settled_pool as isize - steady_state_target as isize).abs();
    assert!(
        diff <= 15,
        "PoolController must settle close to target: target={steady_state_target}, got={settled_pool}"
    );

    // Overshoot bound: peak pool must not exceed 1.35 * steady_state_target
    let max_peak = *pool_history.iter().max().unwrap_or(&0);
    assert!(
        max_peak <= 170,
        "Step response overshoot must be bounded: max_peak={max_peak}, target={steady_state_target}"
    );
}

#[test]
fn test_no_windup_after_60s_saturation() {
    let config = PoolControllerConfig {
        kp: 0.5,
        ki: 0.2,
        kd: 0.05,
        min_pool: 50,
        max_pool: 200,
        pool_nominal: 100,
        target_latency_ms: 100.0,
        filter_time_constant_s: 0.05,
    };

    let mut controller = PoolController::new(config).expect("valid config");
    let mut clock = ManualControlClock::new(0);

    // Saturate actuator continuously for 60 seconds (600 steps of dt = 0.1s)
    // Latency = 1000ms (way above target 100ms) -> error = -900ms -> pool hits min_pool=50
    for _ in 0..600 {
        clock.advance_nanos(100_000_000);
        let pool = controller.update_with_clock(&clock, 1000.0);
        assert_eq!(pool, 50, "Actuator must be clamped to min_pool=50 during saturation");
    }

    // Immediately drop latency to 90ms (error = +10ms, slightly under target)
    // With back-calculation anti-windup, the controller should begin recovering out of saturation immediately
    let mut recovered = false;
    for _ in 0..5 {
        clock.advance_nanos(100_000_000);
        let pool = controller.update_with_clock(&clock, 90.0);
        if pool > 50 {
            recovered = true;
            break;
        }
    }

    assert!(
        recovered,
        "PoolController must recover out of saturation immediately after 60s without integral windup delay"
    );
}

proptest! {
    #[test]
    fn prop_output_always_in_min_max_bounds(
        kp in 0.0f32..10.0f32,
        ki in 0.0f32..5.0f32,
        kd in 0.0f32..5.0f32,
        min_pool in 10usize..100usize,
        max_pool_delta in 10usize..200usize,
        measured_latency in -500.0f32..10000.0f32,
        dt_s in 0.0001f32..5.0f32,
    ) {
        let max_pool = min_pool + max_pool_delta;
        let pool_nominal = (min_pool + max_pool) / 2;

        let config = PoolControllerConfig {
            kp,
            ki,
            kd,
            min_pool,
            max_pool,
            pool_nominal,
            target_latency_ms: 100.0,
            filter_time_constant_s: 0.05,
        };

        if let Ok(mut controller) = PoolController::new(config) {
            for _ in 0..10 {
                let pool = controller.update_step(dt_s, measured_latency);
                prop_assert!(pool >= min_pool, "pool {} < min_pool {}", pool, min_pool);
                prop_assert!(pool <= max_pool, "pool {} > max_pool {}", pool, max_pool);
            }
        }
    }
}
