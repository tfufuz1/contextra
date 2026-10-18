// FILE-CONTEXT
// ZWECK: PID Latency Controller & Latency Budget Guard für Multi-Step Retrieval.
// INVARIANTEN: Skalierungsfaktor immer in [0.3, 1.0] geklemmt; Integrator-Anti-Windup verhindert Oszillation.
// RISIKO-FOKUS: Numerische Stabilität (kein Integrator-Windup, Non-Finite Schutz, Clamping).
// STAND: TS:2026-10-17T00:00:00Z

//! PID Latency Controller & Latency Budget Guard.

use contextra_ports::{Clock, SystemClock};
use std::sync::Arc;

/// Konservativer Default für das P95-Latenzziel in Millisekunden (100ms).
pub const DEFAULT_TARGET_LATENCY_MS: f64 = 100.0;

/// Unterer Clamp für den K-Pool-Skalierungsfaktor (30% des konfigurierten K-Pools).
pub const MIN_SCALING_FACTOR: f64 = 0.3;

/// Oberer Clamp für den K-Pool-Skalierungsfaktor (100% des konfigurierten K-Pools).
pub const MAX_SCALING_FACTOR: f64 = 1.0;

/// Max-Grenze für den I-Anteil (Anti-Windup Clamping).
pub const MAX_INTEGRAL: f64 = 10.0;

/// Classical PID controller for dynamically scaling multi-step retrieval parameters (`k_pool`, `max_hops`).
///
/// DEPRECATED: Use [`crate::PoolController`] instead.
#[deprecated(since = "0.1.0", note = "use PoolController; removal 2027-01-31")]
#[derive(Debug, Clone)]
pub struct PidLatencyController {
    /// Proportional gain coefficient.
    pub kp: f64,
    /// Integral gain coefficient.
    pub ki: f64,
    /// Derivative gain coefficient.
    pub kd: f64,
    /// Target latency budget in milliseconds.
    pub target_latency_ms: f64,
    /// Accumulated error for integral term.
    integral: f64,
    /// Previous error for derivative term.
    previous_error: Option<f64>,
}

#[allow(deprecated)]
impl Default for PidLatencyController {
    fn default() -> Self {
        Self::new(DEFAULT_TARGET_LATENCY_MS)
    }
}

#[allow(deprecated)]
impl PidLatencyController {
    /// Creates a new `PidLatencyController` with specified target latency in milliseconds.
    pub fn new(target_latency_ms: f64) -> Self {
        let target_latency_ms = if target_latency_ms.is_finite() && target_latency_ms > 0.0 {
            target_latency_ms
        } else {
            DEFAULT_TARGET_LATENCY_MS
        };

        Self {
            kp: 0.5,
            ki: 0.05,
            kd: 0.1,
            target_latency_ms,
            integral: 0.0,
            previous_error: None,
        }
    }

    /// Custom constructor specifying gains and target latency.
    pub fn with_params(kp: f64, ki: f64, kd: f64, target_latency_ms: f64) -> Self {
        let target_latency_ms = if target_latency_ms.is_finite() && target_latency_ms > 0.0 {
            target_latency_ms
        } else {
            DEFAULT_TARGET_LATENCY_MS
        };

        Self {
            kp: if kp.is_finite() && kp >= 0.0 { kp } else { 0.5 },
            ki: if ki.is_finite() && ki >= 0.0 {
                ki
            } else {
                0.05
            },
            kd: if kd.is_finite() && kd >= 0.0 { kd } else { 0.1 },
            target_latency_ms,
            integral: 0.0,
            previous_error: None,
        }
    }

    /// Calculates scaling adjustment factor in range `[0.3, 1.0]` based on observed latency.
    pub fn compute_adjustment(&mut self, observed_latency_ms: f64) -> f64 {
        if !observed_latency_ms.is_finite() || observed_latency_ms < 0.0 {
            tracing::warn!(
                observed_latency_ms,
                "Non-finite or negative latency observed; returning unadjusted scaling factor 1.0"
            );
            return MAX_SCALING_FACTOR;
        }

        let error = (self.target_latency_ms - observed_latency_ms) / self.target_latency_ms;
        self.integral = (self.integral + error).clamp(-MAX_INTEGRAL, MAX_INTEGRAL);

        let derivative = match self.previous_error {
            Some(prev) => error - prev,
            None => 0.0,
        };
        self.previous_error = Some(error);

        let pid_output = (self.kp * error) + (self.ki * self.integral) + (self.kd * derivative);
        let raw_scaling = 1.0 + pid_output;

        if raw_scaling.is_finite() {
            raw_scaling.clamp(MIN_SCALING_FACTOR, MAX_SCALING_FACTOR)
        } else {
            MAX_SCALING_FACTOR
        }
    }

    /// Resets the controller integral state and error history.
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.previous_error = None;
    }
}

/// Simple, panic-free latency budget guard for iterative multi-step search loops.
#[derive(Clone)]
pub struct LatencyBudgetGuard {
    clock: Arc<dyn Clock>,
    start_monotonic_nanos: u64,
    budget_ms: f64,
}

impl std::fmt::Debug for LatencyBudgetGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LatencyBudgetGuard")
            .field("start_monotonic_nanos", &self.start_monotonic_nanos)
            .field("budget_ms", &self.budget_ms)
            .finish()
    }
}

impl LatencyBudgetGuard {
    /// Creates a new `LatencyBudgetGuard` with specified budget in ms and injected [`Clock`] port.
    pub fn new_with_clock(budget_ms: f64, clock: Arc<dyn Clock>) -> Self {
        let budget_ms = if budget_ms.is_finite() && budget_ms > 0.0 {
            budget_ms
        } else {
            DEFAULT_TARGET_LATENCY_MS
        };

        let start_monotonic_nanos = clock.monotonic_nanos();
        Self {
            clock,
            start_monotonic_nanos,
            budget_ms,
        }
    }

    /// Creates a new `LatencyBudgetGuard` with the given budget in milliseconds.
    ///
    /// Delegates to [`Self::new_with_clock`] using a standard [`SystemClock`].
    pub fn new(budget_ms: f64) -> Self {
        Self::new_with_clock(budget_ms, Arc::new(SystemClock::new()))
    }

    /// Returns elapsed time in milliseconds since guard instantiation.
    pub fn elapsed_ms(&self) -> f64 {
        let now_nanos = self.clock.monotonic_nanos();
        let elapsed_nanos = now_nanos.saturating_sub(self.start_monotonic_nanos);
        elapsed_nanos as f64 / 1_000_000.0
    }

    /// Returns `true` if the elapsed time has exceeded the configured budget.
    pub fn is_exceeded(&self) -> bool {
        self.elapsed_ms() >= self.budget_ms
    }

    /// Returns configured budget in milliseconds.
    pub fn budget_ms(&self) -> f64 {
        self.budget_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_pid_controller_default_values() {
        let pid = PidLatencyController::default();
        assert_eq!(pid.target_latency_ms, 100.0);
        assert_eq!(pid.kp, 0.5);
        assert_eq!(pid.ki, 0.05);
        assert_eq!(pid.kd, 0.1);
    }

    #[test]
    #[allow(deprecated)]
    fn test_pid_controller_latency_series_monotonically_adjusts() {
        let mut pid = PidLatencyController::new(100.0);

        let f_50 = pid.compute_adjustment(50.0);
        assert_eq!(f_50, 1.0);

        let f_200 = pid.compute_adjustment(200.0);
        assert!(f_200 < 1.0);

        let f_500 = pid.compute_adjustment(500.0);
        assert!(f_500 <= f_200);
        assert!(f_500 >= MIN_SCALING_FACTOR);

        let mut f_recovering = f_500;
        for _ in 0..10 {
            let next_f = pid.compute_adjustment(30.0);
            assert!(next_f >= f_recovering);
            f_recovering = next_f;
        }
        assert_eq!(f_recovering, 1.0);
    }

    #[test]
    #[allow(deprecated)]
    fn test_pid_controller_clamping_bounds() {
        let mut pid = PidLatencyController::new(100.0);

        for _ in 0..20 {
            let adj = pid.compute_adjustment(10000.0);
            assert_eq!(adj, MIN_SCALING_FACTOR);
        }

        for _ in 0..20 {
            let adj = pid.compute_adjustment(1.0);
            assert_eq!(adj, MAX_SCALING_FACTOR);
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_pid_controller_anti_windup_clamping() {
        let mut pid = PidLatencyController::new(100.0);

        for _ in 0..100 {
            pid.compute_adjustment(1000.0);
        }

        assert_eq!(pid.integral, -MAX_INTEGRAL);

        let f_normal = pid.compute_adjustment(100.0);
        assert!(f_normal >= MIN_SCALING_FACTOR);
    }

    #[test]
    #[allow(deprecated)]
    fn test_pid_controller_nan_and_inf_safety() {
        let mut pid = PidLatencyController::new(100.0);

        let f_nan = pid.compute_adjustment(f64::NAN);
        assert_eq!(f_nan, 1.0);

        let f_inf = pid.compute_adjustment(f64::INFINITY);
        assert_eq!(f_inf, 1.0);

        let f_neg = pid.compute_adjustment(-50.0);
        assert_eq!(f_neg, 1.0);
    }

    #[test]
    fn test_latency_budget_guard() {
        let guard = LatencyBudgetGuard::new(500.0);
        assert!(!guard.is_exceeded());
        assert_eq!(guard.budget_ms(), 500.0);
        assert!(guard.elapsed_ms() >= 0.0);

        let guard_zero = LatencyBudgetGuard::new(-10.0);
        assert_eq!(guard_zero.budget_ms(), DEFAULT_TARGET_LATENCY_MS);
    }
}
