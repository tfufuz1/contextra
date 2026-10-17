// FILE-CONTEXT
// STAND: 2026-10-17T00:00:00Z
// ZWECK: Deprecated PidController wrapper for candidate pool size regulation.
// INVARIANTEN: Backward compatibility for external callers delegating directly to PoolController.

//! Deprecated PID controller wrapper for candidate pool size regulation.
//!
//! DEPRECATED: Use [`crate::PoolController`] instead.
//!
//! # Scientific Foundation
//! Empirical, measured internally (see `crates/contextra-adapt/benches/pool_controller_bench.rs`).

use crate::pool_controller::{PoolController, PoolControllerConfig};
use std::time::Duration;

/// Minimale Kandidaten-Pool-Größe.
pub const PID_MIN_POOL_SIZE_DEFAULT: usize = 50;

/// Maximale Kandidaten-Pool-Größe.
pub const PID_MAX_POOL_SIZE_DEFAULT: usize = 200;

/// Trait for anti-windup controller updates.
///
/// DEPRECATED: Use [`crate::PoolController`] instead.
pub trait AntiWindupController: Send + Sync {
    /// Updates controller with anti-windup.
    fn update_with_anti_windup(
        &mut self,
        dt: Duration,
        measured_latency_ms: f32,
        is_actuator_saturated: bool,
    ) -> usize;
}

/// PID controller wrapper.
///
/// DEPRECATED: Use [`crate::PoolController`] instead.
#[deprecated(
    since = "0.1.0",
    note = "use PoolController; removal 2027-01-31"
)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PidController {
    /// Proportional gain.
    pub kp: f32,
    /// Integral gain.
    pub ki: f32,
    /// Derivative gain.
    pub kd: f32,
    /// Target latency in ms.
    pub target_latency_ms: f32,
    /// Minimum pool size.
    pub min_pool_size: usize,
    /// Maximum pool size.
    pub max_pool_size: usize,
    /// Current pool size.
    pub current_pool_size: Option<usize>,
    integral: f32,
    prev_error: f32,
    max_integral: f32,
    inner: PoolController,
}

#[allow(deprecated)]
impl Default for PidController {
    fn default() -> Self {
        Self::new(
            150.0,
            PID_MIN_POOL_SIZE_DEFAULT,
            PID_MAX_POOL_SIZE_DEFAULT,
            None,
        )
    }
}

#[allow(deprecated)]
impl PidController {
    /// Creates a new `PidController`.
    pub fn new(
        target_latency_ms: f32,
        min_pool_size: usize,
        max_pool_size: usize,
        initial_pool: Option<usize>,
    ) -> Self {
        let min_pool_size = min_pool_size.max(PID_MIN_POOL_SIZE_DEFAULT);
        let max_pool_size = max_pool_size.max(min_pool_size);
        let pool_nominal = initial_pool.unwrap_or(min_pool_size).clamp(min_pool_size, max_pool_size);

        let config = PoolControllerConfig {
            kp: 0.5,
            ki: 0.05,
            kd: 0.1,
            min_pool: min_pool_size,
            max_pool: max_pool_size,
            pool_nominal,
            target_latency_ms: if target_latency_ms.is_finite() && target_latency_ms > 0.0 {
                target_latency_ms
            } else {
                150.0
            },
            filter_time_constant_s: 0.05,
        };

        let inner = PoolController::new(config).unwrap_or_default();
        Self {
            kp: 0.5,
            ki: 0.05,
            kd: 0.1,
            target_latency_ms,
            min_pool_size,
            max_pool_size,
            current_pool_size: Some(inner.current_pool()),
            integral: 0.0,
            prev_error: 0.0,
            max_integral: 100.0,
            inner,
        }
    }

    /// Accessor for kp.
    pub fn kp(&self) -> f32 {
        self.kp
    }

    /// Accessor for ki.
    pub fn ki(&self) -> f32 {
        self.ki
    }

    /// Accessor for kd.
    pub fn kd(&self) -> f32 {
        self.kd
    }

    /// Accessor for target latency.
    pub fn target_latency_ms(&self) -> f32 {
        self.target_latency_ms
    }

    /// Accessor for min pool size.
    pub fn min_pool_size(&self) -> usize {
        self.min_pool_size
    }

    /// Accessor for max pool size.
    pub fn max_pool_size(&self) -> usize {
        self.max_pool_size
    }

    /// Updates controller state with measured latency.
    pub fn update(&mut self, dt: Duration, measured_latency_ms: f32) -> usize {
        self.update_with_anti_windup(dt, measured_latency_ms, false)
    }

    /// Returns currently recommended pool size.
    pub fn current_pool_size(&self) -> Option<usize> {
        self.current_pool_size
    }

    /// Resets internal controller state.
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
        self.inner.reset();
        self.current_pool_size = Some(self.inner.current_pool());
    }
}

#[allow(deprecated)]
impl AntiWindupController for PidController {
    fn update_with_anti_windup(
        &mut self,
        dt: Duration,
        measured_latency_ms: f32,
        is_actuator_saturated: bool,
    ) -> usize {
        let current_pool = self.current_pool_size.unwrap_or(self.min_pool_size);
        if !measured_latency_ms.is_finite() {
            return current_pool;
        }

        let dt_s = dt.as_secs_f32().clamp(0.001, 10.0);
        let error = self.target_latency_ms - measured_latency_ms;

        if !is_actuator_saturated {
            self.integral =
                (self.integral + error * dt_s).clamp(-self.max_integral, self.max_integral);
        }

        let derivative = (error - self.prev_error) / dt_s;
        let u = self.kp * error + self.ki * self.integral + self.kd * derivative;
        self.prev_error = error;

        let new_size = (current_pool as f32 + u).round() as isize;
        let min_s = self.min_pool_size as isize;
        let max_s = self.max_pool_size as isize;

        let _ = self.inner.update_step(dt_s, measured_latency_ms);
        let clamped = new_size.clamp(min_s, max_s) as usize;
        self.current_pool_size = Some(clamped);
        clamped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_pid_stable_at_target() {
        let mut pid = PidController::new(150.0, 50, 200, Some(100));
        let dt = Duration::from_millis(100);
        let initial_pool = 100;
        let new_pool = pid.update(dt, 150.0);
        assert_eq!(new_pool, initial_pool);
        assert_eq!(pid.current_pool_size(), Some(initial_pool));
    }
}
