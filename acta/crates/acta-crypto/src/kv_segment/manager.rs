//! # Segmentierte KV-Verwaltung
//!
//! Ermöglicht granulare Schlüsselverwaltung für KV-Segmente.

// FILE-CONTEXT
// ZWECK: Verwaltung von KV-Segment-Verschlüsselung mit KvDeleteMode.
// INVARIANTEN: Verhalten des Segment-Caches unverändert.
// STAND: TS:2026-11-09

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::CryptoError;
use crate::kv_segment::delete_mode::KvDeleteMode;
use std::sync::Arc;

/// Configuration for KV segment storage behavior.
#[derive(Debug, Clone, Default)]
pub struct KvSegmentConfig {
    /// Delete mode specifying whether segments are tombstone-only or crypto-shredded.
    pub delete_mode: KvDeleteMode,
}

/// Representation of a written KV segment (encrypted or raw payload with group_id and nonce).
#[derive(Debug, Clone)]
pub struct KvSegmentPayload {
    /// Group ID corresponding to the segment/group.
    pub group_id: u64,
    /// Physical payload stored on disk (ciphertext or raw plaintext).
    pub payload: Vec<u8>,
    /// AES-GCM-SIV nonce used when payload is encrypted.
    pub nonce: Option<[u8; 12]>,
    /// Indicates whether payload is encrypted.
    pub is_encrypted: bool,
}

/// Manager for KV segment encryption and decryption.
#[derive(Debug)]
#[deprecated(
    since = "0.1.0",
    note = "KV-Segment Logik migriert nach acta-kvcache (ADR-PROPOSAL-crypto-kv-segment, E-04). Bitte acta-kvcache verwenden."
)]
pub struct KvSegmentManager {
    config: KvSegmentConfig,
    master_key: Option<Arc<KeyManager>>,
}

impl KvSegmentManager {
    /// Creates a new manager with specified config and master key.
    pub fn new(
        config: KvSegmentConfig,
        master_key: Option<Arc<KeyManager>>,
    ) -> Self {
        Self {
            config,
            master_key,
        }
    }

    /// Returns the current delete mode.
    pub fn delete_mode(&self) -> KvDeleteMode {
        self.config.delete_mode
    }

    /// Writes a KV segment payload according to the configured mode.
    pub fn write_segment(
        &self,
        group_id: u64,
        plaintext: &[u8],
    ) -> Result<KvSegmentPayload, CryptoError> {
        match self.config.delete_mode {
            KvDeleteMode::CryptoShred => {
                let master_key = self.master_key.as_ref().ok_or_else(|| {
                    CryptoError::Crypto("Master key is required for CryptoShred mode".to_string())
                })?;
                let (ciphertext, nonce) = master_key
                    .encrypt_auto_nonce(plaintext)
                    .map_err(|e| {
                        CryptoError::Crypto(format!("Encryption failed: {e}"))
                    })?;

                Ok(KvSegmentPayload {
                    group_id,
                    payload: ciphertext,
                    nonce: Some(nonce),
                    is_encrypted: true,
                })
            }
            KvDeleteMode::TombstoneOnly => Ok(KvSegmentPayload {
                group_id,
                payload: plaintext.to_vec(),
                nonce: None,
                is_encrypted: false,
            }),
        }
    }

    /// Reads/decrypts a KV segment payload.
    pub fn read_segment(&self, segment: &KvSegmentPayload) -> Result<Vec<u8>, CryptoError> {
        match self.config.delete_mode {
            KvDeleteMode::CryptoShred => {
                if !segment.is_encrypted {
                    return Err(CryptoError::Crypto(
                        "Segment is unencrypted but manager is configured for CryptoShred"
                            .to_string(),
                    ));
                }
                let master_key = self.master_key.as_ref().ok_or_else(|| {
                    CryptoError::Crypto("Master key is required for CryptoShred mode".to_string())
                })?;
                let nonce = segment.nonce.as_ref().ok_or_else(|| {
                    CryptoError::Crypto("Missing nonce for encrypted KV segment".to_string())
                })?;

                master_key
                    .decrypt_auto_nonce(&segment.payload, nonce)
                    .map_err(|e| {
                        CryptoError::Crypto(format!(
                            "Failed to decrypt segment for group {}: {e}",
                            segment.group_id
                        ))
                    })
            }
            KvDeleteMode::TombstoneOnly => {
                if segment.is_encrypted {
                    return Err(CryptoError::Crypto(
                        "Segment is encrypted but manager is configured for TombstoneOnly"
                            .to_string(),
                    ));
                }
                Ok(segment.payload.clone())
            }
        }
    }

    /// Deletes a KV segment.
    pub fn delete_segment(&self, _group_id: u64) -> Result<bool, CryptoError> {
        Ok(false)
    }

}
