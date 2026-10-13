//! Wrapper for untrusted/unverified deletion proof data.

use super::keys::VerificationKey;
use super::proof::DeletionProof;
use crate::error::CryptoError;
use serde::{Deserialize, Serialize};

/// Wrapper around raw deletion proof data that has not yet been cryptographically verified.
///
/// `UnverifiedProof` prevents unverified proofs from being accidentally used as valid
/// deletion assertions. To convert an `UnverifiedProof` into a verified [`DeletionProof`],
/// you must invoke `.verify(...)` with the appropriate public verification key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnverifiedProof(DeletionProof);

impl UnverifiedProof {
    /// Wraps a raw `DeletionProof` instance into an `UnverifiedProof`.
    pub fn new(raw: DeletionProof) -> Self {
        Self(raw)
    }

    /// Returns a reference to the inner raw `DeletionProof` structure without verifying its signature.
    ///
    /// # Security Warning
    /// Using fields from an unverified proof can lead to false claims or spoofing attacks.
    /// Always verify the proof signature via [`.verify()`][UnverifiedProof::verify] before trusting
    /// its contents.
    pub fn raw_unverified(&self) -> &DeletionProof {
        &self.0
    }

    /// Verifies the cryptographic signature of the proof using the provided `VerificationKey`.
    ///
    /// On success, returns the inner verified [`DeletionProof`]. If verification fails,
    /// returns a [`CryptoError::InvalidProofSignature`].
    pub fn verify<'a>(
        &self,
        key: impl Into<VerificationKey<'a>>,
    ) -> std::result::Result<DeletionProof, CryptoError> {
        let is_valid = self
            .0
            .verify(key)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;

        if is_valid {
            Ok(self.0.clone())
        } else {
            Err(CryptoError::InvalidProofSignature)
        }
    }

    /// Verifies the cryptographic signature using ONLY the provided Ed25519 public key.
    ///
    /// On success, returns the inner verified [`DeletionProof`].
    pub fn verify_external(
        &self,
        public_key: &ed25519_dalek::VerifyingKey,
    ) -> std::result::Result<DeletionProof, CryptoError> {
        self.0.verify_external(public_key)?;
        Ok(self.0.clone())
    }
}

impl From<DeletionProof> for UnverifiedProof {
    fn from(proof: DeletionProof) -> Self {
        UnverifiedProof::new(proof)
    }
}
