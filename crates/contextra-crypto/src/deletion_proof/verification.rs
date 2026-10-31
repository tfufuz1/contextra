//! # Verifikation von Löschbeweisen (`deletion_proof/verification.rs`)
//!
//! Prüft kryptographische Signaturen und BLAKE3-Digests fail-closed.
// FILE-CONTEXT
// ZWECK: Verification logic for cryptographic deletion proofs (v1, v2, v3, v5, WAL receipt verification).
// INVARIANTEN: Constant-time comparison for secret/HMAC verification; Fail-closed on missing/mismatched keys.

#![forbid(unsafe_code)]

use crate::deletion_proof::creation::compute_hmac_sha256;
use crate::deletion_proof::types::*;
use crate::ed25519_proof::{DeletionProofError, SignatureVersion};
use crate::error::CryptoError;
use contextra_types::{ContextraError, Result, TenantId};
use subtle::ConstantTimeEq;

impl DeletionProof {
    /// Typed accessor for the signature version.
    pub fn signature_version_typed(
        &self,
    ) -> std::result::Result<SignatureVersion, DeletionProofError> {
        SignatureVersion::try_from(self.signature_version)
    }

    /// Accessor for the TenantId of this proof.
    pub fn tenant_id(&self) -> TenantId {
        match &self.scope {
            DeletionScope::Document { tenant_id, .. } => *tenant_id,
            DeletionScope::Collection { tenant_id, .. } => *tenant_id,
            DeletionScope::Tenant { tenant_id } => *tenant_id,
        }
    }

    /// Verifies the signature of this DeletionProof against a provided key.
    pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool> {
        let v_key = key.into();
        let version = self
            .signature_version_typed()
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        match (version, v_key) {
            (SignatureVersion::V1, VerificationKey::Hmac(k)) => {
                let scope_bytes = bincode::serialize(&self.scope)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                let tx_bytes = self.deleted_after_tx.0.to_le_bytes();
                let expected =
                    compute_hmac_sha256(k, &[&scope_bytes, &self.deleted_keys_hash, &tx_bytes])?;
                Ok(bool::from(self.signature.ct_eq(&expected)))
            }
            (SignatureVersion::V2, VerificationKey::Hmac(k)) => {
                let scope_bytes = bincode::serialize(&self.scope)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                let tx_bytes = self.deleted_after_tx.0.to_le_bytes();
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

                let expected = compute_hmac_sha256(
                    k,
                    &[
                        &scope_bytes,
                        &self.deleted_keys_hash,
                        &tx_bytes,
                        &covered_layers_bytes,
                        &excluded_scopes_bytes,
                        receipt_part,
                    ],
                )?;
                Ok(bool::from(self.signature.ct_eq(&expected)))
            }
            (SignatureVersion::V3, VerificationKey::Ed25519(vk)) => {
                let payload = self
                    .construct_v3_payload()
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                use ed25519_dalek::Verifier;
                let sig = match ed25519_dalek::Signature::from_slice(&self.signature) {
                    Ok(s) => s,
                    Err(_) => return Ok(false),
                };
                Ok(vk.verify(&payload, &sig).is_ok())
            }
            (SignatureVersion::V5, VerificationKey::Ed25519(vk)) => {
                let payload = self
                    .construct_v5_payload()
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                use ed25519_dalek::Verifier;
                let sig = match ed25519_dalek::Signature::from_slice(&self.signature) {
                    Ok(s) => s,
                    Err(_) => return Ok(false),
                };
                Ok(vk.verify(&payload, &sig).is_ok())
            }
            (SignatureVersion::V5, VerificationKey::Hmac(k)) => {
                let payload = self
                    .construct_v5_payload()
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                let expected = compute_hmac_sha256(k, &[&payload])?;
                Ok(bool::from(self.signature.ct_eq(&expected)))
            }
            _ => Ok(false),
        }
    }

    /// Verifies this proof using ONLY the provided Ed25519 public key.
    pub fn verify_external(
        &self,
        public_key: &ed25519_dalek::VerifyingKey,
    ) -> std::result::Result<(), CryptoError> {
        let version = self
            .signature_version_typed()
            .map_err(|_| CryptoError::UnsupportedProofVersion(self.signature_version))?;

        let payload = match version {
            SignatureVersion::V3 => self.construct_v3_payload()?,
            SignatureVersion::V5 => self.construct_v5_payload()?,
            _ => return Err(CryptoError::UnsupportedProofVersion(self.signature_version)),
        };

        use ed25519_dalek::Verifier;
        let sig = ed25519_dalek::Signature::from_slice(&self.signature)
            .map_err(|_| CryptoError::InvalidProofSignature)?;

        public_key
            .verify(&payload, &sig)
            .map_err(|_| CryptoError::InvalidProofSignature)
    }

    /// Formats the deletion proof into a JSON audit string including legal coverage bounds.
    pub fn export_for_audit(&self) -> Result<String> {
        let version = self
            .signature_version_typed()
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        if version == SignatureVersion::V1 {
            let mut clone = self.clone();
            clone.integrity_warning = Some(
                "covered_layers/excluded_scopes are not cryptographically signed in this legacy proof version"
                    .to_string(),
            );
            serde_json::to_string_pretty(&clone)
                .map_err(|e| ContextraError::Internal(e.to_string()))
        } else if version == SignatureVersion::V2 || version == SignatureVersion::V3 {
            let mut clone = self.clone();
            clone.integrity_warning = Some(
                "covered_layers/excluded_scopes are not bound to a verified witness measurement in this legacy proof version"
                    .to_string(),
            );
            serde_json::to_string_pretty(&clone)
                .map_err(|e| ContextraError::Internal(e.to_string()))
        } else {
            serde_json::to_string_pretty(self).map_err(|e| ContextraError::Internal(e.to_string()))
        }
    }
}

/// Verifies a WAL delete receipt HMAC over previous HMAC, delete event, and integrity key.
pub fn verify_wal_delete_receipt(
    receipt: &[u8; 32],
    prev_hmac: &[u8; 32],
    delete_event_payload: &[u8],
    integrity_key: &[u8],
) -> Result<bool> {
    use subtle::ConstantTimeEq;
    let expected = crate::deletion_proof::creation::compute_wal_delete_receipt(
        prev_hmac,
        delete_event_payload,
        integrity_key,
    )?;
    Ok(expected.ct_eq(receipt).into())
}
