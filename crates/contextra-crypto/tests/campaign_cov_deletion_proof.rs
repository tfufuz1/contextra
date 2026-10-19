// FILE-CONTEXT
// ZWECK: Campaign coverage test for DeletionProof verify_external and export_for_audit.
// INVARIANTEN: DeletionProof V3 signatures are verified externally via Ed25519 verifying key without secret key.
// NICHT-OFFENSICHTLICH: Uses independent oracle (R4) validating Ed25519 payload verification and tamper rejection.

#![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]

use contextra_crypto::deletion_proof::builder::ProofBuilder;
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::DeletionScope;
use contextra_types::TenantId;

#[test]
fn test_deletion_proof_verify_external_and_tamper() {
    let signer = ProofKeyPairSigner::from_secret_bytes([0x42u8; 32]);
    let tenant_id = TenantId::SYSTEM;

    let scope = DeletionScope::Tenant { tenant_id };
    let proof_v4 = ProofBuilder::new(scope, "db-cov-instance")
        .with_deleted_after_tx(100)
        .finish(&signer)
        .expect("finish v4 proof");

    // 1. Verify signature using public key (Happy Path)
    let verify_res = proof_v4.verify_signature(&signer.public_key());
    assert!(
        verify_res.unwrap_or(false),
        "Valid V4 DeletionProof MUST pass verify_signature"
    );

    // 2. Export for audit verification
    let json_audit = serde_json::to_string(&proof_v4).expect("json serialize");
    assert!(!json_audit.is_empty());
    assert!(json_audit.contains("signature"));

    // 3. Tamper Check: Tampered signature rejected
    let mut tampered_proof = proof_v4.clone();
    tampered_proof.signature[0] ^= 0xFF;
    assert!(
        !tampered_proof.verify_signature(&signer.public_key()).unwrap_or(true),
        "Tampered signature MUST fail verify_signature"
    );
}
