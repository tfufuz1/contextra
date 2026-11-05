//! Schnittstellen (Traits) zwischen den Schichten: Speicher, Index, Dateisystem, Zeit, Zufall, Löschschicht.
//!
//! Ring 0 Komponente laut SPEC 5.1a, 5.2, 5.3 und 8.2.
//! Erlaubte Abhängigkeiten laut SPEC 5.3: types.
//!
//! Diese Traits definieren die abstrakten Schnittstellen (Ports) zwischen
//! den Schichten zur Realisierung einer hexagonalen Architektur.

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

// AR-10: Nur Typen re-exportieren, die in Trait-Signaturen vorkommen
pub use contextra_types::{
    ContextraError, DocId, ResourceBudget, ResourceTracker, Result, ScoredDocument, TxId,
};

/// Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>;

/// Synchronization barriers for commit, compaction, and purge mutual exclusion.
pub mod barrier;
/// Clock port trait and system time implementation.
pub mod clock;
/// Erasure layer contracts.
pub mod erasure;
/// ReadGate interface and types.
pub mod gate;
/// Unique ID generator port trait and atomic implementation.
pub mod id_gen;
/// Key management, RecordSealer, and FileKeyring.
pub mod keys;
/// Ledger sink contracts.
pub mod ledger;
/// Memory locking and dumping protection ports for vector arenas.
pub mod memory;
/// Metrics reporting port trait.
pub mod metrics;
/// Erase-Observer port trait and event representations (SPEC D-118).
pub mod observer;
/// Pre-gate execution hook port trait (SPEC D-115, D-117).
pub mod pre_gate;
/// Audit report signer port trait (SPEC D-130).
pub mod report_signer;
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

// AR-10: Einzelne Re-Exporte statt Glob-Re-Exporte
pub use barrier::{
    BarrierExclusive, BarrierShared, CommitBarrier, CompactionPermit, PurgeBarrier, PurgePermit,
};
pub use clock::{Clock, SystemClock};
pub use erasure::{
    Armed, EraseTarget, ErasureLayer, KeyCoverage, LayerId, LayerScope, LayerWitness, PurgeCx,
    PurgeOutcome, Scanner,
};
pub use gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};
pub use id_gen::{IdGen, SequentialIdGen};
pub use keys::{Envelope, FileId, FileKey, FileKeyring, FileKind, RecordSealer, SealCtx};
pub use ledger::{LedgerEntry, LedgerSink, RangeSet, Timestamp};
pub use memory::{ArenaGuard, MemoryLockGuard};
pub use metrics::{
    HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricName, MetricSnapshot, Metrics,
    MetricsSink, MetricsSinkAdapter, NoopMetricsSink,
};
pub use observer::{ErasureEvent, ErasureObserver, ObservedPhase, ObserverError, TargetKind};
pub use pre_gate::{PreGate, PreGateIntent, PreGateOutcome};
pub use report_signer::{ReportSignature, ReportSigner, SignerError, SignerId};
pub use rng::{Rng, SeededRng};
pub use storage::{StorageEngine, StorageStats, MAX_SCAN_MERGE_ACCUMULATOR};
pub use text_index::{TextIndex, TextIndexStats};
pub use vector_index::{VectorIndex, VectorIndexStats};
pub use vfs::{VerifiedVia, Vfs, VfsFile, VfsOpenOptions};

#[cfg(test)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn_storage(_: Option<&dyn StorageEngine>) {}
    fn _assert_dyn_metrics_sink(_: Option<&dyn MetricsSink>) {}
    fn _assert_dyn_clock(_: Option<&dyn Clock>) {}
    fn _assert_dyn_rng(_: Option<&dyn Rng>) {}
    fn _assert_dyn_id_gen(_: Option<&dyn IdGen>) {}
    fn _assert_dyn_vfs(_: Option<&dyn Vfs>) {}
    fn _assert_dyn_commit_barrier(_: Option<&dyn CommitBarrier>) {}
    fn _assert_dyn_purge_barrier(_: Option<&dyn PurgeBarrier>) {}
    fn _assert_dyn_arena_guard(_: Option<&dyn ArenaGuard>) {}
    fn _assert_dyn_erasure_observer(_: Option<&dyn ErasureObserver>) {}
    fn _assert_dyn_report_signer(_: Option<&dyn ReportSigner>) {}
    fn _assert_dyn_pre_gate(_: Option<&dyn PreGate>) {}
    fn _assert_dyn_erasure_layer(_: Option<&dyn ErasureLayer>) {}
    fn _assert_dyn_read_gate(_: Option<&dyn ReadGate>) {}
    fn _assert_dyn_record_sealer(_: Option<&dyn RecordSealer>) {}
    fn _assert_dyn_file_keyring(_: Option<&dyn FileKeyring>) {}
    fn _assert_dyn_ledger_sink(_: Option<&dyn LedgerSink>) {}

    /// SPEC: T-ARCH-6
    #[test]
    fn test_dyn_safety_compiles() {
        _assert_dyn_storage(None);
        _assert_dyn_metrics_sink(None);
        _assert_dyn_clock(None);
        _assert_dyn_rng(None);
        _assert_dyn_id_gen(None);
        _assert_dyn_vfs(None);
        _assert_dyn_commit_barrier(None);
        _assert_dyn_purge_barrier(None);
        _assert_dyn_arena_guard(None);
        _assert_dyn_erasure_observer(None);
        _assert_dyn_report_signer(None);
        _assert_dyn_pre_gate(None);
        _assert_dyn_erasure_layer(None);
        _assert_dyn_read_gate(None);
        _assert_dyn_record_sealer(None);
        _assert_dyn_file_keyring(None);
        _assert_dyn_ledger_sink(None);
    }

    /// SPEC: T-ARCH-3 / Rule 3
    #[test]
    fn test_ports_forbid_unsafe() {
        // Enforces crate-level unsafe code prohibition per SPEC §3.2 and Rule 3.
        let lib_src = include_str!("lib.rs");
        assert!(
            lib_src.contains("#![forbid(unsafe_code)]"),
            "contextra-ports must enforce #![forbid(unsafe_code)]"
        );
    }
}
