//! Data Encryption Key (DEK) derivation per D-79 and L-3.

use crate::kdf::hkdf_expand;
use crate::secret::SecretKey;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct UnitKey(pub SecretKey<32>);

impl UnitKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(SecretKey::new(bytes))
    }
}

pub struct Dek(pub SecretKey<32>);

pub type Salt = [u8; 32];
pub type DocTag = [u8; 16];

pub struct AncestorSalts {
    pub t: Salt,
    pub c: Salt,
    pub s: Option<Salt>,
    pub d: Salt,
}

pub const DEK_LABEL: &[u8] = b"ctx/dek/v2";
pub const DEK_SALT_LABEL: &[u8] = b"ctx/dek-salt/v2";

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
