// ANCHOR[id=mvcc-error-root]
//! # Lokale Fehlerdefinition für MVCC (`MvccError`)
//!
//! LINK crates/contextra-mvcc/src/lib.rs
//! LINK crates/contextra-mvcc/src/compat/error.rs
//!
//! Gemäß SPEC-v2 §5.3 (B12):
//! Kontextra-MVCC verwendet einen stark typisierten, lokalen Fehlertyp ohne
//! Zwangskopplung an den monolithischen `ContextraError`.

use thiserror::Error;

// SECTION Error Types
// ANCHOR[id=mvcc-error-enum]
/// Lokaler Fehlertyp für Multi-Version Concurrency Control und Transaktionsverwaltung.
#[non_exhaustive]
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum MvccError {
    /// Nebenläufigkeits- oder SSI-Konflikt (Write-Skew, Anti-Dependency, Phantom-Read).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Ressourcen- oder Kapazitätsgrenze überschritten (z. B. maximale aktive Snapshots).
    #[error("Limit exceeded: {limit} ({context})")]
    LimitExceeded {
        /// Maximale Obergrenze.
        limit: usize,
        /// Kontextbeschreibung oder erschöpfte Ressource.
        context: String,
    },

    /// Transaktionszustandsfehler (z. B. unbekannte TxId oder abgebrochene Transaktion).
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Interner Speicher- oder Pufferierungsfehler.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Ungültige Eingabeparameter (z. B. Sentinel-Werte oder unzulässige Bits).
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Interner System- oder Zustandsfehler.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl MvccError {
    // ANCHOR[id=mvcc-error-limit-exceeded]
    /// Hilfskonstruktor zur Erzeugung eines [`MvccError::LimitExceeded`].
    #[inline]
    pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self {
        Self::LimitExceeded {
            limit,
            context: context.into(),
        }
    }
}

// ANCHOR[id=mvcc-result-type]
/// Lokaler Result-Typ für MVCC-Operationen.
pub type Result<T, E = MvccError> = std::result::Result<T, E>;
// !SECTION Error Types
