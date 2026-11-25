//! Core trait definitions for Contextra subsystems.
//!
//! These traits define the abstract interfaces that concrete implementations
//! must fulfill, enabling modularity and testability.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::type_complexity
    )
)]

use std::future::Future;
use std::pin::Pin;

pub use contextra_types::*;
pub use contextra_types::{error, error_dto, model_fingerprint, schema, tombstone, types};

/// Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>;

/// Attention exporter trait and request ID types.
pub mod attention;
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
/// Key-value bridge storage port trait.
pub mod kv_bridge_port;
/// License and activation gate port traits.
pub mod license;
/// Grounding validator traits and context preparer contracts.
pub mod lifecycle;
/// Metrics reporting port trait.
pub mod metrics;
/// Observability and lifecycle re-exports.
pub mod observability;
/// Plugin registry and dependency resolution.
pub mod plugin;
/// QueryRewriter port trait for multi-step retrieval sub-query generation.
pub mod query_rewriter;
/// Reranker port trait for cross-encoder post-retrieval ranking.
pub mod reranker;
/// Random number generator port trait and SplitMix64 implementation.
pub mod rng;
/// Key-value storage engine traits.
pub mod storage;
/// Text retrieval and indexing traits (BM25 / Inverted Index).
pub mod text_index;
/// Vector retrieval and HNSW indexing traits.
pub mod vector_index;

pub use attention::*;
pub use checkpoint::*;
pub use clock::*;
pub use embedding::*;
pub use graph::*;
pub use graph_index::*;
pub use id_gen::*;
pub use kv::*;
pub use kv_bridge_port::*;
pub use license::*;
pub use lifecycle::*;
pub use metrics::*;
pub use observability::*;
pub use plugin::*;
pub use query_rewriter::*;
pub use reranker::*;
pub use rng::*;
pub use storage::*;
pub use text_index::*;
pub use vector_index::*;

#[cfg(test)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn_storage(_: Option<&dyn StorageEngine>) {}
    fn _assert_dyn_metrics_sink(_: Option<&dyn MetricsSink>) {}
    fn _assert_dyn_kv_bridge_storage(_: Option<&dyn KvBridgeStorage>) {}
    fn _assert_dyn_kv_prefix_store(_: Option<&dyn KvPrefixStore>) {}
    fn _assert_dyn_kv_lifecycle_hooks(_: Option<&dyn KvLifecycleHooks>) {}
    fn _assert_dyn_graph_mutation(_: Option<&dyn GraphCollectionMutation>) {}
    fn _assert_dyn_graph(_: Option<&dyn GraphIndex>) {}
    fn _assert_dyn_embedding(_: Option<&dyn TextEmbeddingEngine>) {}
    fn _assert_dyn_response_grounding_validator(_: Option<&dyn ResponseGroundingValidator>) {}
    fn _assert_dyn_clock(_: Option<&dyn Clock>) {}
    fn _assert_dyn_rng(_: Option<&dyn Rng>) {}
    fn _assert_dyn_id_gen(_: Option<&dyn IdGen>) {}
    fn _assert_dyn_license_gate(_: Option<&dyn LicenseGate>) {}
    fn _assert_dyn_attention_exporter(_: Option<&dyn AttentionExporter>) {}
    fn _assert_dyn_reranker(_: Option<&dyn Reranker>) {}
    fn _assert_dyn_query_rewriter(_: Option<&dyn QueryRewriter>) {}

    #[test]
    fn test_dyn_safety_compiles() {
        _assert_dyn_storage(None);
        _assert_dyn_metrics_sink(None);
        _assert_dyn_kv_bridge_storage(None);
        _assert_dyn_kv_prefix_store(None);
        _assert_dyn_kv_lifecycle_hooks(None);
        _assert_dyn_graph_mutation(None);
        _assert_dyn_graph(None);
        _assert_dyn_embedding(None);
        _assert_dyn_clock(None);
        _assert_dyn_rng(None);
        _assert_dyn_id_gen(None);
        _assert_dyn_license_gate(None);
        _assert_dyn_attention_exporter(None);
        _assert_dyn_reranker(None);
        _assert_dyn_query_rewriter(None);
    }
}

// =============================================================================
// ÄNDERUNGSNOTIZ / CHANGELOG (v17 Teil 2.2 Cleanup)
// =============================================================================
// Bereinigung verwaister und überholter Port-Definitionen gemäß Spec v17 Teil 2.2:
// - `DistanceCalculator`: Verwaister Trait, ersetzt durch `DistanceMetric::compute` in `contextra-types`.
// - `MemoryLifecycleManager`: Verwaister Trait ohne Implementierung im Workspace.
// - `StorageRead`: Überholt und konsolidiert im umfassenden `StorageEngine`-Trait in `storage.rs`.
// - `StorageWrite`: Überholt und konsolidiert im umfassenden `StorageEngine`-Trait in `storage.rs`.
// =============================================================================
