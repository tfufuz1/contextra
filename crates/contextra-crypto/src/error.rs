//! # Kryptographisches Fehlersystem (`error.rs`)
//!
//! Dieses Modul definiert den kanonischen Fehlertyp [`CryptoError`] für alle kryptographischen
//! Operationen in `contextra-crypto` (Ring 0 / Klasse A).

use thiserror::Error;

/// Result type alias for cryptographic operations in `contextra-crypto`.
pub type Result<T> = std::result::Result<T, CryptoError>;

/// Primary Class-A Error type alias.
pub type Error = CryptoError;

/// Standalone error type for all operations within `contextra-crypto`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CryptoError {
    #[error("key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("encryption failed: {0}")]
    Encryption(String),

    #[error("decryption failed: {0}")]
    Decryption(String),

    #[error("invalid nonce or key length: {0}")]
    InvalidLength(String),

    #[error("integrity check failed: authentication tag mismatch")]
    IntegrityViolation,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("capacity exceeded: {0}")]
    CapacityExceeded(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("state conflict: {0}")]
    StateConflict(String),

    #[error("corrupt cryptographic data: {0}")]
    Corrupt(String),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("KV cache format version mismatch: expected {expected}, found {found}")]
    KvFormatVersionMismatch {
        expected: u8,
        found: u8,
    },

    #[error("WAL corruption detected at offset {offset}: {reason}")]
    WalCorruption {
        offset: u64,
        reason: String,
    },

    #[error("WAL truncation detected: tail HMAC mismatch")]
    WalTruncationDetected,

    #[error("storage or I/O error: {0}")]
    Storage(String),
}

impl CryptoError {
    pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self {
        Self::WalCorruption {
            offset,
            reason: reason.into(),
        }
    }
}

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
