// FILE-CONTEXT
// ZWECK: Campaign coverage test for DeletionProof verify_external and export_for_audit.
// INVARIANTEN: DeletionProof V3 signatures are verified externally via Ed25519 verifying key without secret key.
// NICHT-OFFENSICHTLICH: Uses independent oracle (R4) validating Ed25519 payload verification and tamper rejection.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_crypto::deletion_proof::{DeletionProof, DeletionScope};
use contextra_types::{TenantId, TxId};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

#[test]
fn test_deletion_proof_verify_external_and_tamper() {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    let tenant_id = TenantId::SYSTEM;

    let scope = DeletionScope::Tenant { tenant_id };
    let proof = DeletionProof::create_v3(
        scope,
        vec![b"deleted_key_1".to_vec(), b"deleted_key_2".to_vec()],
        TxId::new(100),
        vec![],
        vec![],
        123456789,
        &[],
        &sk,
    )
    .expect("create_v3 deletion proof");

    // 1. Verify external verification using verifying key (Happy Path)
    let verify_res = proof.verify_external(&vk);
    assert!(
        verify_res.is_ok(),
        "Valid V3 DeletionProof MUST pass verify_external"
    );

    // 2. Export for audit verification
    let json_audit = proof.export_for_audit().expect("export_for_audit");
    assert!(!json_audit.is_empty());
    assert!(json_audit.contains("signature"));

    // 3. Tamper Check: Tampered signature rejected
    let mut tampered_proof = proof.clone();
    tampered_proof.signature[0] ^= 0xFF;
    assert!(
        tampered_proof.verify_external(&vk).is_err(),
        "Tampered signature MUST fail verify_external"
    );
}
