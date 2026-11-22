//! Thresholded Local Hyper-Flow Diffusion (TL-HFD, Spec §21.1).

pub mod diffusion;
pub mod error;
pub mod lovasz;
pub mod params;
pub mod shadow;

pub use diffusion::run_diffusion;
pub use error::TlHfdError;
pub use lovasz::{compute_lovasz_extension, truncate_participants};
pub use params::TlHfdParams;
pub use shadow::{
    shadow_compare_forward_push_vs_tl_hfd, DefaultFlipGate, ShadowComparison,
    ShadowDiscrepancyReport, TlHfdFlipGate, MIN_AGREEMENT_THRESHOLD, MIN_SHADOW_SAMPLES,
};

use crate::csr::CsrGraph;
use crate::path_rag::PathGraph;
use ahash::AHashMap;
use contextra_types::EntityId;

/// Computes Thresholded Local Hyper-Flow Diffusion (TL-HFD) over a generic [`PathGraph`].
///
/// # Invariants
/// Runs exclusively in the shadow path and never as default. Bit-identical determinism.
///
/// # Complexity
/// Time complexity $O(\text{iterations} \cdot k \cdot |\text{Seeds}| \cdot \text{max\_edge\_size})$.
pub fn tl_hfd_local<G: PathGraph>(
    graph: &G,
    seeds: &[EntityId],
    params: &TlHfdParams,
) -> Result<AHashMap<EntityId, f32>, TlHfdError> {
    run_diffusion(graph, seeds, params)
}

impl CsrGraph {
    /// Computes Thresholded Local Hyper-Flow Diffusion (TL-HFD, Spec §21.1) over the graph.
    ///
    /// # Invariants
    /// Runs exclusively in the shadow path and never as default. Bit-identical determinism.
    ///
    /// # Complexity
    /// Time complexity $O(\text{iterations} \cdot k \cdot |\text{Seeds}| \cdot \text{max\_edge\_size})$.
    pub fn thresholded_local_hfd(
        &self,
        seeds: &[EntityId],
        params: &TlHfdParams,
    ) -> Result<AHashMap<EntityId, f32>, TlHfdError> {
        tl_hfd_local(self, seeds, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csr::CsrGraph;
    use contextra_types::Entity;

    #[tokio::test]
    async fn test_thresholded_local_hfd_diagnostic_path() {
        let graph = std::sync::Arc::new(CsrGraph::new());

        let e1 = EntityId::new(10);
        let e2 = EntityId::new(20);

        graph
            .insert_entity_direct(Entity::new(e1, "E1", "Type"))
            .unwrap();
        graph
            .insert_entity_direct(Entity::new(e2, "E2", "Type"))
            .unwrap();

        let he = crate::hyperedge::HyperEdge::new(
            crate::hyperedge::HyperEdgeId::new(1),
            crate::csr::EdgeType::Default,
            vec![
                crate::hyperedge::RoleBinding::new(crate::hyperedge::RoleId::new(1), e1),
                crate::hyperedge::RoleBinding::new(crate::hyperedge::RoleId::new(2), e2),
            ],
            1.0,
        );
        graph.insert_hyperedge_direct(he);

        let params = TlHfdParams::default();
        let res = graph.thresholded_local_hfd(&[e1], &params);
        assert!(
            res.is_ok(),
            "thresholded_local_hfd diagnostic call should succeed"
        );
        let scores = res.unwrap();
        assert!(scores.contains_key(&e1));
    }
}
