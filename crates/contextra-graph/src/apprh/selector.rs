// FILE-CONTEXT
// ZWECK: ApprhSelector zur Steuerung von Shadow-Mode-Abtastung und expliziter Produktionsschaltung (P1.3)
// INVARIANTEN: Instanz-Objekt ohne globalen statischen Zustand (ADR-N10, P29).
// STAND: TS:2026-09-28T00:00:00Z

use super::error::ApprhError;
use super::gate_monitor::ApprhGateMonitor;
use super::params::ApprhParams;
use super::shadow::{normalize_and_sort, shadow_compare_forward_push_vs_apprh};
use crate::path_rag::{forward_push_ppr, PathGraph, PprParams};
use contextra_types::EntityId;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Operating mode for [`ApprhSelector`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ApprhMode {
    /// APPRH disabled; authoritative Forward-Push PPR is returned.
    Off,
    /// APPRH shadow mode (default): returns authoritative Forward-Push PPR result,
    /// and periodically evaluates shadow comparisons against APPRH.
    #[default]
    Shadow,
    /// APPRH production mode: returns APPRH results if gate monitor is production ready;
    /// automatically falls back to Shadow mode if monitor loses readiness (hysteresis).
    Production,
}

/// Configuration parameters for [`ApprhSelector`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprhSelectorConfig {
    /// Sampling rate for shadow evaluations: 1 = every call, N = every N-th call.
    pub sample_interval: u64,
    /// Maximum number of seed nodes allowed for shadow evaluation (budget limit).
    pub max_seeds: usize,
    /// Top-k parameter for shadow comparison set overlap evaluation.
    pub shadow_top_k: usize,
}

impl Default for ApprhSelectorConfig {
    fn default() -> Self {
        Self {
            sample_interval: 1,
            max_seeds: 100,
            shadow_top_k: 10,
        }
    }
}

/// Instance-level selector managing APPRH shadow-mode evaluation and production switching (P1.3).
///
/// # Invariants
/// - Instance object without global static state (ADR-N10, P29).
/// - Shadow mode ALWAYS returns Forward-Push PPR results (authoritative).
/// - Production release is strictly explicit (`promote_to_production` / `set_mode`).
/// - Hysteresis: If monitor loses readiness in Production mode, selector automatically falls back
///   to Shadow mode and emits a `tracing::warn!` log.
pub struct ApprhSelector {
    config: ApprhSelectorConfig,
    monitor: Arc<ApprhGateMonitor>,
    mode: RwLock<ApprhMode>,
    call_counter: AtomicU64,
}

impl ApprhSelector {
    /// Creates a new [`ApprhSelector`] with the specified configuration and gate monitor.
    pub fn new(config: ApprhSelectorConfig, monitor: Arc<ApprhGateMonitor>) -> Self {
        Self {
            config,
            monitor,
            mode: RwLock::new(ApprhMode::Shadow),
            call_counter: AtomicU64::new(0),
        }
    }

    /// Creates a selector with default configuration and standard gate monitor.
    ///
    /// # Errors
    /// Returns [`ApprhError`] if monitor configuration fails validation.
    pub fn with_default_monitor(config: ApprhSelectorConfig) -> Result<Self, ApprhError> {
        let monitor = Arc::new(ApprhGateMonitor::default());
        Ok(Self::new(config, monitor))
    }

    /// Returns the current operating mode of the selector.
    pub fn mode(&self) -> ApprhMode {
        *self.mode.read()
    }

    /// Explicitly sets the operating mode of the selector.
    ///
    /// # Errors
    /// Returns [`ApprhError::InvalidParameter`] if attempting to promote to `Production` mode
    /// when `monitor.production_ready()` is `false`.
    pub fn set_mode(&self, new_mode: ApprhMode) -> Result<(), ApprhError> {
        if new_mode == ApprhMode::Production && !self.monitor.production_ready() {
            return Err(ApprhError::InvalidParameter(
                "Cannot promote to Production mode: ApprhGateMonitor is not production ready"
                    .to_string(),
            ));
        }
        *self.mode.write() = new_mode;
        Ok(())
    }

    /// Explicitly promotes the selector to `Production` mode.
    ///
    /// # Errors
    /// Returns [`ApprhError::InvalidParameter`] if gate monitor is not production ready.
    pub fn promote_to_production(&self) -> Result<(), ApprhError> {
        self.set_mode(ApprhMode::Production)
    }

    /// Explicitly demotes the selector to `Shadow` mode.
    pub fn demote_to_shadow(&self) {
        *self.mode.write() = ApprhMode::Shadow;
    }

    /// Returns a reference to the internal [`ApprhGateMonitor`].
    pub fn monitor(&self) -> &ApprhGateMonitor {
        &self.monitor
    }

    /// Evaluates and executes Personalized PageRank over the graph according to current operating mode.
    ///
    /// # Errors
    /// Returns [`ApprhError`] if graph execution fails in non-recoverable paths.
    pub fn evaluate_and_run<G: PathGraph>(
        &self,
        graph: &G,
        seeds: &[EntityId],
        ppr_params: &PprParams,
        apprh_params: &ApprhParams,
    ) -> Result<Vec<(EntityId, f32)>, ApprhError> {
        let current_mode = self.mode();

        match current_mode {
            ApprhMode::Off => {
                let fp_map = forward_push_ppr(graph, seeds, ppr_params);
                Ok(normalize_and_sort(&fp_map))
            }
            ApprhMode::Shadow => {
                let fp_map = forward_push_ppr(graph, seeds, ppr_params);
                let fp_sorted = normalize_and_sort(&fp_map);

                let call_num = self.call_counter.fetch_add(1, Ordering::Relaxed) + 1;
                let is_sampled = self.config.sample_interval > 0
                    && (call_num % self.config.sample_interval == 0);
                let within_budget = seeds.len() <= self.config.max_seeds;

                if is_sampled && within_budget {
                    match shadow_compare_forward_push_vs_apprh(
                        graph,
                        seeds,
                        ppr_params,
                        apprh_params,
                        self.config.shadow_top_k,
                    ) {
                        Ok(report) => {
                            self.monitor.observe(&report);
                        }
                        Err(err) => {
                            tracing::warn!(error = %err, "APPRH shadow comparison failed during selector sampling");
                        }
                    }
                }

                Ok(fp_sorted)
            }
            ApprhMode::Production => {
                if !self.monitor.production_ready() {
                    tracing::warn!(
                        "APPRH monitor lost production readiness; reverting selector to Shadow mode"
                    );
                    self.demote_to_shadow();
                    return self.evaluate_and_run(graph, seeds, ppr_params, apprh_params);
                }

                match super::diffusion::run_apprh_push(graph, seeds, apprh_params) {
                    Ok(apprh_map) => Ok(normalize_and_sort(&apprh_map)),
                    Err(err) => {
                        tracing::warn!(
                            error = %err,
                            "APPRH execution failed in Production mode; falling back to Forward-Push"
                        );
                        let fp_map = forward_push_ppr(graph, seeds, ppr_params);
                        Ok(normalize_and_sort(&fp_map))
                    }
                }
            }
        }
    }
}

impl std::fmt::Debug for ApprhSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApprhSelector")
            .field("config", &self.config)
            .field("mode", &self.mode())
            .field("call_counter", &self.call_counter.load(Ordering::Relaxed))
            .field("monitor", &self.monitor)
            .finish()
    }
}
