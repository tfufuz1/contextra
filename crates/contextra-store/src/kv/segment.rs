// FILE-CONTEXT
// ZWECK: Verdrahtung von KvDeleteMode und contextra_crypto::kv_shredding für KV-Segment-Verschlüsselung und Crypto-Shredding (AP-P0-06 / Spec B.1.8).
// INVARIANTEN:
// - INV-KV-DELETE-1: Löschbeweis auf Key-Value-Seite ist ausschließlich für `CryptoShred`-Segmente möglich, niemals für `TombstoneOnly`.
// - Designprinzip P24: Ein `revoke_subkey`-Aufruf ist O(1) bezüglich der Segmentgröße.

#![forbid(unsafe_code)]

use crate::kv::delete_mode::KvDeleteMode;
use contextra_core::ContextraError;
use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
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
    /// Group ID corresponding to the sub-key in `KeyRegistry`.
    pub group_id: u64,
    /// Physical payload stored on disk (ciphertext or raw plaintext).
    pub payload: Vec<u8>,
    /// AES-GCM-SIV nonce used when payload is encrypted under `CryptoShred` mode.
    pub nonce: Option<[u8; 12]>,
    /// Indicates whether `payload` is encrypted.
    pub is_encrypted: bool,
}

/// Manager for KV segment encryption, decryption, and crypto-shredding.
#[derive(Debug)]
pub struct KvSegmentManager {
    config: KvSegmentConfig,
    registry: Arc<KeyRegistry>,
    master_key: Option<Arc<KeyManager>>,
}

impl KvSegmentManager {
    /// Creates a new `KvSegmentManager` with specified config, key registry, and master key.
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

    /// Writes a KV segment payload according to the configured `delete_mode`.
    ///
    /// Under `KvDeleteMode::CryptoShred`, encrypts `plaintext` using a subkey derived for `group_id` via `KeyRegistry`.
    /// Under `KvDeleteMode::TombstoneOnly`, stores `plaintext` unencrypted without subkey derivation.
    pub fn write_segment(
        &self,
        group_id: u64,
        plaintext: &[u8],
    ) -> Result<KvSegmentPayload, ContextraError> {
        match self.config.delete_mode {
            KvDeleteMode::CryptoShred => {
                let master_key =
                    self.master_key
                        .as_ref()
                        .ok_or(ContextraError::KvDeleteModeConfig(
                            "Master key is required for CryptoShred mode",
                        ))?;
                let (ciphertext, nonce) = self
                    .registry
                    .encrypt_with_group(master_key, group_id, plaintext)
                    .map_err(|e| {
                        ContextraError::Storage(format!("CryptoShred encryption failed: {e}"))
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
    ///
    /// Under `CryptoShred`, decrypts using the subkey in `KeyRegistry`. If the subkey was revoked (deleted),
    /// decryption fails and returns an error.
    /// Under `TombstoneOnly`, returns the raw payload bytes.
    pub fn read_segment(&self, segment: &KvSegmentPayload) -> Result<Vec<u8>, ContextraError> {
        match self.config.delete_mode {
            KvDeleteMode::CryptoShred => {
                if !segment.is_encrypted {
                    return Err(ContextraError::Storage(
                        "Segment is unencrypted but manager is configured for CryptoShred"
                            .to_string(),
                    ));
                }
                let nonce = segment.nonce.as_ref().ok_or_else(|| {
                    ContextraError::Storage("Missing nonce for encrypted KV segment".to_string())
                })?;

                self.registry
                    .decrypt_with_group(segment.group_id, &segment.payload, nonce)
                    .map_err(|e| {
                        ContextraError::Storage(format!(
                            "Failed to decrypt segment for group {}: {e}",
                            segment.group_id
                        ))
                    })
            }
            KvDeleteMode::TombstoneOnly => {
                if segment.is_encrypted {
                    return Err(ContextraError::Storage(
                        "Segment is encrypted but manager is configured for TombstoneOnly"
                            .to_string(),
                    ));
                }
                Ok(segment.payload.clone())
            }
        }
    }

    /// Deletes a KV segment.
    ///
    /// Under `CryptoShred`, revokes (destroys) the subkey for `group_id` in `KeyRegistry` in O(1) time.
    /// The physical ciphertext remains intact on disk, but becomes information-theoretically unrecoverable.
    /// Under `TombstoneOnly`, subkey revocation is a no-op (LSM tombstones are used instead).
    pub fn delete_segment(&self, group_id: u64) -> bool {
        match self.config.delete_mode {
            KvDeleteMode::CryptoShred => self.registry.revoke_subkey(group_id).unwrap_or(false),
            KvDeleteMode::TombstoneOnly => false,
        }
    }

    /// Generates or validates a KV deletion proof.
    ///
    /// INVARIANTE `INV-KV-DELETE-1`: Löschbeweis auf Key-Value-Seite ist ausschließlich für `CryptoShred`-Segmente
    /// möglich, niemals für `TombstoneOnly`. Falls ein Aufrufer versucht, für ein `TombstoneOnly`-Segment einen
    /// Löschbeweis auszustellen, schlägt dies mit `ContextraError::KvDeleteModeConfig` fehl.
    pub fn generate_deletion_proof(&self, group_id: u64) -> Result<bool, ContextraError> {
        match self.config.delete_mode {
            KvDeleteMode::TombstoneOnly => Err(ContextraError::KvDeleteModeConfig(
                "Löschbeweis auf Key-Value-Seite ist ausschließlich für `CryptoShred`-Segmente möglich, niemals für `TombstoneOnly`.",
            )),
            KvDeleteMode::CryptoShred => {
                if !self.registry.was_group_ever_registered(group_id) {
                    return Err(ContextraError::KvDeleteModeConfig(
                        "Löschbeweis unzulässig: Gruppe wurde nie registriert.",
                    ));
                }
                let is_active = self.registry.is_key_active(group_id);
                Ok(!is_active)
            }
        }
    }
}
