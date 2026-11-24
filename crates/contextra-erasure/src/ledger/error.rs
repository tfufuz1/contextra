//! Lokale Fehlertaxonomie für das Ledger-Subsystem (WP-8, SPEC §11.8).

use thiserror::Error;

/// Fehlerzustände des Ledger-Subsystems.
#[derive(Debug, Error)]
pub enum LedgerError {
    /// Das angeforderte Ledger existiert nicht und `create_if_missing` war `false`.
    #[error("Ledger file/storage not found")]
    NotFound,

    /// Datenkorruption (ungültige Magic, ungültige Prüfsumme, abgerissener Record).
    #[error("Ledger storage corrupt: {0}")]
    Corrupt(String),

    /// Kettenbruch in der Hash-Verkettung.
    #[error("Ledger hash chain broken at sequence {seq}: expected {expected:?}, found {actual:?}")]
    BrokenChain {
        /// Sequenznummer des fehlerhaften Eintrags.
        seq: u64,
        /// Erwarteter Hash.
        expected: Box<[u8; 32]>,
        /// Tatsächlich gefundener Hash.
        actual: Box<[u8; 32]>,
    },

    /// Nicht unterstützte Ledger-Dateiversion.
    #[error("Unsupported ledger version: {0}")]
    UnsupportedVersion(u32),

    /// E/A- oder Speicherbackend-Fehler.
    #[error("Ledger storage I/O error ({0:?}): {1}")]
    Storage(std::io::ErrorKind, String),

    /// Ungültiger Record oder Parameter.
    #[error("Invalid ledger record: {0}")]
    InvalidRecord(String),

    /// Leerer Batch übergeben.
    #[error("Cannot append an empty batch to the ledger")]
    EmptyBatch,
}

impl From<std::io::Error> for LedgerError {
    fn from(err: std::io::Error) -> Self {
        LedgerError::Storage(err.kind(), err.to_string())
    }
}
