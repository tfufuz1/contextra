// FILE-CONTEXT
// ZWECK: HKDF-Sub-Key-Ableitung und Key-Registry für KV-Crypto-Shredding (AP-P0-06).
// INVARIANTEN: CryptoShred vernichtet Sub-Key-Registry-Eintrag (O(1)). Amortisiert HKDF via group_id. Zeroizing für SubKeys.
// HOTSPOTS: [SubKey, KeyRegistry, derive_subkey, revoke_subkey]

//! HKDF Sub-Key derivation and thread-safe registry for KV crypto-shredding.

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::{CryptoError, Result};
use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::RwLock;
use zeroize::ZeroizeOnDrop;

/// Default size of a shred key group (number of records sharing one sub-key).
pub const DEFAULT_SHRED_KEY_GROUP_SIZE: usize = 64;

/// Derived sub-key (256 bits) that is zeroized on drop.
#[derive(ZeroizeOnDrop)]
pub struct SubKey(pub [u8; 32]);

impl Clone for SubKey {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl std::fmt::Debug for SubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubKey")
            .field("bytes", &"***REDACTED***")
            .finish()
    }
}

/// Derives a sub-key for a shred group from a master key (`KeyManager`) using HKDF-SHA256.
pub fn derive_subkey(master_key: &KeyManager, group_id: u64) -> Result<SubKey> {
    let hk = Hkdf::<Sha256>::from_prk(master_key.master_key_bytes())
        .map_err(|_| CryptoError::Crypto("Invalid PRK length in KeyManager".to_string()))?;

    let mut info = Vec::with_capacity(b"contextra-kv-shred-v1:".len() + 8);
    info.extend_from_slice(b"contextra-kv-shred-v1:");
    info.extend_from_slice(&group_id.to_le_bytes());

    let mut sub_key_bytes = [0u8; 32];
    hk.expand(&info, &mut sub_key_bytes)
        .map_err(|e| CryptoError::Crypto(format!("HKDF sub-key expansion failed: {e}")))?;

    Ok(SubKey(sub_key_bytes))
}

/// In-memory thread-safe registry for shred group sub-keys.
#[derive(Debug, Default)]
pub struct KeyRegistry {
    entries: RwLock<HashMap<u64, SubKey>>,
}

impl KeyRegistry {
    /// Creates a new empty `KeyRegistry`.
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Derives or retrieves a sub-key for the specified `group_id`.
    /// If the key does not exist or was revoked, derives a new sub-key from `master_key` and stores it.
    pub fn get_or_derive(&self, master_key: &KeyManager, group_id: u64) -> Result<SubKey> {
        {
            let read_guard = self.entries.read().map_err(|_| {
                CryptoError::Crypto("KeyRegistry read lock poisoned".to_string())
            })?;
            if let Some(key) = read_guard.get(&group_id) {
                return Ok(key.clone());
            }
        }

        let mut write_guard = self.entries.write().map_err(|_| {
            CryptoError::Crypto("KeyRegistry write lock poisoned".to_string())
        })?;
        if let Some(key) = write_guard.get(&group_id) {
            return Ok(key.clone());
        }

        let subkey = derive_subkey(master_key, group_id)?;
        write_guard.insert(group_id, subkey.clone());
        Ok(subkey)
    }

    /// Revokes (destroys) the sub-key for `group_id` in O(1).
    /// Returns `true` if a key was present and removed.
    pub fn revoke_subkey(&self, group_id: u64) -> bool {
        if let Ok(mut write_guard) = self.entries.write() {
            write_guard.remove(&group_id).is_some()
        } else {
            false
        }
    }

    /// Returns `true` if `group_id` has an active sub-key in the registry.
    pub fn is_key_active(&self, group_id: u64) -> bool {
        if let Ok(read_guard) = self.entries.read() {
            read_guard.contains_key(&group_id)
        } else {
            false
        }
    }

    /// Encrypts plaintext using the sub-key for `group_id`.
    pub fn encrypt_with_group(
        &self,
        master_key: &KeyManager,
        group_id: u64,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, [u8; 12])> {
        let subkey = self.get_or_derive(master_key, group_id)?;
        let cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv init failed: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::Crypto(format!("Encryption failed: {e}")))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypts ciphertext using the sub-key for `group_id`.
    /// Returns `Err(CryptoError::Crypto(...))` if key was revoked or decryption fails.
    pub fn decrypt_with_group(
        &self,
        group_id: u64,
        ciphertext: &[u8],
        nonce_bytes: &[u8; 12],
    ) -> Result<Vec<u8>> {
        let read_guard = self
            .entries
            .read()
            .map_err(|_| CryptoError::Crypto("KeyRegistry read lock poisoned".to_string()))?;

        let subkey = read_guard
            .get(&group_id)
            .ok_or_else(|| CryptoError::Crypto(format!("Sub-key for group {group_id} has been revoked or is missing")))?;

        let cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv init failed: {e}")))?;

        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::Crypto(format!("Decryption failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_subkey_determinism() -> Result<()> {
        let km = KeyManager::try_new("test-passphrase", b"salt1")?;
        let k1 = derive_subkey(&km, 42)?;
        let k2 = derive_subkey(&km, 42)?;
        let k3 = derive_subkey(&km, 43)?;

        assert_eq!(k1.0, k2.0);
        assert_ne!(k1.0, k3.0);
        Ok(())
    }

    #[test]
    fn test_key_registry_encrypt_decrypt_and_revoke() -> Result<()> {
        let km = KeyManager::try_new("test-passphrase", b"salt1")?;
        let registry = KeyRegistry::new();
        let group_id = 100;
        let plaintext = b"Sensitive Document Payload";

        let (ciphertext, nonce) = registry.encrypt_with_group(&km, group_id, plaintext)?;
        assert!(registry.is_key_active(group_id));

        let decrypted = registry.decrypt_with_group(group_id, &ciphertext, &nonce)?;
        assert_eq!(decrypted, plaintext);

        // Revoke sub-key
        let revoked = registry.revoke_subkey(group_id);
        assert!(revoked);
        assert!(!registry.is_key_active(group_id));

        // Decryption now fails
        let res = registry.decrypt_with_group(group_id, &ciphertext, &nonce);
        assert!(res.is_err());

        Ok(())
    }
}
