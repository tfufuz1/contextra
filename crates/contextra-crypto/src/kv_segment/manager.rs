//! # Segmentierte KV-Verwaltung & Shredding
//!
//! Ermoeglicht granulare Schluesselverwaltung und O(1) Bereinigung sensibler
//! Gruppen durch Schluesselrevokation in der [].

// FILE-CONTEXT
// ZWECK: Verdrahtung von KvDeleteMode und contextra_crypto::kv_shredding für KV-Segment-Verschlüsselung und Crypto-Shredding (AP-P0-06 / Spec B.1.8).
// INVARIANTEN:
// - INV-KV-DELETE-1: Bereinigungsnachweis auf Key-Value-Seite ist ausschließlich für -Segmente möglich, niemals für .
// - Designprinzip P24: Ein -Aufruf ist O(1) bezüglich der Segmentgröße.

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::CryptoError;
use crate::kv_segment::delete_mode::KvDeleteMode;
use crate::kv_shredding::KeyRegistry;
use std::sync::Arc;

/// Configuration for KV segment storage and shredding behavior.
#[derive(Debug, Clone, Default)]
pub struct KvSegmentConfig {
    /// Delete mode specifying whether segments are tombstone-only or crypto-shredded.
    pub delete_mode: KvDeleteMode,
}

/// Representation of a written KV segment (encrypted or raw payload with group_id and nonce).
#[derive(Debug, Clone)]
pub struct KvSegmentPayload {
    /// Group ID corresponding to the sub-key in .
    pub group_id: u64,
    /// Physical payload stored on disk (ciphertext or raw plaintext).
    pub payload: Vec<u8>,
    /// AES-GCM-SIV nonce used when payload is encrypted under  mode.
    pub nonce: Option<[u8; 12]>,
    /// Indicates whether  is encrypted.
    pub is_encrypted: bool,
}

/// Manager for KV segment encryption, decryption, and crypto-shredding.
#[derive(Debug)]
#[deprecated(
    since = "0.1.0",
    note = "KV-Segment Logik migriert nach contextra-kvcache (ADR-PROPOSAL-crypto-kv-segment, E-04). Bitte contextra-kvcache verwenden."
)]
pub struct KvSegmentManager {
    config: KvSegmentConfig,
    registry: Arc<KeyRegistry>,
    master_key: Option<Arc<KeyManager>>,
}

impl KvSegmentManager {
    /// Creates a new  with specified config, key registry, and master key.
    pub fn new(
        config: KvSegmentConfig,
        registry: Arc<KeyRegistry>,
        master_key: Option<Arc<KeyManager>>,
    ) -> Self {
        Self {
            config,
            registry,
            master_key,
        }
    }

    /// Returns the current delete mode.
    pub fn delete_mode(&self) -> KvDeleteMode {
        self.config.delete_mode
    }

    /// Writes a KV segment payload according to the configured .
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
                let (ciphertext, nonce) = self
                    .registry
                    .encrypt_with_group(master_key, group_id, plaintext)
                    .map_err(|e| {
                        CryptoError::Crypto(format!("CryptoShred encryption failed: {e}"))
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
                let nonce = segment.nonce.as_ref().ok_or_else(|| {
                    CryptoError::Crypto("Missing nonce for encrypted KV segment".to_string())
                })?;

                self.registry
                    .decrypt_with_group(segment.group_id, &segment.payload, nonce)
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
    pub fn delete_segment(&self, group_id: u64) -> Result<bool, CryptoError> {
        match self.config.delete_mode {
            KvDeleteMode::CryptoShred => self.registry.revoke_subkey(group_id),
            KvDeleteMode::TombstoneOnly => Ok(false),
        }
    }

    /// Generates or validates a KV deletion proof.
    pub fn generate_deletion_proof(&self, group_id: u64) -> Result<bool, CryptoError> {
        match self.config.delete_mode {
            KvDeleteMode::TombstoneOnly => Err(CryptoError::Crypto(
                "Bereinigungsnachweis auf Key-Value-Seite ist ausschließlich für `CryptoShred`-Segmente möglich, niemals für `TombstoneOnly`.".to_string(),
            )),
            KvDeleteMode::CryptoShred => {
                let is_active = self.registry.is_key_active(group_id);
                Ok(!is_active)
            }
        }
    }
}
