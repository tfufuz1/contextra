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
    /// Restricted to `pub(crate)` to prevent external production code from bypassing cryptographic
    /// verification and using unverified proofs as valid deletion assertions. Always verify the
    /// proof signature via [`.verify()`][UnverifiedProof::verify] before trusting its contents.
    #[allow(dead_code)]
    pub(crate) fn raw_unverified(&self) -> &DeletionProof {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deletion_proof::keys::DeletionProofKeyPair;
    use contextra_types::{DocId, TenantId, TxId};

    #[test]
    fn test_unverified_proof_raw_unverified_access() {
        let keypair = DeletionProofKeyPair::generate();
        let wrong_keypair = DeletionProofKeyPair::generate();

        let raw_proof = DeletionProof {
            signature_version: 3,
            scope: crate::DeletionScope::Document {
                doc_id: DocId(100),
                tenant_id: TenantId::SYSTEM,
            },
            deleted_keys_hash: [0xAA; 32],
            deleted_after_tx: TxId(1),
            timestamp: 1700000000,
            signature: vec![],
            covered_layers: vec![],
            uncovered_layers: vec![],
            excluded_scopes: vec![],
            graph_repair: vec![],
            wal_chain_receipt: None,
            audit_chain_position: None,
            integrity_warning: None,
        };

        let unverified = UnverifiedProof::new(raw_proof.clone());
        assert_eq!(unverified.raw_unverified(), &raw_proof);

        // Verification with wrong key fails (negative path)
        let failed = unverified.verify_external(wrong_keypair.verifying_key());
        assert!(matches!(failed, Err(CryptoError::InvalidProofSignature)));
    }
}
