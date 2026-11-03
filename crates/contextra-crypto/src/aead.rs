//! AEAD operations: Record Envelopes (F.6), Block AEAD (D-60, F.1), and Keyslot Wrap (F.2).

use crate::error::{CryptoError, Result};
use crate::hierarchy::{FileKey, FileKind, MasterKey};
use crate::labels;
use crate::secret::SecretKey;
use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
use rand::RngCore;
use zeroize::Zeroizing;

/// Keyslot AAD structure according to F.2.
#[derive(Debug, Clone, Copy)]
pub struct SlotAad {
    pub store_id: [u8; 16],
    pub slot_id: u8,
    pub anchor_id: u8,
    pub kdf_id: u8,
    pub m_kib: u32,
    pub t: u8,
    pub p: u8,
    pub slot_salt: [u8; 32],
}

impl SlotAad {
    pub fn to_bytes(self) -> Vec<u8> {
        let mut aad = Vec::with_capacity(labels::SLOT_AAD.len() + 16 + 3 + 4 + 2 + 32);
        aad.extend_from_slice(labels::SLOT_AAD);
        aad.extend_from_slice(&self.store_id);
        aad.push(self.slot_id);
        aad.push(self.anchor_id);
        aad.push(self.kdf_id);
        aad.extend_from_slice(&self.m_kib.to_be_bytes());
        aad.push(self.t);
        aad.push(self.p);
        aad.extend_from_slice(&self.slot_salt);
        aad
    }
}

/// Wraps the master key into a 48-byte ciphertext + tag using AES-256-GCM-SIV.
pub fn wrap_master(kek: &SecretKey<32>, aad: &SlotAad, nonce: [u8; 12], m: &MasterKey) -> [u8; 48] {
    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(kek.as_bytes()));
    let nonce_arr = Nonce::from_slice(&nonce);
    let aad_bytes = aad.to_bytes();
    let payload = Payload {
        msg: m.as_bytes(),
        aad: &aad_bytes,
    };
    // AES-GCM-SIV encryption with valid 32-byte key and 12-byte nonce cannot fail mathematically
    let Ok(ct) = cipher.encrypt(nonce_arr, payload) else {
        unreachable!("AES-256-GCM-SIV encrypt with valid key/nonce cannot fail")
    };
    let mut out = [0u8; 48];
    out.copy_from_slice(&ct);
    out
}

/// Unwraps the master key from a 48-byte ciphertext + tag. Fails with CryptoError::Auth on mismatch.
pub fn unwrap_master(
    kek: &SecretKey<32>,
    aad: &SlotAad,
    nonce: [u8; 12],
    w: &[u8; 48],
) -> Result<MasterKey> {
    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(kek.as_bytes()));
    let nonce_arr = Nonce::from_slice(&nonce);
    let aad_bytes = aad.to_bytes();
    let payload = Payload {
        msg: w,
        aad: &aad_bytes,
    };
    let pt = cipher
        .decrypt(nonce_arr, payload)
        .map_err(|_| CryptoError::Auth)?;
    if pt.len() != 32 {
        return Err(CryptoError::Auth);
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&pt);
    Ok(MasterKey::new(out))
}

/// Block data representation.
pub struct Block {
    pub ciphertext: Vec<u8>,
    pub tag: [u8; 16],
}

/// Block-AEAD sealer per D-60 and F.1.
pub struct BlockSealer {
    cipher: Aes256GcmSiv,
    kind: FileKind,
    file_id: [u8; 16],
    prev: Option<[u8; 32]>,
    next_index: u64,
}

impl BlockSealer {
    pub fn new(key: FileKey, kind: FileKind, file_id: [u8; 16], prev: Option<[u8; 32]>) -> Self {
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key.as_bytes()));
        Self {
            cipher,
            kind,
            file_id,
            prev,
            next_index: 0,
        }
    }

    pub fn set_counter_for_test(&mut self, val: u64) {
        self.next_index = val;
    }

    pub fn seal(&mut self, plain: &[u8]) -> Result<Block> {
        if self.next_index >= (1u64 << 32) {
            return Err(CryptoError::Limit);
        }

        // Nonce: 0x00000000 || u64(Blockindex)
        let mut nonce = [0u8; 12];
        if let Some(slot) = nonce.get_mut(4..12) {
            slot.copy_from_slice(&self.next_index.to_be_bytes());
        }

        // AAD: kind || file_id || u64(Blockindex) [|| prev]
        let mut aad = Vec::with_capacity(1 + 16 + 8 + 32);
        aad.push(self.kind as u8);
        aad.extend_from_slice(&self.file_id);
        aad.extend_from_slice(&self.next_index.to_be_bytes());
        if let Some(p) = self.prev {
            aad.extend_from_slice(&p);
        }

        let payload = Payload {
            msg: plain,
            aad: &aad,
        };

        let ct_with_tag = self
            .cipher
            .encrypt(Nonce::from_slice(&nonce), payload)
            .map_err(|_| CryptoError::Internal)?;

        if ct_with_tag.len() < 16 {
            return Err(CryptoError::Internal);
        }
        let (ct, tag) = ct_with_tag.split_at(ct_with_tag.len() - 16);
        let mut tag_arr = [0u8; 16];
        tag_arr.copy_from_slice(tag);

        self.next_index += 1;

        Ok(Block {
            ciphertext: ct.to_vec(),
            tag: tag_arr,
        })
    }

    pub fn header_tag(&self, header48: &[u8; 48]) -> [u8; 16] {
        // Nonce: 0xFFFFFFFF || u64(0)
        let mut nonce = [0u8; 12];
        if let Some(slot) = nonce.get_mut(0..4) {
            slot.copy_from_slice(&[0xFF; 4]);
        }
        let payload = Payload {
            msg: b"",
            aad: header48,
        };
        let Ok(out) = self.cipher.encrypt(Nonce::from_slice(&nonce), payload) else {
            unreachable!("header tag encryption with empty plaintext cannot fail")
        };
        let mut tag = [0u8; 16];
        tag.copy_from_slice(&out);
        tag
    }
}

/// Sealed record envelope according to F.6.
/// Trait for injecting nonces during tests, defaulting to OS CSPRNG.
pub trait NonceSource {
    fn fill_nonce(&mut self, dest: &mut [u8; 12]);
}

pub struct OsCsprng;

impl NonceSource for OsCsprng {
    fn fill_nonce(&mut self, dest: &mut [u8; 12]) {
        rand::rngs::OsRng.fill_bytes(dest);
    }
}

pub fn seal_record(
    dek: &crate::dek::Dek,
    doc: [u8; 16],
    key_bytes: &[u8],
    plain: &[u8],
) -> Vec<u8> {
    seal_record_with_nonce_source(dek, doc, key_bytes, plain, &mut OsCsprng)
}

pub fn seal_record_with_nonce_source<N: NonceSource>(
    dek: &crate::dek::Dek,
    doc: [u8; 16],
    key_bytes: &[u8],
    plain: &[u8],
    nonce_source: &mut N,
) -> Vec<u8> {
    let mut nonce = [0u8; 12];
    nonce_source.fill_nonce(&mut nonce);

    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let mut aad = Vec::with_capacity(labels::REC_AAD.len() + key_bytes.len());
    aad.extend_from_slice(labels::REC_AAD);
    aad.extend_from_slice(key_bytes);

    let payload = Payload {
        msg: plain,
        aad: &aad,
    };
    let Ok(ct) = cipher.encrypt(Nonce::from_slice(&nonce), payload) else {
        unreachable!("AES-256-GCM-SIV encrypt with valid key/nonce cannot fail")
    };

    let mut env = Vec::with_capacity(1 + 1 + 16 + 12 + ct.len());
    env.push(2); // version = 2
    env.push(1); // alg_id = 1
    env.extend_from_slice(&doc);
    env.extend_from_slice(&nonce);
    env.extend_from_slice(&ct);
    env
}

/// Opens a sealed record envelope according to F.6.
pub fn open_record(
    dek: &crate::dek::Dek,
    key_bytes: &[u8],
    env: &[u8],
) -> Result<Zeroizing<Vec<u8>>> {
    if env.len() < 1 + 1 + 16 + 12 + 16 {
        return Err(CryptoError::Corrupt);
    }
    if env.first().copied() != Some(2) {
        return Err(CryptoError::Corrupt);
    }
    if env.get(1).copied() != Some(1) {
        return Err(CryptoError::Corrupt);
    }

    let nonce = match env.get(18..30) {
        Some(n) => n,
        None => return Err(CryptoError::Corrupt),
    };
    let ct = match env.get(30..) {
        Some(c) => c,
        None => return Err(CryptoError::Corrupt),
    };

    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let mut aad = Vec::with_capacity(labels::REC_AAD.len() + key_bytes.len());
    aad.extend_from_slice(labels::REC_AAD);
    aad.extend_from_slice(key_bytes);

    let payload = Payload { msg: ct, aad: &aad };
    let pt = cipher
        .decrypt(Nonce::from_slice(nonce), payload)
        .map_err(|_| CryptoError::Auth)?;
    Ok(Zeroizing::new(pt))
}
