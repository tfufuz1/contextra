// FILE-CONTEXT
// ZWECK: Integration tests for J25 contextra-crypto symbols wiring reachability verification.
// INVARIANTEN: Verifies production path reachability for all 9 audit symbols.

#![forbid(unsafe_code)]

use contextra_crypto::audit_chain::{AuditChain, DataClass};
use contextra_crypto::crypto::KeyManager;
use contextra_crypto::deletion_proof::{
    compute_wal_delete_receipt, DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope,
    ExcludedScope, LayerCleanupProof,
};
use contextra_crypto::kv_shredding::KeyRegistry;
use contextra_ports::SystemClock;
use contextra_types::{DocId, TenantId, TxId};
use std::sync::Arc;

#[test]
fn test_j25_audit_chain_signed_anchor_wiring() {
    let keypair = DeletionProofKeyPair::generate();
    let mut chain = AuditChain::new();

    chain
        .append(
            "schema_v1",
            [0x11u8; 32],
            DataClass::Confidential,
            DocId(1),
            TxId(100),
            None,
        )
        .expect("append entry 1");

    chain
        .append(
            "schema_v1",
            [0x22u8; 32],
            DataClass::Restricted,
            DocId(2),
            TxId(101),
            Some([0x33u8; 32]),
        )
        .expect("append entry 2");

    let (anchor, head_sig) = chain
        .create_signed_anchor(keypair.signing_key())
        .expect("create_signed_anchor must succeed");

    assert_eq!(anchor.index, 1);
    assert_eq!(head_sig.chain_index, 1);

    let valid = chain
        .verify_signed_anchor(&anchor, &head_sig, &keypair.verifying_key)
        .expect("verify_signed_anchor must succeed");

    assert!(valid, "Signed anchor verification must return true");
}

#[test]
fn test_j25_deletion_proof_v3_audit_pos_and_wal_receipt_wiring() {
    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::try_new(7).unwrap(),
    };

    let cleanup_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
    ];

    let proof_v3 = DeletionProof::create_v3(
        scope.clone(),
        vec![b"key_a".to_vec()],
        TxId(123),
        cleanup_proofs.clone(),
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .expect("create_v3 must succeed");

    assert_eq!(proof_v3.signature_version, 3);
    assert!(proof_v3.verify(&keypair.verifying_key).unwrap());

    let proof_with_pos = DeletionProof::create_v3_with_audit_position(
        scope,
        vec![b"key_b".to_vec()],
        TxId(124),
        cleanup_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        Some(42),
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .expect("create_v3_with_audit_position must succeed");

    assert_eq!(proof_with_pos.audit_chain_position, Some(42));
    assert!(proof_with_pos.verify(&keypair.verifying_key).unwrap());

    // Test verify_wal_receipt wiring
    let integrity_key = b"integrity_key_32_bytes_test_wal!";
    let prev_hmac = [0xAAu8; 32];
    let payload = b"delete_event_payload_doc_42";

    let receipt = compute_wal_delete_receipt(&prev_hmac, payload, integrity_key)
        .expect("compute receipt");

    let proof_with_receipt = DeletionProof::create_with_wal_receipt_v3(
        DeletionScope::Tenant {
            tenant_id: TenantId::try_new(7).unwrap(),
        },
        vec![b"k1".to_vec()],
        TxId(125),
        vec![],
        vec![],
        Some(receipt),
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .expect("create_with_wal_receipt_v3");

    let receipt_valid = proof_with_receipt
        .verify_wal_receipt(&prev_hmac, payload, integrity_key)
        .expect("verify_wal_receipt");

    assert!(receipt_valid, "WAL receipt must verify as valid");
}

#[test]
fn test_j25_key_registry_envelope_and_revocation_log_wiring() {
    let km = KeyManager::try_new("test-passphrase-j25", b"salt-j25").expect("KeyManager");
    let keypair = DeletionProofKeyPair::generate();
    let clock = Arc::new(SystemClock::new());

    // Tests KeyRegistry::new_in_memory and RevocationLog::new_in_memory
    let registry = KeyRegistry::new_in_memory(
        clock,
        Some(keypair.signing_key().clone()),
        keypair.verifying_key,
    );

    let group_id = 500;
    let record_id = 1;
    let plaintext = b"Confidential record data for J25 test";

    // Encrypt record
    let payload = registry
        .encrypt_record(&km, group_id, record_id, plaintext)
        .expect("encrypt_record");

    // Tests is_record_active and get_wrapped_dek inside decrypt_record
    assert!(registry.is_record_active(group_id, record_id));
    assert!(registry.get_wrapped_dek(group_id, record_id).is_some());
    assert!(registry.get_wrapped_kek(group_id).is_some());

    let decrypted = registry
        .decrypt_record(&km, &payload)
        .expect("decrypt_record");
    assert_eq!(decrypted, plaintext);

    // Tests encrypt_with_group & decrypt_with_group (which calls get_wrapped_kek)
    let (ct, nonce) = registry
        .encrypt_with_group(&km, group_id, b"Group payload")
        .expect("encrypt_with_group");
    let group_decrypted = registry
        .decrypt_with_group(group_id, &ct, &nonce)
        .expect("decrypt_with_group");
    assert_eq!(group_decrypted, b"Group payload");

    // Revoke record and verify decryption fails
    registry
        .revoke_record(group_id, record_id)
        .expect("revoke_record");
    assert!(!registry.is_record_active(group_id, record_id));

    let decrypt_fail = registry.decrypt_record(&km, &payload);
    assert!(decrypt_fail.is_err());

    // Revoke group
    registry.revoke_group(group_id).expect("revoke_group");
    assert!(registry.get_wrapped_kek(group_id).is_none());

    // Verify revocation log integrity via verify_integrity
    let log = registry.revocation_log.as_ref().expect("revocation log");
    assert!(
        log.verify_integrity().is_ok(),
        "RevocationLog integrity check must pass"
    );
}
