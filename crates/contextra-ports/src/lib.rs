//! Core trait definitions for Contextra subsystems.
//!
//! These traits define the abstract interfaces that concrete implementations
//! must fulfill, enabling modularity and testability.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::type_complexity))]

use std::future::Future;
use std::pin::Pin;

pub use contextra_types::*;
pub use contextra_types::{error, error_dto, model_fingerprint, schema, tombstone, types};

/// Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>;

/// Checkpoint and snapshot traits.
pub mod checkpoint;
/// Clock port trait and system time implementation.
pub mod clock;
/// Embedding provider and LLM generation traits.
pub mod embedding;
/// Graph mutation and CSR traversal port traits.
pub mod graph;
/// Graph index traits and CSR statistics.
pub mod graph_index;
/// Unique ID generator port trait and atomic implementation.
pub mod id_gen;
/// Key-value prefix store traits and types.
pub mod kv;
/// Memory lifecycle, grounding validator, and distance calculator contracts.
pub mod lifecycle;
/// Metrics reporting port trait.
pub mod metrics;
/// Observability and lifecycle re-exports.
pub mod observability;
/// Random number generator port trait and SplitMix64 implementation.
pub mod rng;
/// Key-value storage engine traits.
pub mod storage;
/// Text retrieval and indexing traits (BM25 / Inverted Index).
pub mod text_index;
/// Vector retrieval and HNSW indexing traits.
pub mod vector_index;

pub use checkpoint::*;
pub use clock::*;
pub use embedding::*;
pub use graph::*;
pub use graph_index::*;
pub use id_gen::*;
pub use kv::*;
pub use lifecycle::*;
pub use metrics::*;
pub use observability::*;
pub use rng::*;
pub use storage::*;
pub use text_index::*;
pub use vector_index::*;

impl lifecycle::DistanceCalculator for contextra_types::DistanceMetric {
    fn compute_f32(&self, a: &[f32], b: &[f32]) -> contextra_types::Result<f32> {
        self.compute(a, b)
    }

    fn compute_u8(&self, a: &[u8], b: &[u8]) -> contextra_types::Result<u32> {
        self.compute_u8(a, b)
    }
}

#[cfg(test)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn_storage(_: Option<&dyn StorageEngine>) {}
    fn _assert_dyn_storage_read(_: Option<&dyn StorageRead>) {}
    fn _assert_dyn_storage_write(_: Option<&dyn StorageWrite>) {}
    fn _assert_dyn_metrics_sink(_: Option<&dyn MetricsSink>) {}
    fn _assert_dyn_kv_prefix_store(_: Option<&dyn KvPrefixStore>) {}
    fn _assert_dyn_graph_mutation(_: Option<&dyn GraphCollectionMutation>) {}
    fn _assert_dyn_graph(_: Option<&dyn GraphIndex>) {}
    fn _assert_dyn_embedding(_: Option<&dyn TextEmbeddingEngine>) {}
    fn _assert_dyn_response_grounding_validator(_: Option<&dyn ResponseGroundingValidator>) {}
    fn _assert_dyn_clock(_: Option<&dyn Clock>) {}
    fn _assert_dyn_rng(_: Option<&dyn Rng>) {}
    fn _assert_dyn_id_gen(_: Option<&dyn IdGen>) {}

    #[test]
    fn test_dyn_safety_compiles() {
        _assert_dyn_storage(None);
        _assert_dyn_storage_read(None);
        _assert_dyn_storage_write(None);
        _assert_dyn_metrics_sink(None);
        _assert_dyn_kv_prefix_store(None);
        _assert_dyn_graph_mutation(None);
        _assert_dyn_graph(None);
        _assert_dyn_embedding(None);
        _assert_dyn_clock(None);
        _assert_dyn_rng(None);
        _assert_dyn_id_gen(None);
    }
}
