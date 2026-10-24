// FILE-CONTEXT
// ZWECK: Prädikatsagnostischer Suchpfad (ACORN-Muster, Patel et al. 2024).
// INVARIANTEN: FilteredIndex Trait; Zero Panic; Ring-0 Algorithmen.

pub mod gamma_augmentation;
pub mod naive_reference;

pub use gamma_augmentation::compute_gamma_edge_budget;
pub use naive_reference::NaiveReferenceIndex;

use contextra_core::DocId;

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

/// Errors occurring during ACORN search or graph traversal operations.
#[derive(Debug)]
pub enum AcornError {
    /// Dimension mismatch between query and vector database points.
    DimensionMismatch {
        /// Expected dimension.
        expected: usize,
        /// Provided query dimension.
        got: usize,
    },

    /// Invalid search parameter k (e.g. k = 0).
    InvalidK {
        /// Requested k value.
        k: usize,
    },

    /// Invalid gamma augmentation factor.
    InvalidGamma(u32),

    /// Internal calculation or traversal error.
    Internal(String),
}

impl std::fmt::Display for AcornError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AcornError::DimensionMismatch { expected, got } => {
                write!(f, "Invalid query dimension: expected {expected}, got {got}")
            }
            AcornError::InvalidK { k } => {
                write!(f, "Requested k ({k}) is invalid or zero")
            }
            AcornError::InvalidGamma(gamma) => {
                write!(f, "Invalid gamma factor: {gamma}")
            }
            AcornError::Internal(msg) => {
                write!(f, "Internal ACORN error: {msg}")
            }
        }
    }
}

impl std::error::Error for AcornError {}
