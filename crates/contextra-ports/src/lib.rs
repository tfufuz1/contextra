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

pub use contextra_types::{
    error, error_dto, model_fingerprint, schema, tombstone, types, ConfigFingerprint,
    ContextChunk, ContextraError, DistanceMetric, DocId, ModelFingerprint, ResourceBudget,
    ResourceTracker, Result, ScoredDocument, TenantContext, TenantId, TxId,
};
pub use contextra_types::filter::FilterExpr;

/// Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>;

/// Clock port trait and system time implementation.
pub mod clock;
/// Embedding provider and LLM generation traits.
// Begründung: Re-Export von veralteten Ports für Abwärtskompatibilität externer Konsumenten bis zur Fertigstellung von J14 (SPEC §1.4 / D-24)
#[allow(deprecated)]
pub mod embedding;
/// Unique ID generator port trait and atomic implementation.
pub mod id_gen;
/// Metrics reporting port trait.
pub mod metrics;
/// QueryRewriter port trait for multi-step retrieval sub-query generation.
// Begründung: Re-Export von veralteten Ports für Abwärtskompatibilität externer Konsumenten bis zur Fertigstellung von J14 (SPEC §1.4 / D-24)
#[allow(deprecated)]
pub mod query_rewriter;
/// Reranker port trait for cross-encoder post-retrieval ranking.
// Begründung: Re-Export von veralteten Ports für Abwärtskompatibilität externer Konsumenten bis zur Fertigstellung von J14 (SPEC §1.4 / D-24)
#[allow(deprecated)]
pub mod reranker;
/// Random number generator port trait and SplitMix64 implementation.
pub mod rng;
/// Key-value storage engine traits.
pub mod storage;
/// Text retrieval and indexing traits (BM25 / Inverted Index).
pub mod text_index;
/// Vector retrieval and HNSW indexing traits.
pub mod vector_index;
/// Virtual File System (VFS) port interface and contracts.
pub mod vfs;
/// Erasure layer contracts.
pub mod erasure;
/// ReadGate interface and types.
pub mod gate;
/// Key management, RecordSealer, and FileKeyring.
pub mod keys;
/// Ledger sink contracts.
pub mod ledger;
/// Synchronization barriers for commit, compaction, and purge mutual exclusion.
pub mod barrier;
/// Memory locking and dumping protection ports for vector arenas.
pub mod memory;

pub use barrier::*;
pub use clock::*;
pub use erasure::*;
pub use gate::*;
pub use keys::*;
pub use ledger::*;
pub use memory::*;
// Begründung: Re-Export von veralteten Ports für Abwärtskompatibilität externer Konsumenten bis zur Fertigstellung von J14 (SPEC §1.4 / D-24)
#[allow(deprecated)]
pub use embedding::*;
pub use id_gen::*;
pub use metrics::*;
// Begründung: Re-Export von veralteten Ports für Abwärtskompatibilität externer Konsumenten bis zur Fertigstellung von J14 (SPEC §1.4 / D-24)
#[allow(deprecated)]
pub use query_rewriter::*;
// Begründung: Re-Export von veralteten Ports für Abwärtskompatibilität externer Konsumenten bis zur Fertigstellung von J14 (SPEC §1.4 / D-24)
#[allow(deprecated)]
pub use reranker::*;
pub use rng::*;
pub use storage::*;
pub use text_index::*;
pub use vector_index::*;
pub use vfs::*;

pub use metrics::{
    HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, NoopMetricsSink,
};

#[cfg(test)]
// Begründung: Dynamische Sicherheitsprüfung umfasst auch veraltete Ports
#[allow(deprecated)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn_storage(_: Option<&dyn StorageEngine>) {}
    fn _assert_dyn_metrics_sink(_: Option<&dyn MetricsSink>) {}
    fn _assert_dyn_embedding(_: Option<&dyn TextEmbeddingEngine>) {}
    fn _assert_dyn_clock(_: Option<&dyn Clock>) {}
    fn _assert_dyn_rng(_: Option<&dyn Rng>) {}
    fn _assert_dyn_id_gen(_: Option<&dyn IdGen>) {}
    fn _assert_dyn_reranker(_: Option<&dyn Reranker>) {}
    fn _assert_dyn_query_rewriter(_: Option<&dyn QueryRewriter>) {}
    fn _assert_dyn_vfs(_: Option<&dyn Vfs>) {}
    fn _assert_dyn_commit_barrier(_: Option<&dyn CommitBarrier>) {}
    fn _assert_dyn_purge_barrier(_: Option<&dyn PurgeBarrier>) {}
    fn _assert_dyn_arena_guard(_: Option<&dyn ArenaGuard>) {}

    #[test]
    fn test_dyn_safety_compiles() {
        _assert_dyn_storage(None);
        _assert_dyn_metrics_sink(None);
        _assert_dyn_embedding(None);
        _assert_dyn_clock(None);
        _assert_dyn_rng(None);
        _assert_dyn_id_gen(None);
        _assert_dyn_reranker(None);
        _assert_dyn_query_rewriter(None);
        _assert_dyn_vfs(None);
        _assert_dyn_commit_barrier(None);
        _assert_dyn_purge_barrier(None);
        _assert_dyn_arena_guard(None);
    }
}
