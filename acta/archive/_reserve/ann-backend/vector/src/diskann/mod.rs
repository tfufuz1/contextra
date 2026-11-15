// DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (.3).
// Invarianten: Lock-Hierarchie: header -> mmap -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Mmap für Vektor- & Graphlesezugriffe, ausgelagert an acta-sys::mmap_readonly.

pub(crate) mod build;
pub(crate) mod config;
pub(crate) mod filtered;
pub(crate) mod format;
pub(crate) mod hnsw_bridge;
pub(crate) mod io;
pub(crate) mod integrity;
pub(crate) mod persistence;
#[cfg(feature = "experimental-predicate-augmented-search")]
pub mod predicate_augmented;
pub(crate) mod search;
pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use config::{DiskAnnConfig, DiskAnnFallbackPolicy};
pub use types::DiskAnnIndex;
