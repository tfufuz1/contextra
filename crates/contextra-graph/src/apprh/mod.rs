//! Averaging-based Personalized PageRank for Hypergraphs (APPRH).

pub mod diffusion;
pub mod error;
pub mod gate_monitor;
pub mod params;
pub mod selector;
pub mod shadow;

pub use diffusion::run_apprh_push;
pub use error::ApprhError;
pub use gate_monitor::{
    ApprhGateMonitor, ApprhGateMonitorConfig, ApprhGateMonitorSnapshot, ApprhObservationRecord,
};
pub use params::ApprhParams;
pub use selector::{ApprhMode, ApprhSelector, ApprhSelectorConfig};
pub use shadow::{
    shadow_compare_forward_push_vs_apprh, ApprhFlipGate, ApprhShadowComparison,
    DefaultApprhFlipGate,
};

use crate::csr::CsrGraph;
use crate::path_rag::{PathGraph, PprParams};
use ahash::AHashMap;
use contextra_types::EntityId;

/// Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH) over a generic [`PathGraph`].
///
/// # Invariants
/// Runs exclusively in shadow mode unless feature-flag activated. Bit-identical determinism.
///
/// # Complexity
/// Time complexity bounded by push budget $O(1 / (\alpha \cdot \epsilon))$, independent of total graph size.
pub fn apprh_local<G: PathGraph>(
    graph: &G,
    seeds: &[EntityId],
    params: &ApprhParams,
) -> Result<AHashMap<EntityId, f32>, ApprhError> {
    run_apprh_push(graph, seeds, params)
}

/// Spec-compliant additive APPRH operator (Spec Y.1.2).
///
/// Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH) over a generic [`PathGraph`].
/// Strictly orders push candidates and neighbor iterations by [`EntityId`] for bit-identical IEEE-754 determinism.
///
/// # Errors
/// Returns [`contextra_types::ContextraError::InvalidInput`] if `hyperedge_decay_factor` is non-finite or outside `(0.0, 1.0]`.
pub fn forward_push_apprh<G: PathGraph>(
    graph: &G,
    seeds: &[EntityId],
    params: &PprParams,
    hyperedge_decay_factor: f32,
) -> Result<AHashMap<EntityId, f32>, contextra_types::ContextraError> {
    if !hyperedge_decay_factor.is_finite() // NAN-CHECK-OK
        || hyperedge_decay_factor <= 0.0
        || hyperedge_decay_factor > 1.0
    {
        return Err(contextra_types::ContextraError::InvalidInput(format!(
            "hyperedge_decay_factor must be in (0.0, 1.0] and finite, got {}",
            hyperedge_decay_factor
        )));
    }

    let apprh_params = ApprhParams {
        ppr: params.clone(),
        hyperedge_decay_factor,
        max_iterations: 10_000,
    };

    run_apprh_push(graph, seeds, &apprh_params).map_err(Into::into)
}

impl CsrGraph {
    /// Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH) over the graph.
    ///
    /// # Invariants
    /// Runs exclusively in shadow mode unless feature-flag activated. Bit-identical determinism.
    pub fn apprh_diffusion(
        &self,
        seeds: &[EntityId],
        params: &ApprhParams,
    ) -> Result<AHashMap<EntityId, f32>, ApprhError> {
        apprh_local(self, seeds, params)
    }

    /// Evaluates APPRH diffusion over CsrGraph.
    pub fn evaluate_apprh_diffusion(
        &self,
        seeds: &[EntityId],
        params: &ApprhParams,
    ) -> Result<AHashMap<EntityId, f32>, ApprhError> {
        self.apprh_diffusion(seeds, params)
    }

    /// Computes shadow comparison between Forward-Push PPR and APPRH on CsrGraph.
    pub fn apprh_shadow_compare(
        &self,
        seeds: &[EntityId],
        ppr: &PprParams,
        apprh_params: &ApprhParams,
        top_k: usize,
    ) -> Result<ApprhShadowComparison, ApprhError> {
        shadow_compare_forward_push_vs_apprh(self, seeds, ppr, apprh_params, top_k)
    }

    /// Evaluates APPRH shadow comparison on CsrGraph.
    pub fn evaluate_apprh_shadow(
        &self,
        seeds: &[EntityId],
        ppr: &PprParams,
        apprh_params: &ApprhParams,
        top_k: usize,
    ) -> Result<ApprhShadowComparison, ApprhError> {
        self.apprh_shadow_compare(seeds, ppr, apprh_params, top_k)
    }
}
