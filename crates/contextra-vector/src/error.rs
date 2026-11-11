// FILE-CONTEXT
// ZWECK: Zentrales Fehler-Enum VectorError für contextra-vector (B4/R3-Konformität).
// INVARIANTEN: Zero-Panic, thiserror, non_exhaustive, saubere Zuordnung lokaler Fehler.
// HOTSPOTS: VectorError, Result<T>
// STAND: TS:2026-11-11T02:00:00Z

//! # Fehlerbehandlung für `contextra-vector` (B4- & R3-Konformität)
//!
//! Stellt das typsichere, non-exhaustive Fehler-Enum [`VectorError`] bereit, welches alle
//! Fehlerzustände der Vektor-Indexierungs-, Such- und Persistenzoperationen abbildet.

use thiserror::Error;

// SECTION Fehler-Definitionen

// ANCHOR[contextra-vector:ERR-VECTOR-ERROR]
/// Fehlerzustände von `contextra-vector`.
#[derive(Debug, Error, PartialEq, Clone)]
#[non_exhaustive]
pub enum VectorError {
    /// Ungültige Eingabe oder Konfigurationsparameter.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Strukturelle Beschädigung oder Inkonsistenz des Index.
    #[error("index error: {0}")]
    Index(String),

    /// Dimensionsabweichung bei Einfüge- oder Suchoperationen.
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch {
        /// Erwartete Vektor-Dimensionalität.
        expected: usize,
        /// Tatsächlich übergebene Vektor-Dimensionalität.
        actual: usize,
    },

    /// HNSW-Konnektivitätsdegradation festgestellt.
    #[error("HNSW connectivity degraded: deleted ratio {deleted_ratio:.4}")]
    ConnectivityDegraded {
        /// Verhältnis von gelöschten zu Gesamtknoten.
        deleted_ratio: f64,
    },

    /// Fehler bei der Graph-Reparatur während Knotenlösung/Tombstone-Pruning.
    #[error("graph repair failed: {0}")]
    GraphRepairFailed(String),

    /// Fehler bei Nebenläufigkeit oder Sperren-Erfassung.
    #[error("concurrency error: {0}")]
    Concurrency(String),

    /// Ein-/Ausgabe- oder Persistenzfehler.
    #[error("io error: {0}")]
    Io(String),

    /// Nicht unterstützte Fähigkeit der Index-Engine.
    #[error("unsupported capability: {0}")]
    Unsupported(String),
}

impl VectorError {
    /// Hilfsmethode zur Konstruktion von `InvalidInput`.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }
}

// ANCHOR[contextra-vector:ERR-RESULT]
/// Standardmäßiger Result-Typ für `contextra-vector`.
pub type Result<T> = std::result::Result<T, VectorError>;

impl From<std::io::Error> for VectorError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
// !SECTION
