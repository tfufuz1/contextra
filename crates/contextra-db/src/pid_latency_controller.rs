// FILE-CONTEXT
// ZWECK: PID Latency Controller & Latency Budget Guard für Multi-Step Retrieval.
// INVARIANTEN: Skalierungsfaktor immer in [0.3, 1.0] geklemmt; Integrator-Anti-Windup verhindert Oszillation.
// RISIKO-FOKUS: Numerische Stabilität (kein Integrator-Windup, Non-Finite Schutz, Clamping).

use contextra_ports::{Clock, SystemClock};
use std::sync::Arc;

pub const DEFAULT_TARGET_LATENCY_MS: f64 = 100.0;
pub const MIN_SCALING_FACTOR: f64 = 0.3;
pub const MAX_SCALING_FACTOR: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct PidLatencyController {
    kp: f64,
    ki: f64,
    kd: f64,
    integral: f64,
    last_error: f64,
    target_latency_ms: f64,
    scaling_factor: f64,
}

impl Default for PidLatencyController {
    fn default() -> Self {
        Self::new(DEFAULT_TARGET_LATENCY_MS)
    }
}

impl PidLatencyController {
    pub fn new(target_latency_ms: f64) -> Self {
        Self {
            kp: 0.005,
            ki: 0.001,
            kd: 0.0005,
            integral: 0.0,
            last_error: 0.0,
            target_latency_ms: target_latency_ms.max(1.0),
            scaling_factor: 1.0,
        }
    }

    pub fn target_latency_ms(&self) -> f64 {
        self.target_latency_ms
    }

    pub fn current_scaling_factor(&self) -> f64 {
        self.scaling_factor
    }

    pub fn update(&mut self, elapsed_ms: f64, dt_seconds: f64) -> f64 {
        let dt = if dt_seconds.is_finite() && dt_seconds > 0.0001 {
            dt_seconds.min(10.0)
        } else {
            0.1
        };

        let elapsed = if elapsed_ms.is_finite() && elapsed_ms >= 0.0 {
            elapsed_ms
        } else {
            return self.scaling_factor;
        };

        let error = self.target_latency_ms - elapsed;
        let p_term = self.kp * error;

        self.integral = (self.integral + error * dt).clamp(-500.0, 500.0);
        let i_term = self.ki * self.integral;

        let d_term = self.kd * (error - self.last_error) / dt;
        self.last_error = error;

        let raw_control = p_term + i_term + d_term;
        let delta_factor = raw_control.clamp(-0.2, 0.2);
        self.scaling_factor =
            (self.scaling_factor + delta_factor).clamp(MIN_SCALING_FACTOR, MAX_SCALING_FACTOR);
        self.scaling_factor
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.last_error = 0.0;
        self.scaling_factor = 1.0;
    }
}

#[derive(Debug, Clone)]
pub struct LatencyBudgetGuard {
    start_time_nanos: u64,
    budget_ms: f64,
    clock: Arc<dyn Clock>,
}

impl LatencyBudgetGuard {
    pub fn new(budget_ms: f64) -> Self {
        Self::with_clock(budget_ms, Arc::new(SystemClock::new()))
    }

    pub fn with_clock(budget_ms: f64, clock: Arc<dyn Clock>) -> Self {
        let start = clock.monotonic_nanos();
        Self {
            start_time_nanos: start,
            budget_ms: budget_ms.max(0.1),
            clock,
        }
    }

    pub fn elapsed_ms(&self) -> f64 {
        let now = self.clock.monotonic_nanos();
        let elapsed_nanos = now.saturating_sub(self.start_time_nanos);
        elapsed_nanos as f64 / 1_000_000.0
    }

    pub fn is_exceeded(&self) -> bool {
        self.elapsed_ms() >= self.budget_ms
    }

    pub fn remaining_ms(&self) -> f64 {
        (self.budget_ms - self.elapsed_ms()).max(0.0)
    }

    pub fn budget_ms(&self) -> f64 {
        self.budget_ms
    }
}
