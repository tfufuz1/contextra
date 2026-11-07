//! AEAD Record Envelopes (F.6).

use crate::dek::Dek;
use crate::error::{CryptoError, Result};
use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
use zeroize::Zeroizing;

pub const REC_AAD: &[u8] = b"ctx/rec/v2";

pub fn seal_record(dek: &Dek, doc: [u8; 16], key_bytes: &[u8], plain: &[u8]) -> Vec<u8> {
    let mut nonce = [0u8; 12];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut nonce);

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
        .map_err(|e| CryptoError::Crypto(format!("AEAD authentication failed: {e}")))?;
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
    fn test_aead_negative_wrong_aad_fails() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let wrong_key_bytes = b"wrong_record_key";
        let plaintext = b"authenticated payload";

        let env = seal_record(&dek, doc, key_bytes, plaintext);
        let res = open_record(&dek, wrong_key_bytes, &env);
        assert!(res.is_err(), "Wrong AAD must fail AEAD verification");
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
        assert!(res.is_err(), "Bit-flip must fail AEAD verification");
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
        assert!(res.is_err(), "Truncated tag must fail AEAD verification");
    }
}
