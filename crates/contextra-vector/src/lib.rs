// FILE-CONTEXT
// ZWECK: Layer-1 Vektor-Such- & Index-Engine mit HNSW, DiskANN, Quantisierung und SIMD.
// INVARIANTEN: Einhaltung der DAG Layer-1 Grenzen (keine Aufwärts-Imports); Zero-Panic Invariante.
// NICHT-OFFENSICHTLICH: Hardware-Dispatch wählt zur Laufzeit die optimalen SIMD intrinsics (AVX-512 > AVX2 > Skalar).
// HOTSPOTS: lib.rs (Modul-Deklarationen)
// STAND: TS:2026-09-04T11:41:54Z (SESSION: 9c384478)

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
// ADR-077: Vector index numerical conversions, array indexing, and internal collections.
#![allow(
    clippy::too_many_arguments,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::disallowed_types,
    clippy::collapsible_if,
    clippy::wrong_self_convention
)]

pub mod distance;
pub mod hnsw;
pub mod persistence;
pub mod quantize;

pub mod acorn;
pub mod candidate_stream;
pub mod compute_pool;
pub mod partial_rebuild;

#[cfg(feature = "experimental-rabitq")]
pub mod quantize_rabitq;

pub use candidate_stream::{VectorCandidateStream, DEFAULT_VECTOR_STREAM_BATCH_SIZE};
pub use compute_pool::ComputePool;
pub use hnsw::{
    hnsw_purger, hnsw_scanner, HnswConfig, HnswIndex, HnswPurger, HnswScanner, RebuildStatus,
};
pub use persistence::{HnswHeader, MmapIndex};
pub use quantize::ScalarQuantizer;
#[cfg(feature = "experimental-rabitq")]
pub use quantize_rabitq::RaBitQQuantizer;
