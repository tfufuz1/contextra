//! Ed25519 report signing and verification (D-13, F.10).

use crate::error::{CryptoError, Result};
use crate::hierarchy::SignSeed;
use crate::labels;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};

/// Signs a canonical report message with prefix ctx/report/v1.
///
/// # Inputs
/// * `seed`: Strongly-typed `SignSeed` (32 bytes) derived from the master key.
/// * `canon`: Canonical report byte slice to be signed.
///
/// # Properties & Security
/// Prepends `labels::REPORT_PREFIX` (`ctx/report/v1`) to prevent cross-protocol signature
/// confusion. Generates a 64-byte Ed25519 signature.
pub fn sign_report(seed: &SignSeed, canon: &[u8]) -> [u8; 64] {
    let signing_key = SigningKey::from_bytes(seed.0.as_bytes());
    let mut msg = Vec::with_capacity(labels::REPORT_PREFIX.len() + canon.len());
    msg.extend_from_slice(labels::REPORT_PREFIX);
    msg.extend_from_slice(canon);
    signing_key.sign(&msg).to_bytes()
}

/// Verifies a canonical report signature against an Ed25519 public key using verify_strict (RFC 8032).
///
/// # Inputs
/// * `pubkey`: 32-byte Ed25519 public key.
/// * `canon`: Canonical report byte slice.
/// * `sig`: 64-byte Ed25519 signature to verify.
///
/// # Errors
/// Returns `Err(CryptoError::Auth)` if the public key is malformed or the signature does not match
/// under strict RFC 8032 verification (rejecting non-canonical / malleable signatures).
pub fn verify_report(pubkey: &[u8; 32], canon: &[u8], sig: &[u8; 64]) -> Result<()> {
    let verifying_key = VerifyingKey::from_bytes(pubkey).map_err(|_| CryptoError::Auth)?;
    let signature = Signature::from_bytes(sig);
    let mut msg = Vec::with_capacity(labels::REPORT_PREFIX.len() + canon.len());
    msg.extend_from_slice(labels::REPORT_PREFIX);
    msg.extend_from_slice(canon);
    verifying_key
        .verify_strict(&msg, &signature)
        .map_err(|_| CryptoError::Auth)
}
