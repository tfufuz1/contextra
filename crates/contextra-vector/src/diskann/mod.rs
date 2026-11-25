// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> mmap -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Mmap für Vektor- & Graphlesezugriffe, ausgelagert an contextra-sys::mmap_readonly.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)

pub(crate) mod build;
pub(crate) mod config;
pub(crate) mod filtered;
pub(crate) mod format;
pub(crate) mod persistence;
#[cfg(feature = "experimental-predicate-augmented-search")]
pub mod predicate_augmented;
pub(crate) mod search;
pub(crate) mod types;
pub(crate) mod vector_index_impl;

#[cfg(test)]
mod tests;

pub use config::{DiskAnnConfig, DiskAnnFallbackPolicy};
pub use types::DiskAnnIndex;
