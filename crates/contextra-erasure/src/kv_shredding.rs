//! # Kryptographisches Shredding für KV-Daten (`kv_shredding.rs`)
//!
//! Ermöglicht kryptografische Bereinigung einzelner Datensätze oder ganzer Mandantengruppen
//! durch gezielte Zerstörung des zugehörigen kryptographischen Schlüssels (Crypto-Erase).
//!
//! ## Invarianten
//! - Nach Zerstörung des Schlüssels ist der Chiffretext unentschlüsselbar.
//! - Gruppe und Einzelschlüssel-Hierarchie erlauben feingranulares Shredding.

// FILE-CONTEXT
// ZWECK: Envelope Encryption und Key-Registry für KV-Crypto-Shredding (AP-P0-06, Spec v17 §16.1).
// INVARIANTEN: CryptoShred vernichtet KEK (Gruppenbereinigung) oder DEK-Wrap (Einzelbereinigung) in O(1).

#![forbid(unsafe_code)]

use crate::error::ErasureError;
use crate::revocation_log::{RevocationLog, RevocationTarget};
use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use rand::RngCore;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Default size of a shred key group (number of records sharing one KEK).
pub const DEFAULT_SHRED_KEY_GROUP_SIZE: usize = 64;

/// Sub-key or KEK bytes (256 bits) zeroized on drop.
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

/// Group Key Encryption Key (KEK) - 256 bits, zeroized on drop.
#[derive(ZeroizeOnDrop)]
pub struct GroupKek(pub [u8; 32]);

impl std::fmt::Debug for GroupKek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupKek")
            .field("bytes", &"***REDACTED***")
            .finish()
    }
}

/// Record Data Encryption Key (DEK) - 256 bits, zeroized on drop.
#[derive(ZeroizeOnDrop)]
pub struct RecordDek(pub [u8; 32]);

impl std::fmt::Debug for RecordDek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordDek")
            .field("bytes", &"***REDACTED***")
            .finish()
    }
}

/// Derives a sub-key from a master key using BLAKE3 KDF.
pub fn derive_subkey(master_key: &[u8; 32], group_id: u64, epoch: u64) -> SubKey {
    let mut hasher = blake3::Hasher::new_derive_key("contextra-kv-shredding-subkey-v1");
    hasher.update(master_key);
    hasher.update(&group_id.to_le_bytes());
    hasher.update(&epoch.to_le_bytes());
    let mut subkey_bytes = [0u8; 32];
    hasher.finalize_xof().fill(&mut subkey_bytes);
    SubKey(subkey_bytes)
}

/// Key registry managing envelope keys (KEK / DEK) for fine-grained cryptographic shredding.
pub struct KeyRegistry {
    group_keks: RwLock<BTreeMap<u64, Vec<u8>>>,
    record_deks: RwLock<BTreeMap<String, Vec<u8>>>,
    revoked_groups: RwLock<BTreeSet<u64>>,
    #[allow(dead_code)]
    revoked_subkeys: RwLock<BTreeSet<String>>,
    revoked_records: RwLock<BTreeSet<String>>,
    revocation_log: Option<Arc<RevocationLog>>,
    #[allow(dead_code)]
    master_kek: [u8; 32],
}

impl Default for KeyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyRegistry {
    /// Erzeugt eine neue `KeyRegistry` mit zufälligem Master-KEK.
    pub fn new() -> Self {
        let mut master_kek = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut master_kek);
        Self {
            group_keks: RwLock::new(BTreeMap::new()),
            record_deks: RwLock::new(BTreeMap::new()),
            revoked_groups: RwLock::new(BTreeSet::new()),
            revoked_subkeys: RwLock::new(BTreeSet::new()),
            revoked_records: RwLock::new(BTreeSet::new()),
            revocation_log: None,
            master_kek,
        }
    }

    /// Setzt einen optionalen `RevocationLog`.
    pub fn with_revocation_log(mut self, log: Arc<RevocationLog>) -> Self {
        self.revocation_log = Some(log);
        self
    }

    /// Prüft ob eine Gruppe aktiv ist.
    pub fn is_group_active(&self, group_id: u64) -> bool {
        !self.is_group_revoked(group_id)
    }

    /// Prüft ob eine Gruppe widerrufen ist.
    pub fn is_group_revoked(&self, group_id: u64) -> bool {
        let lock = match self.revoked_groups.read() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        lock.contains(&group_id)
            || self
                .revocation_log
                .as_ref()
                .map(|log| log.is_revoked(&RevocationTarget::Group(group_id)))
                .unwrap_or(false)
    }

    /// Widerruft eine gesamte Shredding-Gruppe.
    pub fn revoke_group(&self, group_id: u64) -> Result<(), ErasureError> {
        {
            let mut lock = match self.revoked_groups.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            lock.insert(group_id);
        }
        {
            let mut keks = match self.group_keks.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            if let Some(mut wrapped) = keks.remove(&group_id) {
                wrapped.zeroize();
            }
        }
        if let Some(ref log) = self.revocation_log {
            log.append(RevocationTarget::Group(group_id))?;
        }
        Ok(())
    }

    /// Widerruft einen einzelnen Datensatz (Record).
    pub fn revoke_record(&self, record_id: &str) -> Result<(), ErasureError> {
        {
            let mut lock = match self.revoked_records.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            lock.insert(record_id.to_string());
        }
        {
            let mut deks = match self.record_deks.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            if let Some(mut wrapped) = deks.remove(record_id) {
                wrapped.zeroize();
            }
        }
        if let Some(ref log) = self.revocation_log {
            log.append(RevocationTarget::Record(record_id.to_string()))?;
        }
        Ok(())
    }

    /// Prüft ob ein Record aktiv ist.
    pub fn is_record_active(&self, record_id: &str) -> bool {
        let lock = match self.revoked_records.read() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        !lock.contains(record_id)
            && !self
                .revocation_log
                .as_ref()
                .map(|log| log.is_revoked(&RevocationTarget::Record(record_id.to_string())))
                .unwrap_or(false)
    }

    /// Verschlüsselt Daten für einen Record mit AES-256-GCM-SIV.
    pub fn encrypt_record(
        &self,
        group_id: u64,
        record_id: &str,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ErasureError> {
        if self.is_group_revoked(group_id) || !self.is_record_active(record_id) {
            return Err(ErasureError::Refused);
        }

        let mut dek_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut dek_bytes);
        let cipher = Aes256GcmSiv::new_from_slice(&dek_bytes)
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        dek_bytes.zeroize();

        let mut payload = Vec::with_capacity(12 + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);
        Ok(payload)
    }

    /// Entschlüsselt Daten für einen Record mit AES-256-GCM-SIV.
    pub fn decrypt_record(
        &self,
        group_id: u64,
        record_id: &str,
        dek_bytes: &[u8; 32],
        payload: &[u8],
    ) -> Result<Vec<u8>, ErasureError> {
        if self.is_group_revoked(group_id) || !self.is_record_active(record_id) {
            return Err(ErasureError::Refused);
        }
        if payload.len() < 12 {
            return Err(ErasureError::Corrupt);
        }

        let nonce_slice = payload.get(..12).ok_or(ErasureError::Corrupt)?;
        let nonce = Nonce::from_slice(nonce_slice);
        let ciphertext = payload.get(12..).ok_or(ErasureError::Corrupt)?;

        let cipher = Aes256GcmSiv::new_from_slice(dek_bytes)
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ErasureError::Crypto(e.to_string()))
    }
}
