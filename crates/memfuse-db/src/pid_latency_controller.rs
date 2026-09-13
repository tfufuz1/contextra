// FILE-CONTEXT
// ZWECK: PID Latency Controller & Latency Budget Guard für Multi-Step Retrieval.
// INVARIANTEN: Skalierungsfaktor immer in [0.3, 1.0] geklemmt; Integrator-Anti-Windup verhindert Oszillation.
// RISIKO-FOKUS: Numerische Stabilität (kein Integrator-Windup, Non-Finite Schutz, Clamping).
// STAND: TS:2026-09-13T00:00:00Z (SESSION: pid-regler-impl)

use std::time::Instant;

/// Konservativer Default für das P95-Latenzziel in Millisekunden (100ms).
///
/// HINWEIS: implementation_plan-db2.md identifiziert den genauen P95-Zielwert
/// als offene Frage (50ms vs. 100ms). Wir wählen 100ms als konservativen Default
/// und machen den Zielwert über `PidLatencyController::new(target_latency_ms)`
/// vollständig konfigurierbar.
pub const DEFAULT_TARGET_LATENCY_MS: f64 = 100.0;

/// Unterer Clamp für den K-Pool-Skalierungsfaktor (30% des konfigurierten K-Pools),
/// um APM-RECALL-COLLAPSE in Standardlastszenarien zu verhindern.
pub const MIN_SCALING_FACTOR: f64 = 0.3;

/// Oberer Clamp für den K-Pool-Skalierungsfaktor (100% des konfigurierten K-Pools).
pub const MAX_SCALING_FACTOR: f64 = 1.0;

/// Max-Grenze für den I-Anteil (Anti-Windup Clamping).
pub const MAX_INTEGRAL: f64 = 10.0;

/// Classical PID controller for dynamically scaling multi-step retrieval parameters (`k_pool`, `max_hops`).
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

impl Default for PidLatencyController {
    fn default() -> Self {
        Self::new(DEFAULT_TARGET_LATENCY_MS)
    }
}

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
            ki: if ki.is_finite() && ki >= 0.0 { ki } else { 0.05 },
            kd: if kd.is_finite() && kd >= 0.0 { kd } else { 0.1 },
            target_latency_ms,
            integral: 0.0,
            previous_error: None,
        }
    }

    /// Calculates scaling adjustment factor in range `[0.3, 1.0]` based on observed latency.
    ///
    /// - Observed latency > target_latency => scaling factor decreases towards 0.3.
    /// - Observed latency <= target_latency => scaling factor recovers towards 1.0.
    /// - Non-finite inputs (NaN / Inf) are safely handled without panicking or corrupting state.
    pub fn compute_adjustment(&mut self, observed_latency_ms: f64) -> f64 {
        if !observed_latency_ms.is_finite() || observed_latency_ms < 0.0 {
            tracing::warn!(
                observed_latency_ms,
                "Non-finite or negative latency observed; returning unadjusted scaling factor 1.0"
            );
            return MAX_SCALING_FACTOR;
        }

        // Error signal: normalized error relative to target budget.
        // Positive error indicates latency is UNDER budget.
        // Negative error indicates latency is OVER budget.
        let error = (self.target_latency_ms - observed_latency_ms) / self.target_latency_ms;

        // Anti-windup clamping for integral accumulator.
        self.integral = (self.integral + error).clamp(-MAX_INTEGRAL, MAX_INTEGRAL);

        // Derivative term.
        let derivative = match self.previous_error {
            Some(prev) => error - prev,
            None => 0.0,
        };
        self.previous_error = Some(error);

        // PID output calculation.
        let pid_output = (self.kp * error) + (self.ki * self.integral) + (self.kd * derivative);

        // Base scaling factor = 1.0 + pid_output, clamped to [0.3, 1.0].
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
#[derive(Debug, Clone)]
pub struct LatencyBudgetGuard {
    start: Instant,
    budget_ms: f64,
}

impl LatencyBudgetGuard {
    /// Creates a new `LatencyBudgetGuard` with the given budget in milliseconds.
    pub fn new(budget_ms: f64) -> Self {
        let budget_ms = if budget_ms.is_finite() && budget_ms > 0.0 {
            budget_ms
        } else {
            DEFAULT_TARGET_LATENCY_MS
        };

        Self {
            start: Instant::now(),
            budget_ms,
        }
    }

    /// Returns elapsed time in milliseconds since guard instantiation.
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
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
    fn test_pid_controller_default_values() {
        let pid = PidLatencyController::default();
        assert_eq!(pid.target_latency_ms, 100.0);
        assert_eq!(pid.kp, 0.5);
        assert_eq!(pid.ki, 0.05);
        assert_eq!(pid.kd, 0.1);
    }

    #[test]
    fn test_pid_controller_latency_series_monotonically_adjusts() {
        let mut pid = PidLatencyController::new(100.0);

        // Under budget (50ms) -> scaling factor stays at MAX (1.0)
        let f_50 = pid.compute_adjustment(50.0);
        assert_eq!(f_50, 1.0);

        // High latency over budget (200ms) -> scaling factor decreases
        let f_200 = pid.compute_adjustment(200.0);
        assert!(f_200 < 1.0);

        // Even higher latency (500ms) -> scaling factor decreases further towards MIN (0.3)
        let f_500 = pid.compute_adjustment(500.0);
        assert!(f_500 <= f_200);
        assert!(f_500 >= MIN_SCALING_FACTOR);

        // Latency drops back under budget (30ms) -> scaling factor recovers
        let mut f_recovering = f_500;
        for _ in 0..10 {
            let next_f = pid.compute_adjustment(30.0);
            assert!(next_f >= f_recovering);
            f_recovering = next_f;
        }
        assert_eq!(f_recovering, 1.0);
    }

    #[test]
    fn test_pid_controller_clamping_bounds() {
        let mut pid = PidLatencyController::new(100.0);

        // Extreme latency (10,000ms) must clamp to MIN_SCALING_FACTOR (0.3)
        for _ in 0..20 {
            let adj = pid.compute_adjustment(10000.0);
            assert_eq!(adj, MIN_SCALING_FACTOR);
        }

        // Extreme low latency (1ms) must clamp to MAX_SCALING_FACTOR (1.0)
        for _ in 0..20 {
            let adj = pid.compute_adjustment(1.0);
            assert_eq!(adj, MAX_SCALING_FACTOR);
        }
    }

    #[test]
    fn test_pid_controller_anti_windup_clamping() {
        let mut pid = PidLatencyController::new(100.0);

        // Saturate integral with extreme over-budget latencies
        for _ in 0..100 {
            pid.compute_adjustment(1000.0);
        }

        // Verify integral magnitude is clamped to MAX_INTEGRAL
        assert_eq!(pid.integral, -MAX_INTEGRAL);

        // Immediately dropping latency to budget (100ms) should begin integral recovery without infinite delay
        let f_normal = pid.compute_adjustment(100.0);
        assert!(f_normal >= MIN_SCALING_FACTOR);
    }

    #[test]
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
