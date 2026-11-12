//! Kompatibilitätsbrücke für bestehende Contextra-Workspace-Kerne (SPEC-v2 §4.3).

#[cfg(any(feature = "ports-compat", feature = "workspace"))]
pub mod candidate_stream;
#[cfg(any(feature = "ports-compat", feature = "workspace"))]
pub mod error;
#[cfg(any(feature = "ports-compat", feature = "workspace"))]
pub mod types;
#[cfg(any(feature = "ports-compat", feature = "workspace"))]
pub mod hnsw_vector_index;
#[cfg(all(any(feature = "ports-compat", feature = "workspace"), feature = "experimental-diskann"))]
pub mod diskann_vector_index;
#[cfg(any(feature = "ports-compat", feature = "workspace"))]
pub mod purge;
#[cfg(any(feature = "ports-compat", feature = "workspace"))]
pub mod scanner;
