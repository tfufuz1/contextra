//! Ed25519 report signing and verification (D-13, F.10).

use crate::error::{CryptoError, Result};
use crate::hierarchy::SignSeed;
use crate::labels;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};

/// Signer ID constants according to SPEC F.10 and D-130:
/// 0 = derived from K_sign (default)
/// 1 = operator / external HSM signer
pub const SIGNER_ID_DERIVED: u8 = 0;
pub const SIGNER_ID_OPERATOR: u8 = 1;

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

/// Public report signer contract according to SPEC D-80 and D-130.
///
/// Provides abstract or concrete report signing capability with zero panic guarantee.
pub trait ReportSigner: Send + Sync {
    /// Returns the signer identifier (0 for derived K_sign, 1 for external).
    fn signer_id(&self) -> u8;

    /// Computes the 32-byte Ed25519 public key / verifying key bytes.
    fn public_key(&self) -> [u8; 32];

    /// Signs the canonical report bytes using Ed25519 with domain separation prefix `ctx/report/v1`.
    fn sign_canonical(&self, canon: &[u8]) -> Result<[u8; 64]>;
}

/// Standard implementation of `ReportSigner` using derived `SignSeed` (SPEC D-80, D-130).
pub struct DerivedReportSigner {
    seed: SignSeed,
    pubkey: [u8; 32],
}

impl DerivedReportSigner {
    /// Constructs a new `DerivedReportSigner` from derived `SignSeed`.
    pub fn new(seed: SignSeed) -> Self {
        let signing_key = SigningKey::from_bytes(seed.0.as_bytes());
        let pubkey = signing_key.verifying_key().to_bytes();
        Self { seed, pubkey }
    }
}

impl ReportSigner for DerivedReportSigner {
    fn signer_id(&self) -> u8 {
        SIGNER_ID_DERIVED
    }

    fn public_key(&self) -> [u8; 32] {
        self.pubkey
    }

    fn sign_canonical(&self, canon: &[u8]) -> Result<[u8; 64]> {
        Ok(sign_report(&self.seed, canon))
    }
}

/// Verifies a canonical report signature against an Ed25519 public key using verify_strict (RFC 8032).
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
