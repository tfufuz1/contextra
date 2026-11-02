// ANCHOR[contextra-mvcc:ERROR-ROOT]
//! # MVCC Fehlerdefinitionen (`MvccError`)
//!
//! Typisierte Fehler für Transaktionsverwaltung, SSI-Validierung und Sequenzbehandlung.

pub use contextra_types::{ContextraError, Result};

/// Typisierte Fehler für `contextra-mvcc`.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum MvccError {
    /// Ungültige Sequenznummer.
    #[error("Invalid sequence number: {0}")]
    InvalidSeq(u64),

    /// Veraltete Variante für Abwärtskompatibilität.
    #[error("Invalid sequence number: {0}")]
    InvalidSequence(u64),

    /// Snapshot wurde widerrufen (z. B. durch Purge / Löschung).
    #[error("Snapshot revoked: sequence {0}")]
    SnapshotRevoked(u64),

    /// Snapshot-Limit je Mandant überschritten.
    #[error("Snapshot limit exceeded")]
    LimitSnapshots,

    /// Datenkonflikt während der Transaktionsvalidierung (SSI / Write-Skew).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Kapazitäts- oder Ressourcenlimit überschritten.
    #[error("Limit exceeded: {0}")]
    LimitExceeded(String),

    /// Allgemeiner interner Transaktionsfehler.
    #[error("Transaction error: {0}")]
    Transaction(String),
}

impl From<MvccError> for ContextraError {
    fn from(err: MvccError) -> Self {
        match err {
            MvccError::InvalidSeq(seq) | MvccError::InvalidSequence(seq) => {
                ContextraError::InvalidSequenceNumber(seq)
            }
            MvccError::SnapshotRevoked(seq) => {
                ContextraError::Conflict(format!("Snapshot revoked at sequence {seq}"))
            }
            MvccError::LimitSnapshots => ContextraError::limit_exceeded(
                1024,
                "Snapshot limit per tenant exceeded".to_string(),
            ),
            MvccError::Conflict(msg) => ContextraError::Conflict(msg),
            MvccError::LimitExceeded(msg) => ContextraError::limit_exceeded(0, msg),
            MvccError::Transaction(msg) => ContextraError::Transaction(msg),
        }
    }
}
