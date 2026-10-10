//! Deletion proof signature and WAL receipt verification logic.

use super::keys::VerificationKey;
use super::proof::DeletionProof;
use super::wal_receipt::{compute_hmac_sha256, verify_wal_delete_receipt};
use crate::ed25519_proof::{DeletionProofError, SignatureVersion};
use crate::error::CryptoError;
use contextra_types::{ContextraError, Result};

impl DeletionProof {
    /// Parst und liefert die typsichere Signaturversion des Beweises.
    ///
    /// # Errors
    /// Gibt [`DeletionProofError::UnsupportedVersion`] zurück, wenn `signature_version` einen ungültigen/unbekannten Wert enthält.
    pub fn signature_version_typed(
        &self,
    ) -> std::result::Result<SignatureVersion, DeletionProofError> {
        SignatureVersion::try_from(self.signature_version)
    }

    /// Verifiziert Signatur.
    /// Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1.
    /// NICHT-GARANTIE: Prüft nur Signatur, nicht ob Storage tatsächlich bereinigt ist.
    pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool> {
        let version = self
            .signature_version_typed()
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        let key = key.into();
        let scope_bytes =
            bincode::serialize(&self.scope).map_err(|e| ContextraError::Internal(e.to_string()))?;
        let tx_bytes = self.deleted_after_tx.0.to_le_bytes();

        match version {
            SignatureVersion::V1 => {
                let proof_key = match key {
                    VerificationKey::Hmac(k) | VerificationKey::HmacV1(k) | VerificationKey::HmacV2(k) => k,
                    VerificationKey::Ed25519(_) => {
                        return Err(ContextraError::Internal(
                            "Ed25519 key provided for HMAC signature_version 1 proof".to_string(),
                        ))
                    }
                };
                let expected = compute_hmac_sha256(
                    proof_key,
                    &[&scope_bytes, &self.deleted_keys_hash, &tx_bytes],
                )?;
                use subtle::ConstantTimeEq;
                Ok(expected.as_slice().ct_eq(&self.signature).into())
            }
            SignatureVersion::V2 => {
                let proof_key = match key {
                    VerificationKey::Hmac(k) | VerificationKey::HmacV1(k) | VerificationKey::HmacV2(k) => k,
                    VerificationKey::Ed25519(_) => {
                        return Err(ContextraError::Internal(
                            "Ed25519 key provided for HMAC signature_version 2 proof".to_string(),
                        ))
                    }
                };
                use subtle::ConstantTimeEq;
                let full_payload = self.construct_v2_full_payload()?;
                let expected_full = compute_hmac_sha256(proof_key, &[&full_payload])?;
                if expected_full.as_slice().ct_eq(&self.signature).into() {
                    return Ok(true);
                }

                // Alt-Verifikation: Fallback für früher erzeugte v2-Proofs
                let covered_layers_bytes = bincode::serialize(&self.covered_layers)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                let excluded_scopes_bytes = bincode::serialize(&self.excluded_scopes)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                let receipt_bytes = self.wal_chain_receipt.unwrap_or([0u8; 32]);
                let receipt_part = if self.wal_chain_receipt.is_some() {
                    receipt_bytes.as_slice()
                } else {
                    &[]
                };
                let expected_legacy = compute_hmac_sha256(
                    proof_key,
                    &[
                        &scope_bytes,
                        &self.deleted_keys_hash,
                        &tx_bytes,
                        &covered_layers_bytes,
                        &excluded_scopes_bytes,
                        receipt_part,
                    ],
                )?;
                Ok(expected_legacy.as_slice().ct_eq(&self.signature).into())
            }
            SignatureVersion::V3 => {
                let verifying_key = match key {
                    VerificationKey::Ed25519(vk) => vk,
                    VerificationKey::Hmac(_) | VerificationKey::HmacV1(_) | VerificationKey::HmacV2(_) => {
                        return Err(ContextraError::Internal(
                            "HMAC key provided for Ed25519 signature_version 3 proof".to_string(),
                        ))
                    }
                };
                let payload = self
                    .construct_v3_payload()
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;

                use ed25519_dalek::Verifier;
                let sig = match ed25519_dalek::Signature::from_slice(&self.signature) {
                    Ok(s) => s,
                    Err(_) => return Ok(false),
                };

                Ok(verifying_key.verify(&payload, &sig).is_ok())
            }
        }
    }

    /// Verifies this proof against a sequence of historical verification keys.
    ///
    /// Evaluates the proof against each key in `keys` sequentially. Returns `Ok(true)` on the first
    /// key that successfully verifies the proof signature. Returns `Ok(false)` if none of the provided
    /// keys match.
    ///
    /// # Key Rotation & Migration to Version 3
    /// Legacy symmetric HMAC proofs (version 1 and version 2) depend on secret key material derived from
    /// the master key. Following a master key rotation, verifying a legacy HMAC proof against only the new
    /// master key will return `Ok(false)`. This method enables callers to supply historical verification
    /// keys alongside the active key during transition periods.
    ///
    /// To permanently avoid key rotation invalidation for audit proofs, migrate to asymmetric version 3
    /// Ed25519 proofs ([`DeletionProof::create_v3`]), where proof verification relies solely on the public
    /// key and is invariant under master key rotations.
    pub fn verify_with_key_history<'a>(&self, keys: &[VerificationKey<'a>]) -> Result<bool> {
        for key in keys {
            if let Ok(true) = self.verify(*key) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Verifies this proof using ONLY the provided Ed25519 public key.
    /// Does NOT require access to any `KeyManager` state — this is the
    /// verification path intended for third parties who received a
    /// `DeletionProof` and the corresponding public key out-of-band.
    ///
    /// # Differences from [`verify`][Self::verify]
    /// - [`verify`][Self::verify] supports legacy HMAC proof versions (v1 and v2) requiring symmetric key access,
    ///   as well as v3 Ed25519 proofs via [`VerificationKey`].
    /// - `verify_external` operates strictly on `signature_version == 3` (Ed25519 asymmetric signatures) and
    ///   requires zero access to secret key material or internal `KeyManager` state. It returns explicit
    ///   [`CryptoError`] types ([`CryptoError::UnsupportedProofVersion`] for v1/v2, [`CryptoError::InvalidProofSignature`]
    ///   for invalid signatures or malformed signature bytes).
    ///
    /// # Cryptographic Verification Standard & Constant-Time Security
    /// Unlike version 1 and version 2 proofs which rely on symmetric HMAC-SHA256 checksums
    /// compared via [`subtle::ConstantTimeEq`], version 3 proof signature verification relies on
    /// Ed25519 asymmetric signature verification ([`ed25519_dalek::Verifier::verify`]).
    ///
    /// Ed25519 verification is an algebraic check on Curve25519 (`[S]B = R + [k]A` in curve point
    /// arithmetic), where scalar multiplication and group operations determine validity.
    /// Replacing curve point verification with a byte equality comparison (`ConstantTimeEq`) on
    /// signature bytes would be mathematically incorrect and cryptographic nonsense, because
    /// there is no "expected signature byte array" known a priori without solving the discrete logarithm problem.
    ///
    /// The `ed25519_dalek` implementation of `Verifier::verify` handles curve arithmetic in constant time
    /// where necessary to prevent timing side channels during public key verification.
    ///
    /// # Errors
    /// - [`CryptoError::UnsupportedProofVersion`] if `signature_version` is not 3.
    /// - [`CryptoError::InvalidProofSignature`] if the signature does not match or cannot be parsed.
    pub fn verify_external(
        &self,
        public_key: &ed25519_dalek::VerifyingKey,
    ) -> std::result::Result<(), CryptoError> {
        // Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1
        // Verzweigung ausschließlich über den typisierten SignatureVersion-Wert.
        let version = self
            .signature_version_typed()
            .map_err(|_| CryptoError::UnsupportedProofVersion(self.signature_version))?;

        if version != SignatureVersion::V3 {
            return Err(CryptoError::UnsupportedProofVersion(self.signature_version));
        }

        let payload = self.construct_v3_payload()?;

        use ed25519_dalek::Verifier;
        let sig = ed25519_dalek::Signature::from_slice(&self.signature)
            .map_err(|_| CryptoError::InvalidProofSignature)?;

        public_key
            .verify(&payload, &sig)
            .map_err(|_| CryptoError::InvalidProofSignature)
    }

    /// Verifiziert die in diesem Beweis enthaltene WAL-Löschquittung (falls vorhanden) gegen das WAL-Event.
    pub fn verify_wal_receipt(
        &self,
        prev_hmac: &[u8; 32],
        delete_event_payload: &[u8],
        integrity_key: &[u8],
    ) -> Result<bool> {
        if let Some(ref receipt) = self.wal_chain_receipt {
            verify_wal_delete_receipt(receipt, prev_hmac, delete_event_payload, integrity_key)
        } else {
            Ok(false)
        }
    }
}
