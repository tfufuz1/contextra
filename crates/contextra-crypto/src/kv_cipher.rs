//! # KV-Segment-Verschlüsselung (`kv_cipher.rs`)
//!
//! Stellt die Authenticated Encryption with Associated Data (AEAD) für Key-Value-Segmente
//! im Cache und Speicher bereit.

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::{CryptoError, Result};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Opaker Modell-Fingerprint zur Bindung von Modellen an Segmente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelFingerprint(pub [u8; 32]);

/// Trait für Key-Value-Chiffren (Ring 0 / Klasse A).
pub trait KvCipher: Send + Sync {
    /// Verschlüsselt Plaintext-Daten.
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Entschlüsselt Ciphertext-Daten.
    fn open(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
}

impl KvCipher for KeyManager {
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let (ct, nonce) = self.encrypt_auto_nonce(plaintext)?;
        let mut envelope = Vec::with_capacity(12 + ct.len());
        envelope.extend_from_slice(&nonce);
        envelope.extend_from_slice(&ct);
        Ok(envelope)
    }

    fn open(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(CryptoError::Crypto("ciphertext too short for nonce".into()));
        }
        let nonce: [u8; 12] = ciphertext[0..12]
            .try_into()
            .map_err(|e: std::array::TryFromSliceError| CryptoError::InvalidInput(e.to_string()))?;
        self.decrypt_auto_nonce(&ciphertext[12..], &nonce)
    }
}

/// Verschlüsselter KV-Layer-Container.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EncryptedKvLayer {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

impl EncryptedKvLayer {
    pub fn new(ciphertext: Vec<u8>, nonce: [u8; 12]) -> Self {
        Self { ciphertext, nonce }
    }
}
