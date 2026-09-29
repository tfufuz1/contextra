//! Averaging-based Personalized PageRank for Hypergraphs (APPRH).

pub mod diffusion;
pub mod error;
pub mod gate_monitor;
pub mod params;
pub mod shadow;

pub use diffusion::run_apprh_push;
pub use error::ApprhError;
pub use gate_monitor::{
    ApprhGateMonitor, ApprhGateMonitorConfig, ApprhGateMonitorSnapshot, ApprhObservationRecord,
};
pub use params::ApprhParams;
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
}
