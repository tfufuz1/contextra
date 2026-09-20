//! Core trait definitions for MemFuse subsystems.
//!
//! These traits define the abstract interfaces that concrete implementations
//! must fulfill, enabling modularity and testability.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::future::Future;
use std::pin::Pin;

pub use memfuse_types::{error, error_dto, model_fingerprint, schema, tombstone, types};
pub use memfuse_types::*;

/// Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>;

/// Checkpoint and snapshot traits.
pub mod checkpoint;
/// Embedding provider and LLM generation traits.
pub mod embedding;
/// Graph index traits and CSR statistics.
pub mod graph_index;
/// Memory lifecycle, grounding validator, and distance calculator contracts.
pub mod lifecycle;
/// Observability and lifecycle re-exports.
pub mod observability;
/// Key-value storage engine traits.
pub mod storage;
/// Text retrieval and indexing traits (BM25 / Inverted Index).
pub mod text_index;
/// Vector retrieval and HNSW indexing traits.
pub mod vector_index;

pub use checkpoint::*;
pub use embedding::*;
pub use graph_index::*;
pub use lifecycle::*;
pub use observability::*;
pub use storage::*;
pub use text_index::*;
pub use vector_index::*;

impl lifecycle::DistanceCalculator for memfuse_types::DistanceMetric {
    fn compute_f32(&self, a: &[f32], b: &[f32]) -> memfuse_types::Result<f32> {
        self.compute(a, b)
    }

    fn compute_u8(&self, a: &[u8], b: &[u8]) -> memfuse_types::Result<u32> {
        self.compute_u8(a, b)
    }
}

#[cfg(test)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn_storage(_: Option<&dyn StorageEngine>) {}
    fn _assert_dyn_graph(_: Option<&dyn GraphIndex>) {}
    fn _assert_dyn_embedding(_: Option<&dyn TextEmbeddingEngine>) {}
    fn _assert_dyn_response_grounding_validator(_: Option<&dyn ResponseGroundingValidator>) {}

    #[test]
    fn test_dyn_safety_compiles() {
        _assert_dyn_storage(None);
        _assert_dyn_graph(None);
        _assert_dyn_embedding(None);
    }
}
