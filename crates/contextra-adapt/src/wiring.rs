//! Composition Root Wiring for `contextra-adapt`.

use std::sync::{Arc, Mutex};

use contextra_ports::{Clock, SystemClock};
use contextra_types::ContextraError;

#[cfg(feature = "bandit-routing")]
use crate::bandit::{BanditPolicy, BanditProfileState};
use crate::decay_controller::AdaptiveDecayController;
use crate::drift::{CatoniDriftDetector, DriftDetector, EnsembleDriftWatcher};
#[cfg(feature = "flow-corrected-thompson")]
use crate::flow_thompson::{BanditDefaultFlipGate, DefaultBanditFlipGate};
use crate::lyapunov::LyapunovDriftWatcher;
use crate::pid::{AntiWindupController, PidController};
use crate::pid_latency_controller::{LatencyBudgetGuard, PidLatencyController};
use crate::shadow_mode::TracingShadowSink;

/// Dependencies required to build an [`AdaptWiring`] composition root.
#[derive(Clone)]
pub struct AdaptWiringDeps {
    /// Injected [`Clock`] port instance.
    pub clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for AdaptWiringDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdaptWiringDeps")
            .field("clock", &"<dyn Clock>")
            .finish()
    }
}

impl Default for AdaptWiringDeps {
    fn default() -> Self {
        Self {
            clock: Arc::new(SystemClock::new()),
        }
    }
}

/// Composition Root Wiring bundle for `contextra-adapt`.
#[derive(Clone)]
pub struct AdaptWiring {
    /// Injected [`Clock`] port.
    pub clock: Arc<dyn Clock>,
    /// Active PID controller for candidate pool size regulation.
    pub pid_controller: Arc<Mutex<PidController>>,
    /// Active PID latency controller for multi-step search latency budgeting.
    pub pid_latency_controller: Arc<Mutex<PidLatencyController>>,
    /// Default latency budget guard initialized with injected clock.
    pub default_latency_guard: LatencyBudgetGuard,
    /// Active decay controller for time-weighted chunk eviction.
    pub decay_controller: Arc<AdaptiveDecayController>,
    /// Active homeostat PID controller.
    pub homeostat: Arc<Mutex<PidController>>,
    /// Anti-windup controller trait object handle.
    pub anti_windup_controller: Arc<Mutex<dyn AntiWindupController>>,
    /// Catoni drift detector.
    pub catoni_drift_detector: Arc<Mutex<CatoniDriftDetector>>,
    /// Generic drift detector trait object handle.
    pub drift_detector: Arc<Mutex<dyn DriftDetector>>,
    /// Ensemble drift watcher combining Catoni and Lyapunov detectors.
    pub ensemble_drift_watcher: Arc<Mutex<EnsembleDriftWatcher>>,
    /// Bandit profile state (available with `bandit-routing` feature).
    #[cfg(feature = "bandit-routing")]
    pub bandit_profile_state: Arc<Mutex<BanditProfileState>>,
    /// Bandit policy trait object handle (available with `bandit-routing` feature).
    #[cfg(feature = "bandit-routing")]
    pub bandit_policy: Arc<Mutex<dyn BanditPolicy>>,
    /// Default bandit flip gate trait object handle (available with `flow-corrected-thompson` feature).
    #[cfg(feature = "flow-corrected-thompson")]
    pub flip_gate: Arc<dyn BanditDefaultFlipGate>,
    /// Shadow sink handle for non-influencing telemetry routing.
    pub shadow_sink: Arc<TracingShadowSink>,
}

impl AdaptWiring {
    /// Builds and wires all `contextra-adapt` components into an [`AdaptWiring`] bundle.
    pub fn build(deps: AdaptWiringDeps) -> Result<Self, ContextraError> {
        let clock = deps.clock;
        let pid_controller = Arc::new(Mutex::new(PidController::default()));
        let pid_latency_controller = Arc::new(Mutex::new(PidLatencyController::default()));
        let default_latency_guard = LatencyBudgetGuard::new_with_clock(100.0, clock.clone());
        let decay_controller = Arc::new(AdaptiveDecayController::with_defaults());
        let homeostat = Arc::new(Mutex::new(PidController::default()));

        let anti_windup_controller: Arc<Mutex<dyn AntiWindupController>> = pid_controller.clone();

        let catoni_drift_detector = Arc::new(Mutex::new(CatoniDriftDetector::default()));
        let drift_detector: Arc<Mutex<dyn DriftDetector>> = catoni_drift_detector.clone();

        let ensemble_drift_watcher = Arc::new(Mutex::new(EnsembleDriftWatcher::new(
            LyapunovDriftWatcher::default(),
            CatoniDriftDetector::default(),
        )));

        #[cfg(feature = "bandit-routing")]
        let bandit_profile_state = Arc::new(Mutex::new(BanditProfileState::cold_start(16, 0.5)));
        #[cfg(feature = "bandit-routing")]
        let bandit_policy: Arc<Mutex<dyn BanditPolicy>> =
            Arc::new(Mutex::new(BanditProfileState::cold_start(16, 0.5)));

        #[cfg(feature = "flow-corrected-thompson")]
        let flip_gate: Arc<dyn BanditDefaultFlipGate> = Arc::new(DefaultBanditFlipGate);

        let shadow_sink = Arc::new(TracingShadowSink::default());

        Ok(Self {
            clock,
            pid_controller,
            pid_latency_controller,
            default_latency_guard,
            decay_controller,
            homeostat,
            anti_windup_controller,
            catoni_drift_detector,
            drift_detector,
            ensemble_drift_watcher,
            #[cfg(feature = "bandit-routing")]
            bandit_profile_state,
            #[cfg(feature = "bandit-routing")]
            bandit_policy,
            #[cfg(feature = "flow-corrected-thompson")]
            flip_gate,
            shadow_sink,
        })
    }
}
