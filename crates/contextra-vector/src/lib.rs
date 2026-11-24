// FILE-CONTEXT
// ZWECK: Crate-Fassade für den autarken contextra-vector Vektorindex.
// INVARIANTEN: Zero-Unsafe-Invariante (#![forbid(unsafe_code)]), Zero-Panic Invariante.
// NICHT-OFFENSICHTLICH: Re-Exports binden kanonische Kern-Typen und Verträge für Endanwender und Tests ein.
// HOTSPOTS: lib.rs (Modul-Deklarationen und Re-Exports)
// STAND: 2026-11-11

//! Contextra Vector — HNSW vector index with SIMD distance computation.

#![forbid(unsafe_code)]
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
#[cfg(any(feature = "ports-compat", feature = "workspace"))]
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
