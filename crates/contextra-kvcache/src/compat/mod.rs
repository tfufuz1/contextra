pub mod adapters;
pub mod prefix_store;
pub mod scanner;
pub mod scratchpad_invalidation;

pub use adapters::*;
pub use prefix_store::*;
pub use scanner::*;
pub use scratchpad_invalidation::*;

// Transitionale Typ-Konvertierungen für Workspace-Kompatibilität.

use crate::types::{ModelFingerprint, TenantId};

#[cfg(feature = "ports-compat")]
impl From<TenantId> for contextra_types::TenantId {
    fn from(t: TenantId) -> Self {
        contextra_types::TenantId::try_new(t.0).unwrap_or(contextra_types::TenantId::SYSTEM)
    }
}

#[cfg(feature = "ports-compat")]
impl From<contextra_types::TenantId> for TenantId {
    fn from(t: contextra_types::TenantId) -> Self {
        Self(t.as_u64())
    }
}

#[cfg(feature = "ports-compat")]
impl From<ModelFingerprint> for contextra_types::model_fingerprint::ModelFingerprint {
    fn from(m: ModelFingerprint) -> Self {
        contextra_types::model_fingerprint::ModelFingerprint::new(m.0, "default", "none")
    }
}

#[cfg(feature = "ports-compat")]
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
