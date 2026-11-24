//! Transitionale Typ-Konvertierungen für Workspace-Kompatibilität.

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
        contextra_types::ContextraError::Internal(err.to_string())
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
