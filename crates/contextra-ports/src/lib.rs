//! Core trait definitions for Contextra subsystems.
//!
//! These traits define the abstract interfaces that concrete implementations
//! must fulfill, enabling modularity and testability.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(deprecated)]
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

/// Re-export canonical `contextra-types` submodules explicitly per Contract M3 / AR-10.
pub use contextra_types::{
    error, ids, model_fingerprint, refs, tenant_scope, types, witness, CollectionRef,
    ConfigFingerprint, ContextChunk, ContextWindow, ContextraError, DocId, DocKey, EntityId,
    ErasureId, FileId, GateId, ModelFingerprint, OperatorRef, QueryRewriteOutput, RequestContext,
    RerankResult, ResourceBudget, ResourceTracker, Result, RetrievalStrategy, SagaRecord,
    ScanMethod, ScoredEntry, Seq, StoreId, SubjectRef, TenantId, TenantRef, TenantScopeViolation,
    TenantScoped, Timestamp, TokenBudget, TxId, WorkflowState, MAX_REF_LEN,
};

/// Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>;

/// Checkpoint and snapshot traits.
pub mod checkpoint;
/// Clock port trait and system time implementation.
pub mod clock;
/// Trait conformance assertions and wiring verification matrix.
pub mod conformance;
/// Embedding provider and LLM generation traits.
pub mod embedding;
/// Graph mutation and CSR traversal port traits.
pub mod graph;
/// Graph index traits and CSR statistics.
pub mod graph_index;
/// Unique ID generator port trait and atomic implementation.
pub mod id_gen;
/// Key-value prefix store traits and types.
/// Key-value bridge storage port trait.
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
/// Storage layer scanners and physical purgers (Vertrag V1, SPEC §12.7.4).
pub mod scanner;
/// Key-value storage engine traits.
pub mod storage;
/// Text retrieval and indexing traits (BM25 / Inverted Index).
pub mod text_index;
/// Vector retrieval and HNSW indexing traits.
pub mod vector_index;
/// Virtual File System (VFS) port interface and contracts.
pub mod vfs;

/// Environment abstractions (Clock, Rng, IdGen, MetricsSink) with sub-prelude.
pub mod env;
/// Filesystem abstractions (Vfs, VfsFile) with sub-prelude.
pub mod fs;
/// Indexing subsystem abstractions (VectorIndex, TextIndex, GraphIndex) with sub-prelude.
pub mod index;
/// Inference subsystem abstractions (Embedding, Reranker, QueryRewriter) with sub-prelude.
pub mod inference;
/// Policy subsystem abstractions (LicenseGate, EgressPolicy) with sub-prelude.
pub mod policy;

pub use checkpoint::*;
pub use clock::*;
pub use conformance::*;
pub use embedding::*;
pub use graph::*;
pub use graph_index::*;
pub use id_gen::*;
pub use license::*;
pub use lifecycle::*;
pub use metrics::*;
pub use observability::*;
pub use plugin::*;
pub use query_rewriter::*;
pub use reranker::*;
pub use rng::*;
pub use scanner::{
    LayerScanner, LayerWitness, PurgeStats, Purger, ScanSession, ScanTarget, ScanTechnique,
    WitnessLayer,
};
pub use storage::*;
pub use text_index::*;
pub use vector_index::*;
pub use vfs::*;

/// Generic contract tests for port implementers.
#[cfg(any(feature = "contract-tests", feature = "conformance"))]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::type_complexity,
    clippy::io_other_error
)]
pub mod contract_tests;

pub use metrics::{
    HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, NoopMetricsSink,
};

#[cfg(test)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn_storage(_: Option<&dyn StorageEngine>) {}
    fn _assert_dyn_metrics_sink(_: Option<&dyn MetricsSink>) {}
    fn _assert_dyn_graph_mutation(_: Option<&dyn GraphCollectionMutation>) {}
    fn _assert_dyn_graph(_: Option<&dyn GraphIndex>) {}
    fn _assert_dyn_embedding(_: Option<&dyn TextEmbeddingEngine>) {}
    fn _assert_dyn_response_grounding_validator(_: Option<&dyn ResponseGroundingValidator>) {}
    fn _assert_dyn_clock(_: Option<&dyn Clock>) {}
    fn _assert_dyn_rng(_: Option<&dyn Rng>) {}
    fn _assert_dyn_id_gen(_: Option<&dyn IdGen>) {}
    fn _assert_dyn_license_gate(_: Option<&dyn LicenseGate>) {}
    fn _assert_dyn_reranker(_: Option<&dyn Reranker>) {}
    fn _assert_dyn_query_rewriter(_: Option<&dyn QueryRewriter>) {}
    fn _assert_dyn_layer_scanner(_: Option<&dyn LayerScanner>) {}
    fn _assert_dyn_purger(_: Option<&dyn Purger>) {}
    fn _assert_dyn_read_gate(_: Option<&dyn ReadGate>) {}
    fn _assert_dyn_erasure_layer(_: Option<&dyn ErasureLayer>) {}
    fn _assert_dyn_erasure_kernel(_: Option<&dyn ErasureKernel>) {}
    fn _assert_dyn_ledger_sink(_: Option<&dyn LedgerSink>) {}
    fn _assert_dyn_ledger_anchor(_: Option<&dyn LedgerAnchor>) {}
    fn _assert_dyn_io_budget(_: Option<&dyn IoBudget>) {}
    fn _assert_dyn_progress_sink(_: Option<&dyn ProgressSink>) {}
    fn _assert_dyn_cancel(_: Option<&dyn Cancel>) {}
    fn _assert_dyn_privileged_read(_: Option<&dyn PrivilegedRead>) {}
    fn _assert_dyn_erasure_observer(_: Option<&dyn ErasureObserver>) {}
    fn _assert_dyn_vfs(_: Option<&dyn Vfs>) {}
    fn _assert_dyn_egress_policy(_: Option<&dyn crate::policy::EgressPolicy>) {}

    #[test]
    fn test_ports_reexport_type_id_equality() {
        use std::any::TypeId;
        assert_eq!(
            TypeId::of::<crate::Seq>(),
            TypeId::of::<contextra_types::Seq>()
        );
        assert_eq!(
            TypeId::of::<crate::gate::Seq>(),
            TypeId::of::<contextra_types::Seq>()
        );
        assert_eq!(
            TypeId::of::<crate::Timestamp>(),
            TypeId::of::<contextra_types::Timestamp>()
        );
        assert_eq!(
            TypeId::of::<crate::ledger::Timestamp>(),
            TypeId::of::<contextra_types::Timestamp>()
        );
        assert_eq!(
            TypeId::of::<crate::observer::Timestamp>(),
            TypeId::of::<contextra_types::Timestamp>()
        );
        assert_eq!(
            TypeId::of::<crate::TargetKind>(),
            TypeId::of::<contextra_types::TargetKind>()
        );
        assert_eq!(
            TypeId::of::<crate::observer::TargetKind>(),
            TypeId::of::<contextra_types::TargetKind>()
        );
        assert_eq!(
            TypeId::of::<crate::erasure::EraseTarget>(),
            TypeId::of::<contextra_types::EraseTarget>()
        );
        assert_eq!(
            TypeId::of::<crate::erasure::ErasureError>(),
            TypeId::of::<contextra_types::ErasureError>()
        );
    }

    #[test]
    fn test_dyn_safety_compiles() {
        _assert_dyn_storage(None);
        _assert_dyn_metrics_sink(None);
        _assert_dyn_graph_mutation(None);
        _assert_dyn_graph(None);
        _assert_dyn_embedding(None);
        _assert_dyn_clock(None);
        _assert_dyn_rng(None);
        _assert_dyn_id_gen(None);
        _assert_dyn_license_gate(None);
        _assert_dyn_reranker(None);
        _assert_dyn_query_rewriter(None);
        _assert_dyn_layer_scanner(None);
        _assert_dyn_purger(None);
        _assert_dyn_read_gate(None);
        _assert_dyn_erasure_layer(None);
        _assert_dyn_erasure_kernel(None);
        _assert_dyn_ledger_sink(None);
        _assert_dyn_ledger_anchor(None);
        _assert_dyn_io_budget(None);
        _assert_dyn_progress_sink(None);
        _assert_dyn_cancel(None);
        _assert_dyn_privileged_read(None);
        _assert_dyn_erasure_observer(None);
        _assert_dyn_vfs(None);
        _assert_dyn_egress_policy(None);
    }
}

pub mod erasure;
pub mod gate;
pub mod ledger;
pub mod observer;
pub mod report_signer;
#[cfg(feature = "stage2")]
pub mod seal;

pub use erasure::{
    compute_plan_hash, AcceptReceipt, Cancel, DocScope, EraseRequest, ErasureKernel, ErasureLayer,
    GateReceipt, IoBudget, IoClass, JobView, KeyPattern, KeyScope, LayerError, PressureHint,
    PrivilegedRead, ProgressSink, PurgeCx, PurgeOutcome, ResolvedScope, SealReceipt, Verdict,
};
pub use gate::{DiscardRule, GateDecision, RangeGate, ReadGate};
pub use ledger::{
    LedgerAnchor, LedgerDraft, LedgerEntry, LedgerHead, LedgerSink, LedgerVerification,
};
pub use observer::{ErasureEvent, ErasureObserver, ObservedPhase, ObserverError, TargetKind};
pub use report_signer::ReportSigner;
#[cfg(feature = "stage2")]
pub use seal::{Envelope, RecordSealer, SealCtx};
