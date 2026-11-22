//! Core trait definitions for Contextra subsystems.
//!
//! These traits define the abstract interfaces that concrete implementations
//! must fulfill, enabling modularity and testability.
//!
//! # Features
//!
//! - `persistence` (ohne weitere Abhängigkeiten): nur das Modul
//!   [`persistence`], der Port-Katalog des Speicherkerns (SPEC §4). Es
//!   importiert ausschließlich `std`.
//! - `full` (Standard): alle übrigen Ports und Abhängigkeiten. Bestehende
//!   Konsumenten beziehen es über den Standard und bleiben unverändert.

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

#[cfg(feature = "full")]
pub use contextra_types::*;
#[cfg(feature = "full")]
pub use contextra_types::{error, error_dto, model_fingerprint, schema, tombstone, types};

/// Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.
#[cfg(feature = "full")]
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>;

/// Persistenz-Ports für contextra-storage (SPEC §4). Kein Glob-Re-Export.
pub mod persistence;

/// Attention exporter trait and request ID types.
#[cfg(feature = "full")]
pub mod attention;
/// Checkpoint and snapshot traits.
#[cfg(feature = "full")]
pub mod checkpoint;
/// Clock port trait and system time implementation.
#[cfg(feature = "full")]
pub mod clock;
/// Embedding provider and LLM generation traits.
#[cfg(feature = "full")]
pub mod embedding;
/// Graph mutation and CSR traversal port traits.
#[cfg(feature = "full")]
pub mod graph;
/// Graph index traits and CSR statistics.
#[cfg(feature = "full")]
pub mod graph_index;
/// Unique ID generator port trait and atomic implementation.
#[cfg(feature = "full")]
pub mod id_gen;
/// Key-value prefix store traits and types.
#[cfg(feature = "full")]
pub mod kv;
/// Key-value bridge storage port trait.
#[cfg(feature = "full")]
pub mod kv_bridge_port;
/// License and activation gate port traits.
#[cfg(feature = "full")]
pub mod license;
/// Grounding validator traits and context preparer contracts.
#[cfg(feature = "full")]
pub mod lifecycle;
/// Metrics reporting port trait.
#[cfg(feature = "full")]
pub mod metrics;
/// Observability and lifecycle re-exports.
#[cfg(feature = "full")]
pub mod observability;
/// Plugin registry and dependency resolution.
#[cfg(feature = "full")]
pub mod plugin;
/// QueryRewriter port trait for multi-step retrieval sub-query generation.
#[cfg(feature = "full")]
pub mod query_rewriter;
/// Reranker port trait for cross-encoder post-retrieval ranking.
#[cfg(feature = "full")]
pub mod reranker;
/// Random number generator port trait and SplitMix64 implementation.
#[cfg(feature = "full")]
pub mod rng;
/// Storage layer scanners and purgers (Vertrag V1, SPEC §12.7.4).
#[cfg(feature = "full")]
pub mod scanner;
/// Key-value storage engine traits.
#[cfg(feature = "full")]
pub mod storage;
/// Text retrieval and indexing traits (BM25 / Inverted Index).
#[cfg(feature = "full")]
pub mod text_index;
/// Vector retrieval and HNSW indexing traits.
#[cfg(feature = "full")]
pub mod vector_index;
/// Virtual File System (VFS) port interface and contracts.
#[cfg(feature = "full")]
pub mod vfs;

#[cfg(feature = "full")]
pub use attention::*;
#[cfg(feature = "full")]
pub use checkpoint::*;
#[cfg(feature = "full")]
pub use clock::*;
#[cfg(feature = "full")]
pub use embedding::*;
#[cfg(feature = "full")]
pub use graph::*;
#[cfg(feature = "full")]
pub use graph_index::*;
#[cfg(feature = "full")]
pub use id_gen::*;
#[cfg(feature = "full")]
pub use kv::*;
#[cfg(feature = "full")]
pub use kv_bridge_port::*;
#[cfg(feature = "full")]
pub use license::*;
#[cfg(feature = "full")]
pub use lifecycle::*;
#[cfg(feature = "full")]
pub use metrics::*;
#[cfg(feature = "full")]
pub use observability::*;
#[cfg(feature = "full")]
pub use plugin::*;
#[cfg(feature = "full")]
pub use query_rewriter::*;
#[cfg(feature = "full")]
pub use reranker::*;
#[cfg(feature = "full")]
pub use rng::*;
#[cfg(feature = "full")]
pub use scanner::{
    LayerScanner, LayerWitness, PurgeStats, Purger, ScanSession, ScanTarget, ScanTechnique,
    WitnessLayer,
};
#[cfg(feature = "full")]
pub use storage::*;
#[cfg(feature = "full")]
pub use text_index::*;
#[cfg(feature = "full")]
pub use vector_index::*;
#[cfg(feature = "full")]
pub use vfs::*;

/// Generic contract tests for port implementers.
#[cfg(feature = "contract-tests")]
pub mod contract_tests;

#[cfg(feature = "full")]
pub use metrics::{
    HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, NoopMetricsSink,
};

#[cfg(all(test, feature = "full"))]
mod dyn_safety {
    use super::*;

    fn _assert_dyn_storage(_: Option<&dyn StorageEngine>) {}
    fn _assert_dyn_metrics_sink(_: Option<&dyn MetricsSink>) {}
    fn _assert_dyn_kv_bridge_storage(_: Option<&dyn KvBridgeStorage>) {}
    fn _assert_dyn_kv_prefix_store(_: Option<&dyn KvPrefixStore>) {}
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
    fn _assert_dyn_layer_scanner(_: Option<&dyn LayerScanner>) {}
    fn _assert_dyn_purger(_: Option<&dyn Purger>) {}
    fn _assert_dyn_vfs(_: Option<&dyn Vfs>) {}

    #[test]
    fn test_dyn_safety_compiles() {
        _assert_dyn_storage(None);
        _assert_dyn_metrics_sink(None);
        _assert_dyn_kv_bridge_storage(None);
        _assert_dyn_kv_prefix_store(None);
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
        _assert_dyn_layer_scanner(None);
        _assert_dyn_purger(None);
        _assert_dyn_vfs(None);
    }
}
