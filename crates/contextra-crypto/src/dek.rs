//! Data Encryption Key (DEK) derivation per D-79 and L-3.
//!
//! Enforces strict zeroization on drop for all key structures (`INV-CRYPTO-ZEROIZE-1`).

use crate::error::{CryptoError, Result};
use crate::kdf::hkdf_expand;
use crate::secret::SecretKey;
use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Key representing a storage or tenant unit.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct UnitKey(pub SecretKey<32>);

impl UnitKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(SecretKey::new(bytes))
    }

    pub fn expose_secret(&self) -> &[u8; 32] {
        self.0.expose_secret()
    }
}

/// Data Encryption Key for per-record / per-block encryption.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Dek(pub SecretKey<32>);

impl Dek {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(SecretKey::new(bytes))
    }

    pub fn expose_secret(&self) -> &[u8; 32] {
        self.0.expose_secret()
    }
}

pub type Salt = [u8; 32];
pub type DocTag = [u8; 16];

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct AncestorSalts {
    pub t: Salt,
    pub c: Salt,
    pub s: Option<Salt>,
    pub d: Salt,
}

pub const DEK_LABEL: &[u8] = b"ctx/dek/v2";
pub const DEK_SALT_LABEL: &[u8] = b"ctx/dek-salt/v2";
pub const WRAP_AAD: &[u8] = b"ctx/dek-wrap/v1";

/// Derives a DEK using AncestorSalts and DocTag.
pub fn derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: DocTag) -> Dek {
    let mut hasher = Sha256::new();
    hasher.update(DEK_SALT_LABEL);
    hasher.update(a.t);
    hasher.update(a.c);
    if let Some(s) = a.s {
        hasher.update(s);
    } else {
        hasher.update([0u8; 32]);
    }
    hasher.update(a.d);
    let salt: [u8; 32] = hasher.finalize().into();

    let mut out = [0u8; 32];
    let mut info = Vec::with_capacity(DEK_LABEL.len() + 32 + 16);
    info.extend_from_slice(DEK_LABEL);
    info.extend_from_slice(&salt);
    info.extend_from_slice(&doc);

    let _ = hkdf_expand(unit.0.as_bytes(), &info, &mut out);
    Dek(SecretKey::new(out))
}

/// Wraps a DEK using a Key Encryption Key (KEK / UnitKey) and AES-256-GCM-SIV.
pub fn wrap_dek(kek: &UnitKey, dek: &Dek, nonce: [u8; 12]) -> Result<Vec<u8>> {
    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(kek.0.as_bytes()));
    let payload = Payload {
        msg: dek.0.as_bytes(),
        aad: WRAP_AAD,
    };
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce), payload)
        .map_err(|e| CryptoError::Aead(format!("DEK wrap failed: {e}")))?;

    let mut wrapped = Vec::with_capacity(12 + ct.len());
    wrapped.extend_from_slice(&nonce);
    wrapped.extend_from_slice(&ct);
    Ok(wrapped)
}

/// Unwraps a wrapped DEK using a Key Encryption Key (KEK / UnitKey).
pub fn unwrap_dek(kek: &UnitKey, wrapped: &[u8]) -> Result<Dek> {
    if wrapped.len() < 12 + 16 {
        return Err(CryptoError::Aead("wrapped DEK payload too short".into()));
    }
    let (nonce, ct) = wrapped.split_at(12);
    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(kek.0.as_bytes()));
    let payload = Payload {
        msg: ct,
        aad: WRAP_AAD,
    };
    let pt = cipher
        .decrypt(Nonce::from_slice(nonce), payload)
        .map_err(|_| CryptoError::Aead("DEK unwrap failed: tag mismatch".into()))?;

    if pt.len() != 32 {
        return Err(CryptoError::InvalidLength(format!(
            "unwrapped key length mismatch: expected 32, got {}",
            pt.len()
        )));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&pt);
    Ok(Dek(SecretKey::new(key_bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dek_wrap_unwrap_roundtrip() {
        let kek = UnitKey::new([0x11u8; 32]);
        let dek = Dek::new([0x22u8; 32]);
        let nonce = [0x33u8; 12];

        let wrapped = wrap_dek(&kek, &dek, nonce).expect("wrap DEK");
        let unwrapped = unwrap_dek(&kek, &wrapped).expect("unwrap DEK");

        assert_eq!(unwrapped.expose_secret(), dek.expose_secret());
    }

    #[test]
    fn test_unwrap_dek_tampered_tag_fails() {
        let kek = UnitKey::new([0x11u8; 32]);
        let dek = Dek::new([0x22u8; 32]);
        let nonce = [0x33u8; 12];

        let mut wrapped = wrap_dek(&kek, &dek, nonce).expect("wrap DEK");
        let len = wrapped.len();
        wrapped[len - 1] ^= 0x01; // tamper tag

        let res = unwrap_dek(&kek, &wrapped);
        assert!(matches!(res, Err(CryptoError::Aead(_))));
    }
}
