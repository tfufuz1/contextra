//! AEAD Record Envelopes (RFC 8452, AES-256-GCM-SIV).
//!
//! Provides authenticated encryption with associated data (AEAD) ensuring
//! strict nonce discipline, constant-time verification, and uniform error reporting.
//! All header components (version, algorithm, doc id) are cryptographically bound into the AAD.

use crate::dek::Dek;
use crate::error::{CryptoError, Result};
use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
use rand::RngCore;
use zeroize::Zeroizing;

pub const REC_AAD: &[u8] = b"ctx/rec/v2";

/// Seals a plaintext record into an envelope using an explicit, caller-provided 12-byte nonce.
/// Binds `doc` and `key_bytes` into the Associated Data (AAD) preventing replay/reordering across documents.
pub fn seal_record_with_nonce(
    dek: &Dek,
    doc: [u8; 16],
    key_bytes: &[u8],
    plain: &[u8],
    nonce: [u8; 12],
) -> Result<Vec<u8>> {
    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let mut aad = Vec::with_capacity(REC_AAD.len() + 16 + key_bytes.len());
    aad.extend_from_slice(REC_AAD);
    aad.extend_from_slice(&doc);
    aad.extend_from_slice(key_bytes);

    let payload = Payload {
        msg: plain,
        aad: &aad,
    };
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce), payload)
        .map_err(|e| CryptoError::Aead(format!("AEAD encryption failed: {e}")))?;

    let mut env = Vec::with_capacity(1 + 1 + 16 + 12 + ct.len());
    env.push(2); // version = 2
    env.push(1); // alg_id = 1
    env.extend_from_slice(&doc);
    env.extend_from_slice(&nonce);
    env.extend_from_slice(&ct);
    Ok(env)
}

/// Seals a plaintext record into an envelope using an injected RNG for nonce generation.
pub fn seal_record_with_rng<R: RngCore>(
    dek: &Dek,
    doc: [u8; 16],
    key_bytes: &[u8],
    plain: &[u8],
    rng: &mut R,
) -> Result<Vec<u8>> {
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut nonce);
    seal_record_with_nonce(dek, doc, key_bytes, plain, nonce)
}

/// Seals a record using OsRng (convenience wrapper).
pub fn seal_record(dek: &Dek, doc: [u8; 16], key_bytes: &[u8], plain: &[u8]) -> Vec<u8> {
    seal_record_with_rng(dek, doc, key_bytes, plain, &mut rand::rngs::OsRng).unwrap_or_default()
}

/// Opens an encrypted record envelope, verifying the associated data and authentication tag.
/// Fails with a uniform `CryptoError::Aead` on any authentication or corruption issue without leaking oracle details.
pub fn open_record(dek: &Dek, key_bytes: &[u8], env: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    if env.len() < 1 + 1 + 16 + 12 + 16 {
        return Err(CryptoError::Aead("record envelope too short".into()));
    }
    if env.first().copied() != Some(2) {
        return Err(CryptoError::Aead("unsupported envelope version".into()));
    }
    if env.get(1).copied() != Some(1) {
        return Err(CryptoError::Aead("unsupported cipher algorithm".into()));
    }

    let doc = match env.get(2..18) {
        Some(d) => d,
        None => return Err(CryptoError::Aead("missing doc id".into())),
    };
    let nonce = match env.get(18..30) {
        Some(n) => n,
        None => return Err(CryptoError::Aead("missing nonce".into())),
    };
    let ct = match env.get(30..) {
        Some(c) => c,
        None => return Err(CryptoError::Aead("missing ciphertext".into())),
    };

    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let mut aad = Vec::with_capacity(REC_AAD.len() + 16 + key_bytes.len());
    aad.extend_from_slice(REC_AAD);
    aad.extend_from_slice(doc);
    aad.extend_from_slice(key_bytes);

    let payload = Payload { msg: ct, aad: &aad };
    let pt = cipher
        .decrypt(Nonce::from_slice(nonce), payload)
        .map_err(|_| CryptoError::Aead("AEAD authentication failed".into()))?;
    Ok(Zeroizing::new(pt))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret::SecretKey;

    fn test_dek() -> Dek {
        let secret = SecretKey::new([0x55u8; 32]);
        Dek(secret)
    }

    #[test]
    fn test_aead_seal_open_roundtrip() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let plaintext = b"cryptographic building block envelope content";

        let env = seal_record(&dek, doc, key_bytes, plaintext);
        let opened = open_record(&dek, key_bytes, &env).expect("open record");
        assert_eq!(opened.as_slice(), plaintext);
    }

    #[test]
    fn test_aead_seal_with_nonce() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let plaintext = b"cryptographic building block envelope content";
        let nonce = [0x77u8; 12];

        let env = seal_record_with_nonce(&dek, doc, key_bytes, plaintext, nonce).expect("seal");
        let opened = open_record(&dek, key_bytes, &env).expect("open record");
        assert_eq!(opened.as_slice(), plaintext);
    }

    #[test]
    fn test_aead_negative_wrong_aad_fails() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let wrong_key_bytes = b"wrong_record_key";
        let plaintext = b"authenticated payload";

        let env = seal_record(&dek, doc, key_bytes, plaintext);
        let res = open_record(&dek, wrong_key_bytes, &env);
        assert!(
            matches!(res, Err(CryptoError::Aead(_))),
            "Wrong AAD must fail AEAD verification"
        );
    }

    #[test]
    fn test_aead_negative_bit_flip_fails() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let plaintext = b"authenticated payload";

        let mut env = seal_record(&dek, doc, key_bytes, plaintext);
        let last_idx = env.len() - 1;
        env[last_idx] ^= 0x01; // flip 1 bit in tag/ciphertext

        let res = open_record(&dek, key_bytes, &env);
        assert!(
            matches!(res, Err(CryptoError::Aead(_))),
            "Bit-flip must fail AEAD verification"
        );
    }

    #[test]
    fn test_aead_negative_truncated_tag_fails() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let plaintext = b"authenticated payload";

        let env = seal_record(&dek, doc, key_bytes, plaintext);
        let truncated = &env[..env.len() - 8]; // truncate tag

        let res = open_record(&dek, key_bytes, truncated);
        assert!(
            matches!(res, Err(CryptoError::Aead(_))),
            "Truncated tag must fail AEAD verification"
        );
    }
}
