//! AEAD Record Envelopes (RFC 8452, AES-256-GCM-SIV).

use crate::dek::Dek;
use crate::error::{CryptoError, Result};
use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
use zeroize::Zeroizing;

pub const REC_AAD: &[u8] = b"ctx/rec/v2";

pub trait CryptoRng {
    fn fill_bytes(&mut self, dest: &mut [u8]);
}

impl<T: rand::RngCore> CryptoRng for T {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand::RngCore::fill_bytes(self, dest);
    }
}

pub fn seal_record_with_rng<R: CryptoRng>(
    dek: &Dek,
    doc: [u8; 16],
    key_bytes: &[u8],
    plain: &[u8],
    rng: &mut R,
) -> Vec<u8> {
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut nonce);
    seal_record_with_nonce(dek, doc, key_bytes, plain, nonce)
}

pub fn seal_record_with_nonce(
    dek: &Dek,
    doc: [u8; 16],
    key_bytes: &[u8],
    plain: &[u8],
    nonce: [u8; 12],
) -> Vec<u8> {
    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let mut aad = Vec::with_capacity(REC_AAD.len() + key_bytes.len());
    aad.extend_from_slice(REC_AAD);
    aad.extend_from_slice(key_bytes);

    let payload = Payload {
        msg: plain,
        aad: &aad,
    };
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce), payload)
        .unwrap_or_default();

    let mut env = Vec::with_capacity(1 + 1 + 16 + 12 + ct.len());
    env.push(2); // version = 2
    env.push(1); // alg_id = 1
    env.extend_from_slice(&doc);
    env.extend_from_slice(&nonce);
    env.extend_from_slice(&ct);
    env
}

pub fn seal_record(dek: &Dek, doc: [u8; 16], key_bytes: &[u8], plain: &[u8]) -> Vec<u8> {
    let mut rng = rand::rngs::OsRng;
    seal_record_with_rng(dek, doc, key_bytes, plain, &mut rng)
}

pub fn open_record(dek: &Dek, key_bytes: &[u8], env: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    if env.len() < 1 + 1 + 16 + 12 + 16 {
        return Err(CryptoError::Crypto("record envelope too short".into()));
    }
    if env.first().copied() != Some(2) {
        return Err(CryptoError::Crypto("unsupported envelope version".into()));
    }
    if env.get(1).copied() != Some(1) {
        return Err(CryptoError::Crypto("unsupported cipher algorithm".into()));
    }

    let nonce = match env.get(18..30) {
        Some(n) => n,
        None => return Err(CryptoError::Crypto("missing nonce".into())),
    };
    let ct = match env.get(30..) {
        Some(c) => c,
        None => return Err(CryptoError::Crypto("missing ciphertext".into())),
    };

    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let mut aad = Vec::with_capacity(REC_AAD.len() + key_bytes.len());
    aad.extend_from_slice(REC_AAD);
    aad.extend_from_slice(key_bytes);

    let payload = Payload { msg: ct, aad: &aad };
    let pt = cipher
        .decrypt(Nonce::from_slice(nonce), payload)
        .map_err(|_| CryptoError::IntegrityViolation)?;
    Ok(Zeroizing::new(pt))
}
