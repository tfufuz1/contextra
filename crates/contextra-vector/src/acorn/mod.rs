// FILE-CONTEXT
// ZWECK: Prädikatsagnostischer Suchpfad (ACORN-Muster, Patel et al. 2024).
// INVARIANTEN: FilteredIndex Trait; Zero Panic; Ring-0 Algorithmen.
// HOTSPOTS: FilteredIndex::search_knn_acorn, AcornError
// STAND: 2026-11-12

// SECTION Submodule & Re-Exports

// ANCHOR[contextra-vector:ACORN-MOD-GAMMA]
pub mod gamma_augmentation;
// ANCHOR[contextra-vector:ACORN-MOD-NAIVE]
pub mod naive_reference;

pub use gamma_augmentation::compute_gamma_edge_budget;
pub use naive_reference::NaiveReferenceIndex;

use crate::types::DocId;
use thiserror::Error;
// !SECTION

// SECTION Trait & Fehler-Definitionen

// ANCHOR[contextra-vector:ACORN-TRAIT-FILTERED-INDEX]
/// Prädikatsagnostischer Suchpfad (ACORN-Muster, Patel et al. 2024).
/// Im Gegensatz zu Post-Filtering wird das Prädikat WÄHREND der Graph-Traversal ausgewertet,
/// mit erhöhter Kantenzahl (γ-Augmentierung) zur Konnektivitätssicherung bei hoher Selektivität.
pub trait FilteredIndex {
    type Predicate: ?Sized;

    fn search_knn_acorn(
        &self,
        query: &[f32],
        k: usize,
        predicate: &Self::Predicate,
        gamma: u32,
    ) -> Result<Vec<(DocId, f32)>, AcornError>;
}

// ANCHOR[contextra-vector:ACORN-ERROR]
/// Errors occurring during ACORN search or graph traversal operations.
#[derive(Debug, Error)]
pub enum AcornError {
    /// Dimension mismatch between query and vector database points.
    #[error("Invalid query dimension: expected {expected}, got {got}")]
    DimensionMismatch {
        /// Expected dimension.
        expected: usize,
        /// Provided query dimension.
        got: usize,
    },

    /// Invalid search parameter k (e.g. k = 0).
    #[error("Requested k ({k}) is invalid or zero")]
    InvalidK {
        /// Requested k value.
        k: usize,
    },

    /// Invalid gamma augmentation factor.
    #[error("Invalid gamma factor: {0}")]
    InvalidGamma(u32),

    /// Internal calculation or traversal error.
    #[error("Internal ACORN error: {0}")]
    Internal(String),
}
// !SECTION
