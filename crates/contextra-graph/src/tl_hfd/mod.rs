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
pub use shadow::{shadow_compare_forward_push_vs_tl_hfd, ShadowComparison};

use crate::csr::CsrGraph;
use crate::path_rag::PathGraph;
use ahash::AHashMap;
use contextra_core::EntityId;

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
