//! # Kryptographisches Fehlersystem (`error.rs`)
//!
//! Dieses Modul definiert den kanonischen Fehlertyp [`CryptoError`] für alle kryptographischen
//! Operationen in Contextra (Ring 0).
//!
//! ## Invarianten
//! - Keine Panics in Fehlertyp-Konvertierungen.
//! - Fehler enthalten keine sensiblen Klartextdaten oder Schlüsselmaterialien.
//! - Nahtlose Integration in `ContextraError` (Ring 0/Types).

//! Error types for `contextra-crypto`.

use thiserror::Error;

/// Result type alias for cryptographic operations in `contextra-crypto`.
pub type Result<T> = std::result::Result<T, CryptoError>;

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

    #[error("AEAD operation failed: {0}")]
    Aead(String),

    #[error("invalid nonce or key length: {0}")]
    InvalidLength(String),

    #[error("integrity check failed: authentication tag mismatch")]
    IntegrityViolation,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("KV cache format version mismatch: expected {expected}, found {found}")]
    KvFormatVersionMismatch {
        /// Expected format version number.
        expected: u8,
        /// Format version number found in payload.
        found: u8,
    },

    #[error("WAL corruption detected at offset {offset}: {reason}")]
    WalCorruption {
        /// Byte offset in WAL file where corruption occurred.
        offset: u64,
        /// Detail text describing corruption cause.
        reason: String,
    },


    #[error("WAL truncation detected: tail HMAC mismatch")]
    WalTruncationDetected,

    #[error("key or group has been revoked: {0}")]
    KeyRevoked(String),

    #[error("storage or I/O error: {0}")]
    Storage(String),
}

impl CryptoError {
    /// Helper to construct a `WalCorruption` error.
    pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self {
        Self::WalCorruption {
            offset,
            reason: reason.into(),
        }
    }

    /// Helper to construct a `Storage` error.
    pub fn storage(reason: impl Into<String>) -> Self {
        Self::Storage(reason.into())
    }
}
