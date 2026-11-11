//! Lokale Fehlertypen fuer contextra-store (SPEC §5.3, §6.2)

use thiserror::Error;

/// Lokaler Fehlertyp des LSM-Speicherkerns `contextra-store`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StoreError {
    /// Speicherfehler im LSM-Kern.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Standard I/O-Fehler.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Korruption im Write-Ahead-Log.
    #[error("WAL corruption detected at offset {offset}: {reason}")]
    WalCorruption {
        /// Byte-Offset im WAL.
        offset: u64,
        /// Detailbeschreibung.
        reason: String,
    },

    /// Pruefsummenfehler beim Blocklesen.
    #[error("Checksum mismatch: file={path}, block={block_id}")]
    ChecksumMismatch {
        /// Pfad der Datei.
        path: String,
        /// Block-ID oder Offset.
        block_id: u64,
    },

    /// Unerwartetes Abschneiden oder Kuerzung des WAL.
    #[error("WAL truncation attack detected: expected tail HMAC {expected_hmac:?}, actual tail HMAC {actual_hmac:?}")]
    WalTruncationDetected {
        /// Erwarteter HMAC.
        expected_hmac: [u8; 32],
        /// Tatsaechlicher HMAC.
        actual_hmac: [u8; 32],
    },

    /// Fehler beim Hardlink ueber Dateisystemgrenzen.
    #[error("Cross-device link failed: cannot hard link '{source_path}' to '{target_path}' across distinct filesystems/mounts")]
    CrossDeviceLink {
        /// Quellpfad.
        source_path: String,
        /// Zielpfad.
        target_path: String,
    },

    /// Transaktionsbezogener Fehler.
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Konflikt bei Schreiboperation.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Timeout bei Gruppen-Commit.
    #[error("Group commit timed out for transaction {tx_id}")]
    CommitTimeout {
        /// Transaktions-ID.
        tx_id: u64,
    },

    /// Serialisierungsfehler.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Parsefehler beim Lesen von Datenstrukturen.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Ungueltige Eingabeparameter.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Schluessel oder Eintrag nicht gefunden.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Interner System- oder Logikfehler.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Speicherbudget ueberschritten.
    #[error("Memory budget exceeded: {used_mb}MB / {limit_mb}MB")]
    MemoryBudgetExceeded {
        /// Genutzter Speicher in MB.
        used_mb: u64,
        /// Limit in MB.
        limit_mb: u64,
    },

    /// Limit fuer Scan oder Eintraege ueberschritten.
    #[error("Limit exceeded: {context} (limit: {limit})")]
    LimitExceeded {
        /// Limit.
        limit: usize,
        /// Kontext.
        context: String,
    },

    /// Konfigurationsfehler im Durability-Modus.
    #[error("Durability configuration error: {0}")]
    DurabilityConfig(String),

    /// Sandbox error.
    #[error("Sandbox error: {0}")]
    Sandbox(String),
}

impl StoreError {
    /// Erzeugt einen neuen `StoreError::WalCorruption`.
    #[must_use]
    pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self {
        Self::WalCorruption {
            offset,
            reason: reason.into(),
        }
    }

    /// Erzeugt einen neuen `StoreError::ChecksumMismatch`.
    #[must_use]
    pub fn checksum_mismatch(path: impl Into<String>, block_id: u64) -> Self {
        Self::ChecksumMismatch {
            path: path.into(),
            block_id,
        }
    }

    /// Erzeugt einen neuen `StoreError::InvalidInput`.
    #[must_use]
    pub fn wal_truncation_detected(expected_hmac: [u8; 32], actual_hmac: [u8; 32]) -> Self {
        Self::WalTruncationDetected {
            expected_hmac,
            actual_hmac,
        }
    }

    pub fn invalid_input(reason: impl Into<String>) -> Self {
        Self::InvalidInput(reason.into())
    }
}

impl From<contextra_crypto::CryptoError> for StoreError {
    fn from(err: contextra_crypto::CryptoError) -> Self {
        StoreError::Storage(format!("Crypto error: {err}"))
    }
}

/// Ergebnis-Typ fuer Speicheroperationen.
pub type StoreResult<T> = std::result::Result<T, StoreError>;

impl From<contextra_mvcc::MvccError> for StoreError {
    fn from(err: contextra_mvcc::MvccError) -> Self {
        match err {
            contextra_mvcc::MvccError::Conflict(s) => StoreError::Conflict(s),
            contextra_mvcc::MvccError::Transaction(s) => StoreError::Transaction(s),
            contextra_mvcc::MvccError::LimitExceeded { limit, context } => StoreError::LimitExceeded { limit, context },
            contextra_mvcc::MvccError::Storage(s) => StoreError::Storage(s),
            contextra_mvcc::MvccError::InvalidInput(s) => StoreError::InvalidInput(s),
            other => StoreError::Transaction(other.to_string()),
        }
    }
}
