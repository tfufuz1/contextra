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
// ANCHOR[contextra-graph:SRC-APPRH-MOD-001]
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
// ANCHOR[contextra-graph:SRC-APPRH-MOD-002]
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
// ANCHOR[contextra-graph:SRC-APPRH-MOD-003]
    pub fn apprh_diffusion(
        &self,
        seeds: &[EntityId],
        params: &ApprhParams,
    ) -> Result<AHashMap<EntityId, f32>, ApprhError> {
        apprh_local(self, seeds, params)
    }

    /// Computes shadow comparison between Forward-Push PPR and APPRH on CsrGraph.
// ANCHOR[contextra-graph:SRC-APPRH-MOD-004]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csr::CsrGraph;
    use contextra_types::Entity;

// ANCHOR[contextra-graph:SRC-APPRH-MOD-005]
    #[tokio::test]
    async fn test_apprh_diffusion_and_shadow_compare_diagnostic_paths() {
        let graph = std::sync::Arc::new(CsrGraph::new());
        let e1 = EntityId::new(1);
        let e2 = EntityId::new(2);

        graph
            .insert_entity_direct(Entity::new(e1, "Node1", "Type"))
            .unwrap();
        graph
            .insert_entity_direct(Entity::new(e2, "Node2", "Type"))
            .unwrap();
        graph.insert_edge_direct(e1, e2, 1.0).await.unwrap();

        let apprh_params = ApprhParams {
            ppr: PprParams::default(),
            hyperedge_decay_factor: 0.8,
            max_iterations: 100,
        };

        // Test graph.apprh_diffusion
        let res = graph.apprh_diffusion(&[e1], &apprh_params);
        assert!(
            res.is_ok(),
            "apprh_diffusion diagnostic call should succeed"
        );
        let scores = res.unwrap();
        assert!(scores.contains_key(&e1));

        // Test graph.apprh_shadow_compare
        let ppr_params = PprParams::default();
        let shadow_res = graph.apprh_shadow_compare(&[e1], &ppr_params, &apprh_params, 5);
        assert!(
            shadow_res.is_ok(),
            "apprh_shadow_compare diagnostic call should succeed"
        );
        let comparison = shadow_res.unwrap();
        assert!(comparison.top_k_overlap >= 0.0 && comparison.top_k_overlap <= 1.0);
    }
}
