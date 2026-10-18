// FILE-CONTEXT
// STAND: 2026-10-17T00:00:00Z
// ZWECK: Position-form PID PoolController with back-calculation anti-windup and measurement filtering.
// INVARIANTEN: Candidate pool bounded strictly in [min_pool, max_pool], min_pool > 0, validated config, zero panics.

//! Consolidated position-form PID controller for candidate pool size regulation.
//!
//! Controls retrieval/reranking candidate pool size $P$ to maintain target latency $r$:
//! - Position form: $P = \text{clamp}(P_{\text{nominal}} + u, P_{\text{min}}, P_{\text{max}})$
//! - Control signal: $u = K_p \cdot e + K_i \cdot I + K_d \cdot \dot{e}$
//! - Error signal: $e = r - y$ (target latency - measured latency)
//! - Derivative term: $\dot{e} = -\dot{y}_f$ derived from low-pass-filtered measurement (no setpoint kick).
//! - Anti-windup: Back-calculation tracking $\frac{dI}{dt} = e + K_{bc} (u_{\text{clamped}} - u_{\text{unclamped}})$.
//!
//! # Scientific Validation
//! Empirical, measured internally (see `crates/contextra-adapt/benches/pool_controller_bench.rs`).

use contextra_ports::Clock;
use contextra_types::{ContextraError, Result};
use serde::{Deserialize, Serialize};

/// Clock trait for control loops enabling deterministic nano-resolution timestamping.
pub trait ControlClock: Send + Sync {
    /// Returns current monotonic timestamp in nanoseconds.
    fn now_nanos(&self) -> u64;
}

impl<T: Clock + ?Sized> ControlClock for T {
    fn now_nanos(&self) -> u64 {
        self.monotonic_ticks()
    }
}

/// Dummy struct for clock implementation using direct `dt` intervals.
#[derive(Debug, Clone, Copy, Default)]
pub struct ManualControlClock {
    now_nanos: u64,
}

impl ManualControlClock {
    /// Creates a manual clock initialized to the given timestamp in nanoseconds.
    pub fn new(now_nanos: u64) -> Self {
        Self { now_nanos }
    }

    /// Advances the manual clock by `dt_nanos`.
    pub fn advance_nanos(&mut self, dt_nanos: u64) {
        self.now_nanos = self.now_nanos.saturating_add(dt_nanos);
    }
}

impl ControlClock for ManualControlClock {
    fn now_nanos(&self) -> u64 {
        self.now_nanos
    }
}

/// Configuration parameters for [`PoolController`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PoolControllerConfig {
    /// Proportional gain $K_p$.
    pub kp: f32,
    /// Integral gain $K_i$.
    pub ki: f32,
    /// Derivative gain $K_d$.
    pub kd: f32,
    /// Minimum allowed candidate pool size (must be $> 0$, default 50).
    pub min_pool: usize,
    /// Maximum allowed candidate pool size (must be $\ge \text{min\_pool}$, default 200).
    pub max_pool: usize,
    /// Nominal candidate pool size baseline (must be in $[\text{min\_pool}, \text{max\_pool}]$, default 100).
    pub pool_nominal: usize,
    /// Target latency budget in milliseconds (must be $> 0$).
    pub target_latency_ms: f32,
    /// Low-pass filter time constant $T_f$ in seconds for measurement filtering (must be $> 0$).
    pub filter_time_constant_s: f32,
}

impl Default for PoolControllerConfig {
    fn default() -> Self {
        Self {
            kp: 0.5,
            ki: 0.05,
            kd: 0.1,
            min_pool: 50,
            max_pool: 200,
            pool_nominal: 100,
            target_latency_ms: 100.0,
            filter_time_constant_s: 0.05,
        }
    }
}

impl PoolControllerConfig {
    /// Validates the configuration parameters.
    pub fn validate(&self) -> Result<()> {
        if !self.kp.is_finite() || self.kp < 0.0 {
            return Err(ContextraError::InvalidInput(format!(
                "kp must be finite and non-negative, got {}",
                self.kp
            )));
        }
        if !self.ki.is_finite() || self.ki < 0.0 {
            return Err(ContextraError::InvalidInput(format!(
                "ki must be finite and non-negative, got {}",
                self.ki
            )));
        }
        if !self.kd.is_finite() || self.kd < 0.0 {
            return Err(ContextraError::InvalidInput(format!(
                "kd must be finite and non-negative, got {}",
                self.kd
            )));
        }
        if self.min_pool == 0 {
            return Err(ContextraError::InvalidInput(
                "min_pool must be greater than 0".to_string(),
            ));
        }
        if self.max_pool < self.min_pool {
            return Err(ContextraError::InvalidInput(format!(
                "max_pool ({}) must be >= min_pool ({})",
                self.max_pool, self.min_pool
            )));
        }
        if self.pool_nominal < self.min_pool || self.pool_nominal > self.max_pool {
            return Err(ContextraError::InvalidInput(format!(
                "pool_nominal ({}) must be in range [min_pool ({}), max_pool ({})]",
                self.pool_nominal, self.min_pool, self.max_pool
            )));
        }
        if !self.target_latency_ms.is_finite() || self.target_latency_ms <= 0.0 {
            return Err(ContextraError::InvalidInput(format!(
                "target_latency_ms must be finite and positive, got {}",
                self.target_latency_ms
            )));
        }
        if !self.filter_time_constant_s.is_finite() || self.filter_time_constant_s <= 0.0 {
            return Err(ContextraError::InvalidInput(format!(
                "filter_time_constant_s must be finite and positive, got {}",
                self.filter_time_constant_s
            )));
        }
        Ok(())
    }
}

/// Consolidated Position-Form PID Controller for Candidate Pool Regulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolController {
    config: PoolControllerConfig,
    current_pool: usize,
    integral: f32,
    filtered_measurement: Option<f32>,
    last_nanos: Option<u64>,
}

impl Default for PoolController {
    fn default() -> Self {
        let config = PoolControllerConfig::default();
        Self {
            config,
            current_pool: config.pool_nominal,
            integral: 0.0,
            filtered_measurement: None,
            last_nanos: None,
        }
    }
}

impl PoolController {
    /// Creates a new `PoolController` from validated configuration parameters.
    pub fn new(config: PoolControllerConfig) -> Result<Self> {
        config.validate()?;
        let initial_pool = config.pool_nominal;
        Ok(Self {
            config,
            current_pool: initial_pool,
            integral: 0.0,
            filtered_measurement: None,
            last_nanos: None,
        })
    }

    /// Returns the active configuration.
    pub fn config(&self) -> &PoolControllerConfig {
        &self.config
    }

    /// Returns the currently recommended pool size.
    pub fn current_pool(&self) -> usize {
        self.current_pool
    }

    /// Resets the controller internal integral accumulator and filter state.
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.filtered_measurement = None;
        self.last_nanos = None;
        self.current_pool = self.config.pool_nominal;
    }

    /// Updates controller state with a measured latency value using a timestamp from a [`ControlClock`].
    pub fn update_with_clock(
        &mut self,
        clock: &dyn ControlClock,
        measured_latency_ms: f32,
    ) -> usize {
        let now = clock.now_nanos();
        let dt_s = match self.last_nanos {
            Some(prev) => {
                let diff = now.saturating_sub(prev);
                (diff as f32 / 1_000_000_000.0).clamp(0.001, 10.0)
            }
            None => 0.1, // Default initial step size
        };
        self.last_nanos = Some(now);
        self.update_step(dt_s, measured_latency_ms)
    }

    /// Updates controller state directly given time delta `dt_s` in seconds and `measured_latency_ms`.
    pub fn update_step(&mut self, dt_s: f32, measured_latency_ms: f32) -> usize {
        if !measured_latency_ms.is_finite() || measured_latency_ms < 0.0 {
            return self.current_pool;
        }

        let dt_s = dt_s.clamp(0.001, 10.0);

        // Low-pass filter measurement: y_f
        let tf = self.config.filter_time_constant_s;
        let alpha_f = dt_s / (dt_s + tf);
        let (y_f, dy_f) = match self.filtered_measurement {
            Some(prev_yf) => {
                let new_yf = prev_yf + alpha_f * (measured_latency_ms - prev_yf);
                let dy = (new_yf - prev_yf) / dt_s;
                (new_yf, dy)
            }
            None => (measured_latency_ms, 0.0),
        };
        self.filtered_measurement = Some(y_f);

        // Error: target - measured
        let error = self.config.target_latency_ms - measured_latency_ms;

        // Derivative of error: de = -dy_f (since setpoint target is constant)
        let de = -dy_f;

        // Unclamped output signal: u_unclamped = Kp*e + Ki*I + Kd*de
        let u_unclamped = self.config.kp * error + self.config.ki * self.integral + self.config.kd * de;

        // Candidate pool calculation in position form: pool_nominal + u
        let pool_nominal_f = self.config.pool_nominal as f32;
        let p_unclamped = pool_nominal_f + u_unclamped;

        let min_f = self.config.min_pool as f32;
        let max_f = self.config.max_pool as f32;
        let p_clamped = p_unclamped.clamp(min_f, max_f);

        // Saturation tracking for back-calculation anti-windup:
        let u_clamped = p_clamped - pool_nominal_f;
        let e_s = u_clamped - u_unclamped;

        // Tracking gain K_bc = 1 / Kp (if Kp > 0) else 1.0
        let k_bc = if self.config.kp > 1e-6 {
            1.0 / self.config.kp
        } else {
            1.0
        };

        // Integral accumulation update with back-calculation anti-windup
        let d_integral = error + k_bc * e_s;
        self.integral += d_integral * dt_s;

        let new_pool = p_clamped.round() as usize;
        self.current_pool = new_pool.clamp(self.config.min_pool, self.config.max_pool);
        self.current_pool
    }
}
