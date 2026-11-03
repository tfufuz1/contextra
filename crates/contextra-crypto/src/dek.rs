//! Data Encryption Key (DEK) derivation per D-79 and L-3.

use crate::hierarchy::UnitKey;
use crate::kdf::hkdf_expand;
use crate::labels;
use crate::secret::SecretKey;
use sha2::{Digest, Sha256};

pub struct Dek(pub SecretKey<32>);

pub type Salt = [u8; 32];
pub type DocTag = [u8; 16];

pub struct AncestorSalts {
    pub t: Salt,
    pub c: Salt,
    pub s: Option<Salt>,
    pub d: Salt,
}

/// Derives a DEK using AncestorSalts and DocTag.
/// salt = SHA-256("ctx/dek-salt/v2" || salt_T || salt_C || salt_S || salt_D)
/// with salt_S = 0^32 when no subject is present.
pub fn derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: DocTag) -> Dek {
    let mut hasher = Sha256::new();
    hasher.update(labels::DEK_SALT);
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
    // HKDF with unit key as PRK, doc_tag as info
    let mut info = Vec::with_capacity(labels::DEK.len() + 32 + 16);
    info.extend_from_slice(labels::DEK);
    info.extend_from_slice(&salt);
    info.extend_from_slice(&doc);

    if hkdf_expand(unit.0.as_bytes(), &info, &mut out).is_err() {
        out.fill(0);
    }
    Dek(SecretKey::new(out))
}
