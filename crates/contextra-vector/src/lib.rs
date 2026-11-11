// FILE-CONTEXT
// ZWECK: Layer-1 Vektor-Such- & Index-Engine mit HNSW, DiskANN, Quantisierung und SIMD.
// INVARIANTEN: Einhaltung der DAG Layer-1 Grenzen (keine Aufwärts-Imports); Zero-Panic Invariante.
// NICHT-OFFENSICHTLICH: Hardware-Dispatch wählt zur Laufzeit die optimalen SIMD intrinsics (AVX-512 > AVX2 > Skalar).
// HOTSPOTS: lib.rs (Modul-Deklarationen)
// STAND: TS:2026-11-11T02:00:00Z

//! Contextra Index — HNSW vector index with SIMD distance computation.
// INVARIANT: Vector Engine (Triebwerk — Layer 1).
// IMPLEMENTS: VectorIndex Trait (contextra-ports/src/vector.rs)
// KERNKOMPONENTEN: HNSW (Graph-basierte ANN) + CSR Graph (Relationen) + SIMD-Distanz.
// INVARIANTE: HNSW-Graphen liegen exklusiv im RAM. Disk-Storage erfolgt über contextra-store (via LsmStorage).

// ANCHOR[REFACTOR:WP-0.0-STABLESIMD] STATUS:DONE (TS:2026-06-01T00:00:00Z) — Remove nightly portable_simd
// TEST: cargo +stable check -p contextra-index
// DONE: #![feature(portable_simd)] ist entfernt und distance.rs nutzt stabiles Rust.
// INVARIANTE: Zero unsafe code in contextra-vector. SIMD an contextra-simd, mmap an contextra-sys ausgelagert.
#![forbid(unsafe_code)]
// WP-VEC-7: Zero-Panic & Checked Casts
#![deny(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
#![cfg_attr(
    test,
    allow(
        clippy::indexing_slicing,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )
)]

// SECTION Modul-Deklarationen
// ANCHOR[contextra-vector:ROOT-MOD-CASTING]
// Checked Casts für Konvertierungen zwischen Zeigerbreiten (u32, usize).
pub mod casting;

// ANCHOR[contextra-vector:ROOT-MOD-COMPAT]
// Optionale Übergangsbrücke für contextra-ports und contextra-types.
#[cfg(feature = "ports-compat")]
pub mod compat;

// ANCHOR[contextra-vector:ROOT-MOD-CONTRACT]
// Crate-lokale Kern-Verträge (Index, Maintain, DistanceFn, Transform).
pub mod contract;
// ANCHOR[contextra-vector:ROOT-MOD-ERROR]
// Crate-lokales Fehler-Enum VectorError.
pub mod error;
// ANCHOR[contextra-vector:ROOT-MOD-TYPES]
// Crate-lokale Kerntypen (DocId, DistanceMetric, Clock, Rng).
pub mod types;

// ANCHOR[contextra-vector:ROOT-MOD-DISKANN]
// Experimenteller Out-of-Core Graphindex DiskANN.
#[cfg(feature = "experimental-diskann")]
pub mod diskann;
// ANCHOR[contextra-vector:ROOT-MOD-DISTANCE]
// Distanzfunktionen (Kosinus, Euklidisch, Dot) mit SIMD-Fallback.
pub mod distance;
// ANCHOR[contextra-vector:ROOT-MOD-HNSW]
// HNSW Vektorindex-Implementierung.
pub mod hnsw;
// ANCHOR[contextra-vector:ROOT-MOD-PERSISTENCE]
// Format-Header, Node-Serialisierung und Mmap-Unterstützung.
pub mod persistence;
// ANCHOR[contextra-vector:ROOT-MOD-QUANTIZE]
// Skalare Quantisierung (SQ8).
pub mod quantize;

// ANCHOR[contextra-vector:ROOT-MOD-ACORN]
// Prädikat-augmentierte Vektorsuche (ACORN-1).
pub mod acorn;
// ANCHOR[contextra-vector:ROOT-MOD-STREAM]
// Zero-Allocation Streaming von Such-Kandidaten.
// ANCHOR[contextra-vector:ROOT-MOD-POOL]
// Thread-Pool für rechenintensive Distanz- und Rebuild-Operationen.
pub mod compute_pool;
// ANCHOR[contextra-vector:ROOT-MOD-HASHER]
// Fester SipHash-Zustand gegen HashDoS und für deterministisches Verhalten.
pub mod fixed_hasher;
// ANCHOR[contextra-vector:ROOT-MOD-REBUILD]
// Partieller Inkrementeller Rebuild-Pfad (hinter Feature-Gate).
pub mod partial_rebuild;

// ANCHOR[contextra-vector:ROOT-MOD-RABITQ]
// Experimentelle 1-Bit RaBitQ Vektorquantisierung.
#[cfg(feature = "experimental-rabitq")]
pub mod quantize_rabitq;
// !SECTION

// SECTION Öffentliche Re-Exports
// MIGRATION(S-70): neuer Pfad crate::compat::candidate_stream
#[cfg(any(feature = "ports-compat", feature = "workspace"))]
pub use compat::candidate_stream::{VectorCandidateStream, DEFAULT_VECTOR_STREAM_BATCH_SIZE};
pub use compute_pool::ComputePool;
pub use contract::{
    BlockTransform, Clock as ContractClock, DistanceFn, Error as ContractError, ExternalId,
    Index as ContractIndex, Maintain as ContractMaintain, Metric as ContractMetric,
    Result as ContractResult, Rng as ContractRng, ScoredPoint,
};
#[cfg(feature = "experimental-diskann")]
pub use diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex};
#[cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::await_holding_lock
    )
)]
#[allow(deprecated)]
#[cfg(feature = "ports-compat")]
pub use hnsw::{hnsw_purger, hnsw_scanner, HnswPurger, HnswScanner};
pub use hnsw::{HnswConfig, HnswIndex, RebuildStatus};
pub use persistence::{HnswHeader, MmapIndex};
pub use quantize::ScalarQuantizer;
#[cfg(feature = "experimental-rabitq")]
pub use quantize_rabitq::RaBitQQuantizer;
pub use types::{Clock, DistanceMetric, DocId, Maintain, MetricsSink, Rng, ScoredDocument, TxId};
// !SECTION
