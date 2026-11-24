//! Indexing subsystem port contracts (VectorIndex, TextIndex, GraphIndex).

pub use crate::graph::{GraphCollectionMutation, GraphMutationError};
pub use crate::graph_index::{CommunityResolver, GraphIndex, GraphIndexStats};
pub use crate::text_index::{SegmentSynthesizer, TextIndex, TextIndexStats};
pub use crate::vector_index::{HybridSearchProvider, VectorIndex, VectorIndexStats};

/// Prelude for indexing ports.
pub mod prelude {
    pub use super::{GraphCollectionMutation, GraphIndex, TextIndex, VectorIndex};
}
