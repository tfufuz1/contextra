//! WAL cryptographic primitives internal to contextra-store (SPEC ST-03, F.4, D-05).

use contextra_types::{ContextraError, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use hkdf::Hkdf;
use rand::RngCore;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

pub struct WalHmac {
    mac: Hmac<Sha256>,
}

impl WalHmac {
    pub fn new(integrity_key: &[u8]) -> Result<Self> {
        if integrity_key.is_empty() {
            return Err(ContextraError::InvalidInput("Key cannot be empty".to_string()));
        }
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(integrity_key)
            .map_err(|e| ContextraError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(b"contextra-wal-v1");
        Ok(Self { mac })
    }

    pub fn update(&mut self, data: &[u8]) {
        self.mac.update(data);
    }

    pub fn finalize(self) -> [u8; 32] {
        self.mac.finalize().into_bytes().into()
    }
}

#[derive(Debug, Clone)]
pub struct WalEntrySnapshot {
    pub tx_id: u64,
    pub seq_no: u64,
    pub op_type: u8,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub checksum: [u8; 32],
    pub prev_hmac: [u8; 32],
}

pub struct IntegrityVerifier {
    pub last_hmac: [u8; 32],
    pub integrity_key: Vec<u8>,
    pub last_seq_no: Option<u64>,
}

impl IntegrityVerifier {
    pub fn new(integrity_key: &[u8]) -> Self {
        Self {
            last_hmac: [0u8; 32],
            integrity_key: integrity_key.to_vec(),
            last_seq_no: None,
        }
    }

    pub fn set_last_hmac(&mut self, hmac: [u8; 32]) {
        self.last_hmac = hmac;
    }

    pub fn last_hmac_snapshot(&self) -> [u8; 32] {
        self.last_hmac
    }

    pub fn set_last_seq_no(&mut self, seq_no: Option<u64>) {
        self.last_seq_no = seq_no;
    }

    pub fn last_seq_no_snapshot(&self) -> Option<u64> {
        self.last_seq_no
    }

    pub fn verify_and_update_v3(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()> {
        if entry.op_type > 2 {
            return Err(ContextraError::wal_corruption(
                offset,
                format!("Unsupported op_type {}", entry.op_type),
            ));
        }

        if let Some(last_seq) = self.last_seq_no {
            if entry.seq_no < last_seq {
                return Err(ContextraError::wal_corruption(
                    offset,
                    format!(
                        "Non-monotonic sequence number {} (last: {})",
                        entry.seq_no, last_seq
                    ),
                ));
            }
        }

        let mut mac = WalHmac::new(&self.integrity_key)?;
        mac.update(&self.last_hmac);
        mac.update(&entry.seq_no.to_le_bytes());

        let tx_id_bytes = entry.tx_id.to_le_bytes();
        mac.update(&tx_id_bytes);

        if entry.op_type == 0 {
            mac.update(&[0u8]);
            let k_len = u32::try_from(entry.key.len())
                .map_err(|_| ContextraError::Serialization("Key length exceeds u32".into()))?;
            mac.update(&k_len.to_le_bytes());
            mac.update(&entry.key);
            let v_len = u32::try_from(entry.value.len())
                .map_err(|_| ContextraError::Serialization("Value length exceeds u32".into()))?;
            mac.update(&v_len.to_le_bytes());
            mac.update(&entry.value);
        } else if entry.op_type == 1 {
            mac.update(&[1u8]);
            let k_len = u32::try_from(entry.key.len())
                .map_err(|_| ContextraError::Serialization("Key length exceeds u32".into()))?;
            mac.update(&k_len.to_le_bytes());
            mac.update(&entry.key);
        } else {
            mac.update(&[2u8]);
            mac.update(&entry.value);
        }

        let computed = mac.finalize();
        if bool::from(!computed.ct_eq(&entry.checksum)) {
            return Err(ContextraError::wal_corruption(
                offset,
                "WAL entry HMAC checksum mismatch".to_string(),
            ));
        }

        self.last_hmac = entry.checksum;
        self.last_seq_no = Some(entry.seq_no);
        Ok(())
    }

    pub fn verify_and_update_v2(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()> {
        let mut mac = WalHmac::new(&self.integrity_key)?;
        mac.update(&self.last_hmac);
        mac.update(&entry.seq_no.to_le_bytes());
        mac.update(&[entry.op_type]);
        mac.update(&entry.key);
        mac.update(&entry.value);

        let computed = mac.finalize();
        if bool::from(!computed.ct_eq(&entry.checksum)) {
            return Err(ContextraError::wal_corruption(
                offset,
                "WAL entry HMAC checksum mismatch (v2)".to_string(),
            ));
        }

        self.last_hmac = entry.checksum;
        self.last_seq_no = Some(entry.seq_no);
        Ok(())
    }

    pub fn skip_hmac_verify_legacy(&mut self, entry: &WalEntrySnapshot) {
        self.last_hmac = entry.checksum;
        self.last_seq_no = Some(entry.seq_no);
    }
}

pub fn verify_wal_chain_completeness(
    wal_tail_hmac: &[u8; 32],
    manifest_high_water_mark: &[u8; 32],
) -> Result<()> {
    if bool::from(wal_tail_hmac.ct_eq(manifest_high_water_mark)) {
        Ok(())
    } else {
        Err(ContextraError::wal_truncation_detected(
            *manifest_high_water_mark,
            *wal_tail_hmac,
        ))
    }
}

pub struct KeyManager {
    key: [u8; 32],
    cipher_cache: OnceLock<Aes256GcmSiv>,
    nonce_prefix: [u8; 4],
    nonce_counter: AtomicU64,
}

impl std::fmt::Debug for KeyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyManager")
            .field("key", &"***REDACTED***")
            .finish()
    }
}

impl KeyManager {
    pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self> {
        let hk = Hkdf::<Sha256>::new(Some(salt), passphrase.as_bytes());
        let mut key_raw = [0u8; 32];
        hk.expand(b"contextra-aes-256-gcm-key", &mut key_raw)
            .map_err(|e| ContextraError::Internal(format!("HKDF expansion failed: {}", e)))?;

        let cipher = Aes256GcmSiv::new_from_slice(&key_raw)
            .map_err(|e| ContextraError::Internal(format!("Aes256GcmSiv init failed: {}", e)))?;
        let cipher_cache = OnceLock::new();
        let _ = cipher_cache.set(cipher);

        let mut nonce_prefix = [0u8; 4];
        rand::rngs::OsRng.fill_bytes(&mut nonce_prefix);

        Ok(Self {
            key: key_raw,
            cipher_cache,
            nonce_prefix,
            nonce_counter: AtomicU64::new(1),
        })
    }

    pub fn derive_file_key(&self, file_id: &[u8]) -> Result<Self> {
        let hk = Hkdf::<Sha256>::from_prk(&self.key)
            .map_err(|_| ContextraError::Internal("Invalid PRK length".to_string()))?;

        let mut sub_key = [0u8; 32];
        let mut info = Vec::with_capacity(b"contextra-file-key-v1:".len() + 4 + file_id.len());
        info.extend_from_slice(b"contextra-file-key-v1:");
        let f_len = u32::try_from(file_id.len())
            .map_err(|_| ContextraError::Serialization("file_id length exceeds u32".into()))?;
        info.extend_from_slice(&f_len.to_le_bytes());
        info.extend_from_slice(file_id);

        hk.expand(&info, &mut sub_key)
            .map_err(|e| ContextraError::Internal(format!("HKDF sub-key expansion failed: {}", e)))?;

        let cipher = Aes256GcmSiv::new_from_slice(&sub_key)
            .map_err(|e| ContextraError::Internal(format!("Aes256GcmSiv key init failed: {}", e)))?;
        let cipher_cache = OnceLock::new();
        let _ = cipher_cache.set(cipher);

        let mut nonce_prefix = [0u8; 4];
        rand::rngs::OsRng.fill_bytes(&mut nonce_prefix);

        Ok(Self {
            key: sub_key,
            cipher_cache,
            nonce_prefix,
            nonce_counter: AtomicU64::new(1),
        })
    }

    pub fn integrity_key(&self) -> Result<[u8; 32]> {
        let hk = Hkdf::<Sha256>::from_prk(&self.key)
            .map_err(|_| ContextraError::Internal("Invalid PRK length".to_string()))?;
        let mut key = [0u8; 32];
        hk.expand(b"contextra-hmac-sha256-key", &mut key)
            .map_err(|e| ContextraError::Internal(format!("HKDF integrity expansion failed: {}", e)))?;
        Ok(key)
    }

    fn cipher(&self) -> Result<&Aes256GcmSiv> {
        if let Some(c) = self.cipher_cache.get() {
            return Ok(c);
        }
        let cipher = Aes256GcmSiv::new_from_slice(&self.key)
            .map_err(|e| ContextraError::Internal(format!("Aes256GcmSiv init failed: {}", e)))?;
        let _ = self.cipher_cache.set(cipher);
        self.cipher_cache
            .get()
            .ok_or_else(|| ContextraError::Internal("Failed to get cipher".into()))
    }

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
                ContextraError::Internal(
                    "Nonce counter exhausted".to_string(),
                )
            })?;

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&self.nonce_prefix);
        nonce_bytes[4..12].copy_from_slice(&counter_val.to_be_bytes());

        let cipher = self.cipher()?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| ContextraError::Internal(format!("Encryption failed: {}", e)))?;

        Ok((ciphertext, nonce_bytes))
    }

    pub fn decrypt_auto_nonce(&self, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>> {
        let cipher = self.cipher()?;
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ContextraError::Internal(format!("Decryption failed: {}", e)))
    }
}
