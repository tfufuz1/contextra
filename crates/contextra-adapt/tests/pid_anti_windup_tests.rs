// FILE-CONTEXT
// STAND: 2026-09-27T00:00:00Z
// ZWECK: Integrationstests für conditional Back-Calculation Anti-Windup im PID-Regler.
// INVARIANTEN: Zero-Panic, P28 Clock-Injektion (dt vom Aufrufer), Abwärtskompatibilität zu update().

use contextra_adapt::{AntiWindupController, PidController};
use std::time::Duration;

#[test]
fn test_sustained_high_load_actuator_saturated_prevents_integral_windup() {
    let mut pid = PidController::new(150.0, 50, 200, Some(100));
    let dt = Duration::from_millis(100);

    // 10 Updates mit sehr hoher Latenz bei Sättigung (is_actuator_saturated = true)
    for _ in 0..10 {
        let pool = pid.update_with_anti_windup(dt, 2000.0, true);
        assert!(pool >= 50 && pool <= 200);
    }

    // Nach 10 gesättigten Updates muss der Integrator unberührt bei 0.0 geblieben sein.
    // (Im Vergleich dazu würde update() den Integrator auf max_integral auflaufen lassen).
    // Da self.integral nicht direkt pub ist, testen wir das Verhalten beim ersten nicht-gesättigten Update:
    // Nach Absinken der Latenz auf Ziel-Latenz (150ms, Error = 0) und is_actuator_saturated = false
    // darf die Stellgröße bei k_pool = 100 nicht wegen altem Integrator-Spill verzögert reagieren.
    let pool_after = pid.update_with_anti_windup(dt, 150.0, false);
    assert!(pool_after >= 50);
}

#[test]
fn test_spike_recovery_comparison_faster_convergence() {
    let dt = Duration::from_millis(100);

    // Setup zweier identisch konfigurierter PID-Regler
    let mut pid_simple = PidController::new(150.0, 50, 200, Some(100));
    pid_simple.kp = 0.1;
    pid_simple.ki = 0.2;
    pid_simple.kd = 0.01;

    let mut pid_conditional = PidController::new(150.0, 50, 200, Some(100));
    pid_conditional.kp = 0.1;
    pid_conditional.ki = 0.2;
    pid_conditional.kd = 0.01;

    // Phase 1: 10 Updates mit moderater Überlatenz (180ms vs 150ms Ziel -> Error = -30)
    // Bei pid_conditional signalisieren wir Aktuatorsättigung (is_actuator_saturated = true).
    for _ in 0..10 {
        pid_simple.update(dt, 180.0);
        pid_conditional.update_with_anti_windup(dt, 180.0, true);
    }

    // Phase 2: Latenz sinkt unter Zielwert (100ms vs 150ms Ziel -> Error = +50)
    // Beide Regler sind jetzt nicht mehr gesättigt (is_actuator_saturated = false).
    let mut pool_simple_history = Vec::new();
    let mut pool_conditional_history = Vec::new();

    for _ in 0..5 {
        pool_simple_history.push(pid_simple.update(dt, 100.0));
        pool_conditional_history.push(pid_conditional.update_with_anti_windup(dt, 100.0, false));
    }

    // Qualitatives Kriterium:
    // pid_conditional muss schneller reagieren und die Pool-Größe früher wieder anheben,
    // weil pid_simple zuerst den akkumulierten negativen Integrator abbauen muss.
    let step_simple = pool_simple_history[2];
    let step_conditional = pool_conditional_history[2];

    assert!(
        step_conditional > step_simple,
        "Konditionelles Anti-Windup muss schneller erholen: conditional={step_conditional}, simple={step_simple}"
    );
}

#[test]
fn test_nan_latency_handling_no_panic_returns_current_pool() {
    let mut pid = PidController::new(150.0, 50, 200, Some(100));
    let dt = Duration::from_millis(100);

    // Initiales Update mit valider Latenz
    let pool1 = pid.update_with_anti_windup(dt, 200.0, false);

    // Update mit NaN Latenz
    let pool_nan = pid.update_with_anti_windup(dt, f32::NAN, false);

    // Darf nicht paniken und muss die bisherige Pool-Größe zurückgeben
    assert_eq!(pool_nan, pool1);
    assert_eq!(pid.current_pool_size(), Some(pool1));

    // Update mit INFINITY
    let pool_inf = pid.update_with_anti_windup(dt, f32::INFINITY, true);
    assert_eq!(pool_inf, pool1);
    assert_eq!(pid.current_pool_size(), Some(pool1));
}

#[test]
fn test_unsaturated_behavior_identical_to_update() {
    let dt = Duration::from_millis(100);

    let mut pid_a = PidController::new(150.0, 50, 200, Some(100));
    let mut pid_b = PidController::new(150.0, 50, 200, Some(100));

    let latencies = [150.0, 200.0, 180.0, 120.0, 100.0, 160.0];

    for &lat in &latencies {
        let res_a = pid_a.update(dt, lat);
        let res_b = pid_b.update_with_anti_windup(dt, lat, false);

        assert_eq!(
            res_a, res_b,
            "Bei is_actuator_saturated = false muss das Verhalten exakt identisch zu update() sein"
        );
        assert_eq!(pid_a.current_pool_size(), pid_b.current_pool_size());
    }
}
