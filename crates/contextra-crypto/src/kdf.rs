//! Argon2id key derivation and HKDF expand functions.

use crate::error::{CryptoError, Result};
use crate::secret::{Secret, SecretKey};
use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use sha2::Sha256;

/// KDF parameter bounds per D-06, D-68, FM-16.
pub const MIN_M_KIB: u32 = 65536; // 64 MiB
pub const MAX_M_KIB: u32 = 4194304; // 4 GiB
pub const MIN_T: u8 = 3;
pub const MAX_T: u8 = 16;
pub const MIN_P: u8 = 1;
pub const MAX_P: u8 = 16;

/// Minimum passphrase length in bytes according to SPEC §5.8 / Table 852.
pub const MIN_PASSPHRASE_BYTES: usize = 12;

/// Argon2id parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u8,
    pub p: u8,
}

impl KdfParams {
    /// Standard parameters per D-68: m = 128 MiB (131072 KiB), t = 3, p = 4.
    pub const DEFAULT: Self = Self {
        m_kib: 131072,
        t: 3,
        p: 4,
    };

    /// Validate and construct from header input (fails with Corrupt).
    pub fn from_header(m_kib: u32, t: u8, p: u8) -> Result<Self> {
        if !(MIN_M_KIB..=MAX_M_KIB).contains(&m_kib)
            || !(MIN_T..=MAX_T).contains(&t)
            || !(MIN_P..=MAX_P).contains(&p)
        {
            return Err(CryptoError::Corrupt);
        }
        Ok(Self { m_kib, t, p })
    }

    /// Validate and construct from config input (fails with Config).
    pub fn from_config(m_kib: u32, t: u8, p: u8) -> Result<Self> {
        if !(MIN_M_KIB..=MAX_M_KIB).contains(&m_kib)
            || !(MIN_T..=MAX_T).contains(&t)
            || !(MIN_P..=MAX_P).contains(&p)
        {
            return Err(CryptoError::Config);
        }
        Ok(Self { m_kib, t, p })
    }

    /// Generic new constructor for compatibility (defaults to Config error).
    pub fn new(m_kib: u32, t: u8, p: u8) -> Result<Self> {
        Self::from_config(m_kib, t, p)
    }
}

/// Derives a Key Encryption Key (KEK) using Argon2id.
///
/// Enforces `pass.as_bytes().len() >= MIN_PASSPHRASE_BYTES` (12 bytes) before
/// triggering the expensive Argon2id derivation to fail fast with `CryptoError::Config`.
pub fn derive_kek(pass: &Secret, salt: &[u8; 32], params: &KdfParams) -> Result<SecretKey<32>> {
    if pass.as_bytes().len() < MIN_PASSPHRASE_BYTES {
        return Err(CryptoError::Config);
    }

    let argon2_params = Params::new(
        params.m_kib,
        u32::from(params.t),
        u32::from(params.p),
        Some(32),
    )
    .map_err(|_| CryptoError::Config)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(pass.as_bytes(), salt, &mut key)
        .map_err(|_| CryptoError::Internal)?;
    Ok(SecretKey::new(key))
}

/// HKDF-SHA256 expand.
pub fn hkdf_expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> Result<()> {
    let hk = Hkdf::<Sha256>::from_prk(prk).map_err(|_| CryptoError::Internal)?;
    hk.expand(info, out).map_err(|_| CryptoError::Internal)?;
    Ok(())
}
