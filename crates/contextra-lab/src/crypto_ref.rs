//! Unabhängige Krypto-Referenzimplementierung für das Erasure Lab.
//!
//! # Architektur-Rolle: Referenz-Primitive (AR-4, SPEC §3.1, §8.16)
//!
//! Stellt unabhängige Primitive-Aufrufe über externe, geprüfte Crates bereit:
//! - AES-256-GCM-SIV (RFC 8452)
//! - Argon2id KDF
//! - HKDF-SHA256
//! - HMAC-SHA256
//! - BLAKE3 Keyed MAC
//! - Ed25519 Signaturverifikation (verify_strict)
//!
//! Gemäß AR-4 und SPEC §8.16 werden keine Produktions-Decoder oder
//! Krypto-Implementierungen aus `contextra-crypto` importiert.

use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use ed25519_dalek::{Signature, VerifyingKey};
use hkdf::Hkdf;
use hmac::digest::Mac;
use hmac::Hmac;
use sha2::Sha256;

use crate::error::LabError;

/// Typ-Alias für HMAC-SHA256.
pub type HmacSha256 = Hmac<Sha256>;

/// Entschlüsselt einen AES-256-GCM-SIV Block mit 12-Byte Nonce und AAD.
pub fn aes_256_gcm_siv_decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    aad: &[u8],
    ciphertext_and_tag: &[u8],
) -> Result<Vec<u8>, LabError> {
    let cipher = Aes256GcmSiv::new_from_slice(key)
        .map_err(|_| LabError::Corrupted("Fehler bei AES-Schlüsselerzeugung"))?;
    let nonce = Nonce::from_slice(nonce);
    let payload = Payload {
        msg: ciphertext_and_tag,
        aad,
    };
    cipher
        .decrypt(nonce, payload)
        .map_err(|_| LabError::VerificationFailed("AES-GCM-SIV Entschlüsselung fehlgeschlagen"))
}

/// Verschlüsselt einen AES-256-GCM-SIV Block mit 12-Byte Nonce und AAD.
pub fn aes_256_gcm_siv_encrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, LabError> {
    let cipher = Aes256GcmSiv::new_from_slice(key)
        .map_err(|_| LabError::Corrupted("Fehler bei AES-Schlüsselerzeugung"))?;
    let nonce = Nonce::from_slice(nonce);
    let payload = Payload {
        msg: plaintext,
        aad,
    };
    cipher
        .encrypt(nonce, payload)
        .map_err(|_| LabError::VerificationFailed("AES-GCM-SIV Verschlüsselung fehlgeschlagen"))
}

/// Leitet einen Master-Schlüssel mittels Argon2id aus einer Passphrase ab.
pub fn derive_argon2id(
    passphrase: &[u8],
    salt: &[u8],
    m_cost_kib: u32,
    t_cost: u32,
    p_cost: u32,
    out: &mut [u8],
) -> Result<(), LabError> {
    let params = Params::new(m_cost_kib, t_cost, p_cost, Some(out.len()))
        .map_err(|_| LabError::Corrupted("Ungültige Argon2id-Parameter"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    argon2
        .hash_password_into(passphrase, salt, out)
        .map_err(|_| LabError::VerificationFailed("Argon2id KDF-Fehler"))
}

/// Leitet einen Unterschlüssel mittels HKDF-SHA256 ab.
pub fn derive_hkdf_sha256(
    ikm: &[u8],
    salt: Option<&[u8]>,
    info: &[u8],
    okm: &mut [u8],
) -> Result<(), LabError> {
    let hk = Hkdf::<Sha256>::new(salt, ikm);
    hk.expand(info, okm)
        .map_err(|_| LabError::Corrupted("HKDF Expand-Fehler"))
}

/// Berechnet einen BLAKE3-Keyed-MAC (16 Byte).
pub fn blake3_keyed_mac_16(key: &[u8; 32], data: &[u8]) -> [u8; 16] {
    let full = blake3::keyed_hash(key, data);
    let mut out = [0u8; 16];
    if let Some(prefix) = full.as_bytes().get(..16) {
        out.copy_from_slice(prefix);
    }
    out
}

/// Berechnet einen BLAKE3-Keyed-MAC (8 Byte) für Tresor-Slots.
pub fn blake3_keyed_mac_8(key: &[u8; 32], data: &[u8]) -> [u8; 8] {
    let full = blake3::keyed_hash(key, data);
    let mut out = [0u8; 8];
    if let Some(prefix) = full.as_bytes().get(..8) {
        out.copy_from_slice(prefix);
    }
    out
}

/// Berechnet einen HMAC-SHA256 Tag (32 Byte).
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<[u8; 32], LabError> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .map_err(|_| LabError::Corrupted("Ungültiger HMAC-Schlüssel"))?;
    mac.update(data);
    let result = mac.finalize();
    let bytes = result.into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// Verifiziert eine Ed25519-Signatur strikt nach RFC 8032.
pub fn verify_ed25519_strict(
    public_key_bytes: &[u8; 32],
    message: &[u8],
    signature_bytes: &[u8; 64],
) -> Result<(), LabError> {
    let vk = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|_| LabError::VerificationFailed("Ungültiger Ed25519 Public Key"))?;
    let sig = Signature::from_bytes(signature_bytes);
    vk.verify_strict(message, &sig)
        .map_err(|_| LabError::VerificationFailed("Ed25519 Signaturverifikation fehlgeschlagen"))
}
