// FILE-CONTEXT
// ZWECK: Layer-1 Vektor-Such- & Index-Engine mit HNSW, Quantisierung und SIMD.
// INVARIANTEN: Einhaltung der DAG Layer-1 Grenzen (keine Aufwärts-Imports); Zero-Panic Invariante.

//! Contextra Index — HNSW vector index with SIMD distance computation.

#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

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
pub use hnsw::{HnswConfig, HnswIndex, RebuildStatus};
pub use persistence::{HnswHeader, MmapIndex};
pub use quantize::ScalarQuantizer;
#[cfg(feature = "experimental-rabitq")]
pub use quantize_rabitq::RaBitQQuantizer;
