// FILE-CONTEXT
// ZWECK: Public facade for third-party DeletionProof and AuditChain verification without DB access.
// INVARIANTEN: Zero DB state requirement; structured error reporting with explicit failure reasons.

use crate::deletion_proof::types::DeletionProof;
use contextra_types::TenantId;
use serde::{Deserialize, Serialize};

/// Detailed failure reasons for third-party verification of deletion proofs and audit chains.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationErrorReason {
    /// Digital signature is invalid or tampered with.
    InvalidSignature(String),
    /// Unbroken chain invariant violated (hash mismatch or missing predecessor).
    ChainBroken {
        /// Position in chain where break occurred.
        position: usize,
        /// Description of the break.
        details: String,
    },
    /// Audit chain head signature or head hash does not match expected anchor.
    HeadMismatch {
        /// Expected head hash.
        expected_head_hash: [u8; 32],
        /// Actual head hash found in chain.
        actual_head_hash: [u8; 32],
    },
    /// Unsupported signature version or proof format.
    UnsupportedVersion(u8),
    /// Chain has been truncated or is missing trailing entries without anchor update.
    TruncatedChain {
        /// Expected minimum chain length.
        expected_len: usize,
        /// Actual chain length.
        actual_len: usize,
    },
    /// Tenant mismatch in proof vs expected context.
    TenantMismatch {
        /// Expected tenant ID.
        expected: TenantId,
        /// Actual tenant ID in proof.
        actual: TenantId,
    },
    /// Provided chain is empty.
    EmptyChain,
    /// Invalid or malformed public verification key.
    InvalidKey(String),
}

/// Structured outcome of third-party deletion proof verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationResult {
    /// `true` if all cryptographic checks, signatures, and chain continuities passed.
    pub is_valid: bool,
    /// Detailed error reason if `is_valid` is `false`.
    pub error_reason: Option<VerificationErrorReason>,
}

impl VerificationResult {
    /// Creates a successful verification result.
    pub fn success() -> Self {
        Self {
            is_valid: true,
            error_reason: None,
        }
    }

    /// Creates a failed verification result with an explicit reason.
    pub fn failure(reason: VerificationErrorReason) -> Self {
        Self {
            is_valid: false,
            error_reason: Some(reason),
        }
    }
}

/// Public facade for standalone third-party verification of deletion proofs and audit chains.
///
/// Enables external auditors, regulators, or third parties to verify deletion proof bundles and
/// audit chains without direct access to Contextra database instances or internal state.
///
/// # Example
///
/// ```rust
/// use contextra_crypto::deletion_proof::{
///     DeletionProof, DeletionProofVerifier, DeletionScope, LayerCleanupProof, DeletionLayer
/// };
/// use contextra_crypto::ed25519_proof::DeletionProofKeyPair;
/// use contextra_types::{TenantId, TxId};
///
/// let keypair = DeletionProofKeyPair::generate();
///
/// let proof = DeletionProof::create_v3(
///     TenantId(42),
///     TxId(100),
///     DeletionScope::Tenant { tenant_id: TenantId(42) },
///     &[b"doc_key_1".to_vec()],
///     1700000000,
///     &[LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
///     &keypair,
/// ).unwrap();
///
/// let vk_bytes = keypair.verifying_key_bytes();
/// let result = DeletionProofVerifier::verify_standalone(&proof, &vk_bytes);
/// assert!(result.is_valid);
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct DeletionProofVerifier;

impl DeletionProofVerifier {
    /// Verifies a single standalone `DeletionProof` using a public key.
    pub fn verify_standalone(
        proof: &DeletionProof,
        verifying_key_bytes: &[u8; 32],
    ) -> VerificationResult {
        let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(verifying_key_bytes) {
            Ok(vk) => vk,
            Err(e) => return VerificationResult::failure(VerificationErrorReason::InvalidKey(e.to_string())),
        };

        if let Err(e) = proof.verify_external(&verifying_key) {
            let msg = e.to_string();
            let reason = if msg.contains("unsupported deletion proof version") {
                VerificationErrorReason::UnsupportedVersion(proof.signature_version)
            } else if msg.contains("invalid verifying key") {
                VerificationErrorReason::InvalidKey(msg)
            } else {
                VerificationErrorReason::InvalidSignature(msg)
            };
            return VerificationResult::failure(reason);
        }

        VerificationResult::success()
    }

    /// Verifies a `DeletionProof` and checks tenant binding.
    pub fn verify_standalone_for_tenant(
        proof: &DeletionProof,
        expected_tenant: TenantId,
        verifying_key_bytes: &[u8; 32],
    ) -> VerificationResult {
        if proof.tenant_id() != expected_tenant {
            return VerificationResult::failure(VerificationErrorReason::TenantMismatch {
                expected: expected_tenant,
                actual: proof.tenant_id(),
            });
        }
        Self::verify_standalone(proof, verifying_key_bytes)
    }

    /// Verifies an audit chain and checks head signature against expected verifying key.
    pub fn verify_audit_chain_and_head(
        chain: &crate::audit_chain::AuditChain,
        head_signature: &crate::audit_chain::AuditChainHeadSignature,
        verifying_key_bytes: &[u8; 32],
    ) -> VerificationResult {
        if chain.entries.is_empty() {
            return VerificationResult::failure(VerificationErrorReason::EmptyChain);
        }

        let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(verifying_key_bytes) {
            Ok(vk) => vk,
            Err(e) => return VerificationResult::failure(VerificationErrorReason::InvalidKey(e.to_string())),
        };

        match chain.verify_chain() {
            Ok(true) => {}
            Ok(false) => {
                return VerificationResult::failure(VerificationErrorReason::ChainBroken {
                    position: 0,
                    details: "AuditChain sequential hash continuity check failed".to_string(),
                });
            }
            Err(e) => {
                return VerificationResult::failure(VerificationErrorReason::ChainBroken {
                    position: 0,
                    details: e.to_string(),
                });
            }
        }

        let actual_last_hash = match chain.entries.last() {
            Some(last) => last.entry_hash,
            None => return VerificationResult::failure(VerificationErrorReason::EmptyChain),
        };

        if head_signature.head_hash != actual_last_hash {
            return VerificationResult::failure(VerificationErrorReason::HeadMismatch {
                expected_head_hash: head_signature.head_hash,
                actual_head_hash: actual_last_hash,
            });
        }

        match crate::audit_chain::AuditChain::verify_head_signature(head_signature, &verifying_key) {
            Ok(true) => VerificationResult::success(),
            Ok(false) => VerificationResult::failure(VerificationErrorReason::HeadMismatch {
                expected_head_hash: head_signature.head_hash,
                actual_head_hash: actual_last_hash,
            }),
            Err(e) => VerificationResult::failure(VerificationErrorReason::InvalidSignature(e.to_string())),
        }
    }
}
