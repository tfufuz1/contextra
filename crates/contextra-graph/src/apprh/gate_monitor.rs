// FILE-CONTEXT
// ZWECK: ApprhGateMonitor zur Serienbewertung von APPRH-Shadow-Vergleichen fuer Produktionsfreigabe (P1.3)
// INVARIANTEN: Thread-sicheres gleitendes Fenster; Hysterese-Reset bei Ausreissern.
// STAND: TS:2026-09-28T00:00:00Z

use super::error::ApprhError;
use super::shadow::{ApprhFlipGate, ApprhShadowComparison, DefaultApprhFlipGate};
use parking_lot::Mutex;
use std::collections::VecDeque;

/// Configuration for [`ApprhGateMonitor`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprhGateMonitorConfig {
    /// Maximum capacity of the sliding window for historical observations.
    pub window_size: usize,
    /// Number of consecutive valid observations required for production readiness.
    pub required_consecutive_passes: usize,
}

impl Default for ApprhGateMonitorConfig {
    fn default() -> Self {
        Self {
            window_size: 20,
            required_consecutive_passes: 10,
        }
    }
}

impl ApprhGateMonitorConfig {
    /// Validates the monitor configuration parameters.
    ///
    /// # Errors
    /// Returns [`ApprhError::InvalidParameter`] if any parameter is invalid.
    pub fn validate(&self) -> Result<(), ApprhError> {
        if self.window_size == 0 {
            return Err(ApprhError::InvalidParameter(
                "window_size must be greater than 0".to_string(),
            ));
        }
        if self.required_consecutive_passes == 0 {
            return Err(ApprhError::InvalidParameter(
                "required_consecutive_passes must be greater than 0".to_string(),
            ));
        }
        if self.required_consecutive_passes > self.window_size {
            return Err(ApprhError::InvalidParameter(
                "required_consecutive_passes cannot exceed window_size".to_string(),
            ));
        }
        Ok(())
    }
}

/// Historical observation record stored within the sliding window of [`ApprhGateMonitor`].
#[derive(Debug, Clone, PartialEq)]
pub struct ApprhObservationRecord {
    /// Maximum absolute diff from report.
    pub max_abs_diff: f32,
    /// Top-k Jaccard overlap ratio from report.
    pub top_k_overlap: f32,
    /// Boolean indicator whether discrepancy threshold was breached.
    pub discrepancy: bool,
    /// Whether this observation passed the gate evaluation and had no discrepancy.
    pub passed: bool,
}

/// Telemetry snapshot from [`ApprhGateMonitor`].
#[derive(Debug, Clone, PartialEq)]
pub struct ApprhGateMonitorSnapshot {
    /// Total number of observations received since creation or last reset.
    pub total_observations: u64,
    /// Current count of consecutive passed observations (streak).
    pub current_streak: usize,
    /// Top-k overlap ratio of the most recent observation, if any.
    pub last_overlap: Option<f32>,
    /// Maximum absolute score difference within the current sliding window.
    pub max_abs_diff_in_window: f32,
    /// Minimum top-k overlap ratio within the current sliding window.
    pub min_overlap_in_window: f32,
    /// Number of observations in the current sliding window with a discrepancy flag.
    pub discrepancy_count_in_window: usize,
    /// Whether APPRH is currently deemed ready for production replacement.
    pub production_ready: bool,
}

struct MonitorState {
    total_observations: u64,
    current_streak: usize,
    window: VecDeque<ApprhObservationRecord>,
}

/// Independent, thread-safe monitor evaluating a series of APPRH shadow mode comparisons
/// for production readiness (P1.3).
///
/// # Invariants
/// - Ring 0 (pure sync, no tokio dependencies).
/// - An outlier (gate failure or discrepancy flag) resets the pass streak to 0 (hysteresis against flickering).
/// - Thread-safe operations via internal `parking_lot::Mutex`.
pub struct ApprhGateMonitor {
    config: ApprhGateMonitorConfig,
    gate: Box<dyn ApprhFlipGate + Send + Sync>,
    state: Mutex<MonitorState>,
}

impl ApprhGateMonitor {
    /// Creates a new [`ApprhGateMonitor`] with the provided configuration and standard [`DefaultApprhFlipGate`].
    ///
    /// # Errors
    /// Returns [`ApprhError::InvalidParameter`] if `config` validation fails.
    pub fn new(config: ApprhGateMonitorConfig) -> Result<Self, ApprhError> {
        Self::with_gate(config, DefaultApprhFlipGate)
    }

    /// Creates a new [`ApprhGateMonitor`] with the provided configuration and a custom [`ApprhFlipGate`].
    ///
    /// # Errors
    /// Returns [`ApprhError::InvalidParameter`] if `config` validation fails.
    pub fn with_gate<G>(config: ApprhGateMonitorConfig, gate: G) -> Result<Self, ApprhError>
    where
        G: ApprhFlipGate + Send + Sync + 'static,
    {
        config.validate()?;
        let window_cap = config.window_size;
        Ok(Self {
            config,
            gate: Box::new(gate),
            state: Mutex::new(MonitorState {
                total_observations: 0,
                current_streak: 0,
                window: VecDeque::with_capacity(window_cap),
            }),
        })
    }

    /// Observes a single [`ApprhShadowComparison`] report, evaluating it against the injected gate.
    ///
    /// An observation is considered a pass if the gate returns `true` AND `report.discrepancy` is `false`.
    /// Any outlier resets `current_streak` to 0 (hysteresis).
    pub fn observe(&self, report: &ApprhShadowComparison) {
        let gate_passed = self.gate.should_flip(report);
        let passed = gate_passed && !report.discrepancy;

        let record = ApprhObservationRecord {
            max_abs_diff: report.max_abs_diff,
            top_k_overlap: report.top_k_overlap,
            discrepancy: report.discrepancy,
            passed,
        };

        let mut state = self.state.lock();
        state.total_observations = state.total_observations.saturating_add(1);

        if passed {
            state.current_streak = state.current_streak.saturating_add(1);
        } else {
            state.current_streak = 0;
        }

        if state.window.len() >= self.config.window_size {
            state.window.pop_front();
        }
        state.window.push_back(record);
    }

    /// Returns `true` if the monitor currently deems APPRH ready for production replacement.
    ///
    /// Production readiness is declared when the streak of consecutive clean passes
    /// meets or exceeds `required_consecutive_passes`.
    pub fn production_ready(&self) -> bool {
        let state = self.state.lock();
        state.current_streak >= self.config.required_consecutive_passes
    }

    /// Captures a telemetry snapshot of the current monitor state and sliding window.
    pub fn snapshot(&self) -> ApprhGateMonitorSnapshot {
        let state = self.state.lock();
        let production_ready = state.current_streak >= self.config.required_consecutive_passes;

        let (max_abs_diff_in_window, min_overlap_in_window, discrepancy_count_in_window) =
            if state.window.is_empty() {
                (0.0f32, 1.0f32, 0)
            } else {
                let mut max_diff = 0.0f32;
                let mut min_overlap = 1.0f32;
                let mut disc_count = 0usize;

                for rec in &state.window {
                    if rec.max_abs_diff > max_diff {
                        max_diff = rec.max_abs_diff;
                    }
                    if rec.top_k_overlap < min_overlap {
                        min_overlap = rec.top_k_overlap;
                    }
                    if rec.discrepancy {
                        disc_count += 1;
                    }
                }

                (max_diff, min_overlap, disc_count)
            };

        let last_overlap = state.window.back().map(|rec| rec.top_k_overlap);

        ApprhGateMonitorSnapshot {
            total_observations: state.total_observations,
            current_streak: state.current_streak,
            last_overlap,
            max_abs_diff_in_window,
            min_overlap_in_window,
            discrepancy_count_in_window,
            production_ready,
        }
    }

    /// Resets the monitor state, clearing the sliding window, total observations counter, and pass streak.
    pub fn reset(&self) {
        let mut state = self.state.lock();
        state.total_observations = 0;
        state.current_streak = 0;
        state.window.clear();
    }

    /// Returns a reference to the monitor's configuration.
    pub fn config(&self) -> &ApprhGateMonitorConfig {
        &self.config
    }
}

impl Default for ApprhGateMonitor {
    fn default() -> Self {
        let config = ApprhGateMonitorConfig::default();
        let window_cap = config.window_size;
        Self {
            config,
            gate: Box::new(DefaultApprhFlipGate),
            state: Mutex::new(MonitorState {
                total_observations: 0,
                current_streak: 0,
                window: VecDeque::with_capacity(window_cap),
            }),
        }
    }
}

impl std::fmt::Debug for ApprhGateMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.lock();
        f.debug_struct("ApprhGateMonitor")
            .field("config", &self.config)
            .field("total_observations", &state.total_observations)
            .field("current_streak", &state.current_streak)
            .field("window_len", &state.window.len())
            .field(
                "production_ready",
                &(state.current_streak >= self.config.required_consecutive_passes),
            )
            .finish()
    }
}
