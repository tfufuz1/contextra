pub mod adapters;
pub mod error;
pub mod prefix_store;
pub mod scanner;
pub mod scratchpad_invalidation;

pub use adapters::*;
pub use prefix_store::*;
pub use scanner::*;
pub use scratchpad_invalidation::*;

// Transitionale Typ-Konvertierungen für Workspace-Kompatibilität.

use crate::types::{ModelFingerprint, RequestId, TenantId, TxId};

#[cfg(feature = "workspace")]
impl From<TenantId> for contextra_types::TenantId {
    fn from(t: TenantId) -> Self {
        contextra_types::TenantId::new(t.0)
    }
}

#[cfg(feature = "workspace")]
impl From<contextra_types::TenantId> for TenantId {
    fn from(t: contextra_types::TenantId) -> Self {
        TenantId(t.as_u64())
    }
}

#[cfg(feature = "workspace")]
impl From<TxId> for contextra_types::TxId {
    fn from(t: TxId) -> Self {
        contextra_types::TxId::from(t.0)
    }
}

#[cfg(feature = "workspace")]
impl From<contextra_types::TxId> for TxId {
    fn from(t: contextra_types::TxId) -> Self {
        TxId(t.inner())
    }
}

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
impl From<contextra_types::model_fingerprint::ModelFingerprint> for ModelFingerprint {
    fn from(m: contextra_types::model_fingerprint::ModelFingerprint) -> Self {
        ModelFingerprint(m.hash)
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

#[cfg(feature = "workspace")]
impl PartialEq<contextra_types::TenantId> for TenantId {
    fn eq(&self, other: &contextra_types::TenantId) -> bool {
        self.0 == other.as_u64()
    }
}

#[cfg(feature = "workspace")]
impl PartialEq<TenantId> for contextra_types::TenantId {
    fn eq(&self, other: &TenantId) -> bool {
        self.as_u64() == other.0
    }
}
