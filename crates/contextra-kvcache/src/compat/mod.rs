pub mod adapters;
pub mod prefix_store;
pub mod scanner;
pub mod scratchpad_invalidation;

pub use adapters::*;
pub use prefix_store::*;
pub use scanner::*;
pub use scratchpad_invalidation::*;

// Transitionale Typ-Konvertierungen für Workspace-Kompatibilität.

use crate::types::{ModelFingerprint, RequestId};

#[cfg(feature = "workspace")]
impl From<RequestId> for contextra_ports::RequestId {
    fn from(r: RequestId) -> Self {
        contextra_ports::RequestId(r.0)
    }
}

#[cfg(feature = "workspace")]
impl From<contextra_ports::RequestId> for RequestId {
    fn from(r: contextra_ports::RequestId) -> Self {
        RequestId(r.0)
    }
}

#[cfg(feature = "workspace")]
impl From<ModelFingerprint> for contextra_types::model_fingerprint::ModelFingerprint {
    fn from(m: ModelFingerprint) -> Self {
        contextra_types::model_fingerprint::ModelFingerprint::new(m.0, "default", "none")
    }
}

#[cfg(feature = "workspace")]
impl From<crate::error::KvCacheError> for contextra_types::ContextraError {
    fn from(err: crate::error::KvCacheError) -> Self {
        match err {
            crate::error::KvCacheError::InvalidInput(msg) => {
                contextra_types::ContextraError::InvalidInput(msg)
            }
            crate::error::KvCacheError::CryptoError(msg) => {
                contextra_types::ContextraError::Crypto(msg)
            }
            crate::error::KvCacheError::PinBudgetExceeded(msg) => {
                contextra_types::ContextraError::PinBudgetExceeded(msg)
            }
            crate::error::KvCacheError::LimitExceeded { limit, context } => {
                contextra_types::ContextraError::LimitExceeded { limit, context }
            }
            crate::error::KvCacheError::PolicyViolation(msg) => {
                contextra_types::ContextraError::PolicyViolation(msg)
            }
            crate::error::KvCacheError::Serialization(msg) => {
                contextra_types::ContextraError::Serialization(msg)
            }
            crate::error::KvCacheError::QuantizationError(msg) => {
                contextra_types::ContextraError::KvQuantization(msg)
            }
            crate::error::KvCacheError::Internal(msg) => {
                contextra_types::ContextraError::Internal(msg)
            }
            crate::error::KvCacheError::InvalidTenant(msg) => {
                contextra_types::ContextraError::InvalidInput(format!("Ungültiger Mandant: {msg}"))
            }
            crate::error::KvCacheError::InvalidSegment(msg) => {
                contextra_types::ContextraError::Internal(format!("Ungültiges Segment: {msg}"))
            }
            crate::error::KvCacheError::SegmentNotFound(id) => {
                contextra_types::ContextraError::NotFound(format!("Segment nicht gefunden: {id}"))
            }
            crate::error::KvCacheError::SegmentKeyDestroyed => {
                contextra_types::ContextraError::Internal("Segmentschlüssel ist zerstört".into())
            }
            crate::error::KvCacheError::BudgetExceeded(msg) => {
                contextra_types::ContextraError::MemoryLimitExceeded(msg)
            }
            crate::error::KvCacheError::ConfigError(msg) => {
                contextra_types::ContextraError::Internal(format!("Konfigurationsfehler: {msg}"))
            }
            crate::error::KvCacheError::IoError(err) => contextra_types::ContextraError::Io(err),
        }
    }
}

#[cfg(feature = "crypto-compat")]
use crate::cipher::SegmentCipher;
#[cfg(feature = "crypto-compat")]
use crate::segment::SegmentKey;

#[cfg(feature = "crypto-compat")]
/// Adapter, der contextra-crypto an SegmentCipher anbindet.
pub struct CryptoSegmentCipher;

#[cfg(feature = "crypto-compat")]
impl SegmentCipher for CryptoSegmentCipher {
    fn encrypt_segment(&self, key: &SegmentKey, plaintext: &[u8]) -> crate::error::Result<Vec<u8>> {
        let (nonce, ct) = key
            .encrypt(plaintext)
            .map_err(|e| crate::error::KvCacheError::CryptoError(e.to_string()))?;
        let mut out = Vec::with_capacity(12 + ct.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    fn decrypt_segment(
        &self,
        key: &SegmentKey,
        ciphertext: &[u8],
    ) -> crate::error::Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(crate::error::KvCacheError::CryptoError(
                "Chiffrat zu kurz für Nonce".into(),
            ));
        }
        let (nonce_bytes, ct) = ciphertext.split_at(12);
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(nonce_bytes);
        key.decrypt(ct, &nonce)
            .map_err(|e| crate::error::KvCacheError::CryptoError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::KvCacheError;
    use contextra_types::ContextraError;

    #[test]
    #[allow(clippy::type_complexity)]
    fn test_kv_cache_error_to_contextra_error_mapping() {
        let cases: Vec<(KvCacheError, fn(&ContextraError) -> bool)> = vec![
            (
                KvCacheError::InvalidInput("bad input".into()),
                |e| matches!(e, ContextraError::InvalidInput(msg) if msg == "bad input"),
            ),
            (
                KvCacheError::CryptoError("crypto failure".into()),
                |e| matches!(e, ContextraError::Crypto(msg) if msg == "crypto failure"),
            ),
            (
                KvCacheError::PinBudgetExceeded("limit 64mb".into()),
                |e| matches!(e, ContextraError::PinBudgetExceeded(msg) if msg == "limit 64mb"),
            ),
            (
                KvCacheError::LimitExceeded {
                    limit: 100,
                    context: "max segments".into(),
                },
                |e| matches!(e, ContextraError::LimitExceeded { limit, context } if *limit == 100 && context == "max segments"),
            ),
            (
                KvCacheError::PolicyViolation("disallowed".into()),
                |e| matches!(e, ContextraError::PolicyViolation(msg) if msg == "disallowed"),
            ),
            (
                KvCacheError::Serialization("bincode err".into()),
                |e| matches!(e, ContextraError::Serialization(msg) if msg == "bincode err"),
            ),
            (
                KvCacheError::QuantizationError("kivi err".into()),
                |e| matches!(e, ContextraError::KvQuantization(msg) if msg == "kivi err"),
            ),
            (
                KvCacheError::Internal("core panic".into()),
                |e| matches!(e, ContextraError::Internal(msg) if msg == "core panic"),
            ),
            (
                KvCacheError::InvalidTenant("0".into()),
                |e| matches!(e, ContextraError::InvalidInput(msg) if msg.contains("Ungültiger Mandant: 0")),
            ),
            (
                KvCacheError::InvalidSegment("bad handle".into()),
                |e| matches!(e, ContextraError::Internal(msg) if msg.contains("Ungültiges Segment: bad handle")),
            ),
            (
                KvCacheError::SegmentNotFound(42),
                |e| matches!(e, ContextraError::NotFound(msg) if msg.contains("Segment nicht gefunden: 42")),
            ),
            (
                KvCacheError::SegmentKeyDestroyed,
                |e| matches!(e, ContextraError::Internal(msg) if msg.contains("Segmentschlüssel ist zerstört")),
            ),
            (
                KvCacheError::BudgetExceeded("ram full".into()),
                |e| matches!(e, ContextraError::MemoryLimitExceeded(msg) if msg == "ram full"),
            ),
            (
                KvCacheError::ConfigError("invalid cfg".into()),
                |e| matches!(e, ContextraError::Internal(msg) if msg.contains("Konfigurationsfehler: invalid cfg")),
            ),
            (
                KvCacheError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "file missing",
                )),
                |e| matches!(e, ContextraError::Io(_)),
            ),
        ];

        for (local_err, check) in cases {
            let contextra_err: ContextraError = local_err.into();
            assert!(
                check(&contextra_err),
                "Error conversion failed for {:?}",
                contextra_err
            );
        }
    }
}
