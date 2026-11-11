// FILE-CONTEXT
// ZWECK: Zentrales Fehler-Enum VectorError für contextra-vector (B4/R3-Konformität).
// INVARIANTEN: Zero-Panic, thiserror, non_exhaustive, saubere Zuordnung lokaler Fehler.
// HOTSPOTS: VectorError, Result<T>
// STAND: 2026-11-11

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
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Strukturelle Beschädigung oder Inkonsistenz des Index.
    #[error("Index error: {0}")]
    Index(String),

    /// Speicher- oder Persistenzfehler.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Dimensionsabweichung bei Einfüge- oder Suchoperationen.
    #[error("Embedding dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch {
        /// Erwartete Vektor-Dimensionalität.
        expected: usize,
        /// Tatsächlich übergebene Vektor-Dimensionalität.
        actual: usize,
    },

    /// HNSW-Konnektivitätsdegradation festgestellt.
    #[error("HNSW graph connectivity degraded: {deleted_ratio:.1}% deleted nodes")]
    ConnectivityDegraded {
        /// Verhältnis von gelöschten zu Gesamtknoten.
        deleted_ratio: f64,
    },

    /// Fehler bei der Graph-Reparatur während Knotenlösung/Tombstone-Pruning.
    #[error("graph repair failed: {0}")]
    GraphRepairFailed(String),

    /// Ressource, Knoten oder Dokument nicht gefunden.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Konflikt bei Schreiboperation oder Transaktion.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Fehler bei Nebenläufigkeit oder Sperren-Erfassung.
    #[error("Concurrency error: {0}")]
    Concurrency(String),

    /// Ein-/Ausgabe- oder Persistenzfehler.
    #[error("I/O error: {0}")]
    Io(String),

    /// Nicht unterstützte Fähigkeit der Index-Engine.
    #[error("Capability unsupported: vector - {0}")]
    Unsupported(String),

    /// Interner Fehlerzustand.
    #[error("Internal error: {0}")]
    Internal(String),
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
