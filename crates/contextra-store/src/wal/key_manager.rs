//! Key management for store encryption and authenticated decryption.

use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use contextra_types::{ContextraError, Result};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// In-memory encryption key wrapped with ZeroizeOnDrop.
#[derive(Zeroize, ZeroizeOnDrop)]
struct VolatileKey {
    bytes: [u8; 32],
}

impl VolatileKey {
    fn new(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

/// Central key manager for symmetric encryption in the store.
pub struct KeyManager {
    key: VolatileKey,
    cipher_cache: OnceLock<Aes256GcmSiv>,
    nonce_prefix: [u8; 4],
    nonce_counter: AtomicU64,
}

impl std::fmt::Debug for KeyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyManager")
            .field("key", &"***REDACTED***")
            .field("nonce_prefix", &self.nonce_prefix)
            .finish()
    }
}

impl KeyManager {
    /// Creates a new KeyManager by deriving a key from a passphrase and salt using HKDF-SHA256.
    pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self> {
        if passphrase.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Passphrase cannot be empty".to_string(),
            ));
        }
        if salt.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Salt cannot be empty".to_string(),
            ));
        }
        if salt.len() > 10_000 {
            return Err(ContextraError::InvalidInput(format!(
                "Salt length {} exceeds maximum allowed bound of 10000 bytes",
                salt.len()
            )));
        }
        let hk = Hkdf::<Sha256>::new(Some(salt), passphrase.as_bytes());
        let mut key_raw = [0u8; 32];
        hk.expand(b"contextra-aes-256-gcm-key", &mut key_raw)
            .map_err(|e| ContextraError::Crypto(format!("HKDF expansion failed: {}", e)))?;

        let cipher = Aes256GcmSiv::new_from_slice(&key_raw)
            .map_err(|e| ContextraError::Crypto(format!("Aes256GcmSiv key init failed: {}", e)))?;
        let cipher_cache = OnceLock::new();
        let _ = cipher_cache.set(cipher);

        let mut nonce_prefix = [0u8; 4];
        rand::rngs::OsRng.fill_bytes(&mut nonce_prefix);

        Ok(Self {
            key: VolatileKey::new(key_raw),
            cipher_cache,
            nonce_prefix,
            nonce_counter: AtomicU64::new(1),
        })
    }

    /// Derives a sub-key for a specific file or logical stream.
    pub fn derive_file_key(&self, file_id: &[u8]) -> Result<Self> {
        if file_id.is_empty() {
            return Err(ContextraError::InvalidInput(
                "file_id cannot be empty".to_string(),
            ));
        }
        if file_id.len() > 10_000 {
            return Err(ContextraError::InvalidInput(format!(
                "file_id length {} exceeds maximum allowed bound of 10000 bytes",
                file_id.len()
            )));
        }
        let hk = Hkdf::<Sha256>::from_prk(self.key.as_bytes())
            .map_err(|_| ContextraError::Crypto("Invalid PRK length".to_string()))?;

        let mut sub_key = [0u8; 32];
        let mut info = Vec::with_capacity(b"contextra-file-key-v1:".len() + 4 + file_id.len());
        info.extend_from_slice(b"contextra-file-key-v1:");
        info.extend_from_slice(&(file_id.len() as u32).to_le_bytes());
        info.extend_from_slice(file_id);

        hk.expand(&info, &mut sub_key)
            .map_err(|e| ContextraError::Crypto(format!("HKDF sub-key expansion failed: {}", e)))?;

        let cipher = Aes256GcmSiv::new_from_slice(&sub_key)
            .map_err(|e| ContextraError::Crypto(format!("Aes256GcmSiv key init failed: {}", e)))?;
        let cipher_cache = OnceLock::new();
        let _ = cipher_cache.set(cipher);

        let mut nonce_prefix = [0u8; 4];
        rand::rngs::OsRng.fill_bytes(&mut nonce_prefix);

        Ok(Self {
            key: VolatileKey::new(sub_key),
            cipher_cache,
            nonce_prefix,
            nonce_counter: AtomicU64::new(1),
        })
    }

    /// Derives an integrity key for HMAC-SHA256.
    pub fn integrity_key(&self) -> Result<[u8; 32]> {
        let hk = Hkdf::<Sha256>::from_prk(self.key.as_bytes())
            .map_err(|_| ContextraError::Crypto("Invalid PRK length".to_string()))?;
        let mut key = [0u8; 32];
        hk.expand(b"contextra-hmac-sha256-key", &mut key)
            .map_err(|e| {
                ContextraError::Crypto(format!("HKDF integrity expansion failed: {}", e))
            })?;
        Ok(key)
    }

    fn cipher(&self) -> Result<&Aes256GcmSiv> {
        if let Some(c) = self.cipher_cache.get() {
            return Ok(c);
        }
        let cipher = Aes256GcmSiv::new_from_slice(self.key.as_bytes())
            .map_err(|e| ContextraError::Crypto(format!("Aes256GcmSiv key init failed: {}", e)))?;
        let _ = self.cipher_cache.set(cipher);
        self.cipher_cache.get().ok_or_else(|| {
            ContextraError::Crypto("Failed to retrieve cached Aes256GcmSiv cipher".to_string())
        })
    }

    /// Encrypts a block of data with an auto-incrementing 12-byte nonce.
    pub fn encrypt_auto_nonce(&self, data: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let counter_val = self
            .nonce_counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                if val == u64::MAX {
                    None
                } else {
                    Some(val + 1)
                }
            })
            .map_err(|_| {
                ContextraError::Crypto(
                    "Nonce counter exhausted: maximum encryptions reached for KeyManager instance"
                        .to_string(),
                )
            })?;

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&self.nonce_prefix);
        nonce_bytes[4..12].copy_from_slice(&counter_val.to_be_bytes());

        let cipher = self.cipher()?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| ContextraError::Crypto(format!("Encryption failed: {}", e)))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypts a block of data using a 12-byte nonce.
    pub fn decrypt_auto_nonce(&self, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>> {
        let cipher = self.cipher()?;
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ContextraError::Crypto(format!("Decryption failed: {}", e)))
    }
}
