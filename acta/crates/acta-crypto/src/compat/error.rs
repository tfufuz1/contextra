//! # Fehler-Kompatibilität (`compat/error.rs`)
//!
//! Stellt die Konvertierung von [`CryptoError`] nach [`acta_types::ActaError`] bereit.

use crate::error::CryptoError;
use acta_types::ActaError;

impl From<CryptoError> for ActaError {
    fn from(e: CryptoError) -> Self {
        match e {
            CryptoError::WalCorruption { offset, reason } => Self::wal_corruption(offset, reason),
            CryptoError::InvalidInput(msg) => Self::InvalidInput(msg),
            CryptoError::KvFormatVersionMismatch { expected, found } => Self::InvalidInput(
                format!("KV cache format version mismatch: expected {expected}, found {found}"),
            ),
            CryptoError::Storage(msg) => Self::Storage(msg),
            other => Self::Crypto(other.to_string()),
        }
    }
}

impl From<ActaError> for CryptoError {
    fn from(e: ActaError) -> Self {
        CryptoError::Storage(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::type_complexity)]
    fn test_table_driven_from_crypto_error() {
        let cases: Vec<(CryptoError, Box<dyn Fn(&ActaError) -> bool>)> = vec![
            (
                CryptoError::WalCorruption {
                    offset: 42,
                    reason: "bad hmac".to_string(),
                },
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::WalCorruption {
                            offset: 42,
                            ref reason,
                            ..
                        } if reason == "bad hmac"
                    )
                }),
            ),
            (
                CryptoError::InvalidInput("bad key".to_string()),
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::InvalidInput(ref msg) if msg == "bad key"
                    )
                }),
            ),
            (
                CryptoError::KvFormatVersionMismatch {
                    expected: 2,
                    found: 1,
                },
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::InvalidInput(ref msg) if msg.contains("expected 2, found 1")
                    )
                }),
            ),
            (
                CryptoError::KeyDerivation("hkdf failed".to_string()),
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::Crypto(ref msg) if msg.contains("key derivation failed: hkdf failed")
                    )
                }),
            ),
            (
                CryptoError::Encryption("aes failed".to_string()),
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::Crypto(ref msg) if msg.contains("encryption failed: aes failed")
                    )
                }),
            ),
            (
                CryptoError::Decryption("corrupted tag".to_string()),
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::Crypto(ref msg) if msg.contains("decryption failed: corrupted tag")
                    )
                }),
            ),
            (
                CryptoError::Aead("auth fail".to_string()),
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::Crypto(ref msg) if msg.contains("AEAD operation failed: auth fail")
                    )
                }),
            ),
            (
                CryptoError::InvalidLength("16 bytes".to_string()),
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::Crypto(ref msg) if msg.contains("invalid nonce or key length: 16 bytes")
                    )
                }),
            ),
            (
                CryptoError::IntegrityViolation,
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::Crypto(ref msg) if msg.contains("integrity check failed: authentication tag mismatch")
                    )
                }),
            ),
            (
                CryptoError::WalTruncationDetected,
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::Crypto(ref msg) if msg.contains("WAL truncation detected: tail HMAC mismatch")
                    )
                }),
            ),
            (
                CryptoError::KeyRevoked("group_1".to_string()),
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::Crypto(ref msg) if msg.contains("key or group has been revoked: group_1")
                    )
                }),
            ),
            (
                CryptoError::Storage("io error disk full".to_string()),
                Box::new(|err| {
                    matches!(
                        err,
                        ActaError::Storage(ref msg) if msg == "io error disk full"
                    )
                }),
            ),
        ];

        for (crypto_err, check_fn) in cases {
            let display_str = crypto_err.to_string();
            let mf_err: ActaError = crypto_err.into();
            assert!(
                check_fn(&mf_err),
                "Failed check for error variant: {display_str}"
            );
        }
    }
}
