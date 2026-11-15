// Crate-Fassade für den autarken acta-vec Vektorindex.
// Invarianten: Zero-Unsafe-Invariante (#![forbid(unsafe_code)]), Zero-Panic Invariante.
// NICHT-OFFENSICHTLICH: Re-Exports binden kanonische Kern-Typen und Verträge für Endanwender und Tests ein.
// HOTSPOTS: lib.rs (Modul-Deklarationen und Re-Exports)

//! Acta Vector — HNSW vector index with SIMD distance computation.

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

// Checked Casts für Konvertierungen zwischen Zeigerbreiten (u32, usize).
pub mod casting;

// Optionale Übergangsbrücke für acta-base und acta-base.
#[cfg(any(feature = "ports-compat", feature = "workspace"))]
pub mod compat;

// Crate-lokale Kern-Verträge (Index, Maintain, DistanceFn, Transform).
pub mod contract;

// Crate-lokales Fehler-Enum VectorError.
pub mod error;

// Crate-lokale Kerntypen (DocId, DistanceMetric, Clock, Rng).
pub mod types;

// Experimenteller Out-of-Core Graphindex DiskANN.
#[cfg(feature = "experimental-diskann")]
pub mod diskann;

// Distanzfunktionen (Kosinus, Euklidisch, Dot) mit SIMD-Fallback.
pub mod distance;

// HNSW Vektorindex-Implementierung.
pub mod hnsw;

// Format-Header, Node-Serialisierung und Mmap-Unterstützung.
pub mod persistence;

// Skalare Quantisierung (SQ8).
pub mod quantize;

// Prädikat-augmentierte Vektorsuche (ACORN-1).
pub mod acorn;

// Thread-Pool für rechenintensive Distanz- und Rebuild-Operationen.
pub mod compute_pool;

// Fester SipHash-Zustand gegen HashDoS und für deterministisches Verhalten.
pub mod fixed_hasher;

// Partieller Inkrementeller Rebuild-Pfad (hinter Feature-Gate).
pub mod partial_rebuild;

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
