// ZWECK: J25 Closure Test for contextra-crypto 9 symbols verification.
// SYMBOLE:
// 1. sign_head (audit_chain.rs)
// 2. verify_head_signature (audit_chain.rs)
// 3. create_v3_with_audit_position (deletion_proof.rs)
// 4. verify_wal_delete_receipt (deletion_proof.rs)
// 5. get_wrapped_dek (kv_shredding.rs)
// 6. get_wrapped_kek (kv_shredding.rs)
// 7. is_record_active (kv_shredding.rs)
// 8. new_in_memory (revocation_log.rs)
// 9. verify_integrity (revocation_log.rs)

use contextra_crypto::audit_chain::{AuditChain, DataClass};
use contextra_crypto::crypto::KeyManager;
use contextra_crypto::deletion_proof::{
    compute_wal_delete_receipt, verify_wal_delete_receipt, DeletionLayer, DeletionProof,
    DeletionProofKeyPair, DeletionScope, LayerCleanupProof,
};
use contextra_crypto::kv_shredding::KeyRegistry;
use contextra_crypto::revocation_log::RevocationLog;
use contextra_ports::SystemClock;
use contextra_types::{DocId, TenantId, TxId};
use std::sync::Arc;

#[test]
fn test_j25_closure_all_9_symbols_verification() -> Result<(), Box<dyn std::error::Error>> {
    // 1 & 2. sign_head & verify_head_signature in AuditChain
    let mut audit_chain = AuditChain::new();
    let doc_id = DocId(101);
    let tx_id = TxId(202);
    let rules_hash = [0xAAu8; 32];
    audit_chain.append(
        "schema_v1",
        rules_hash,
        DataClass::Confidential,
        doc_id,
        tx_id,
        None,
    )?;

    let keypair = DeletionProofKeyPair::generate();
    let head_sig = audit_chain.sign_head(keypair.signing_key())?;
    let is_sig_valid = AuditChain::verify_head_signature(&head_sig, &keypair.verifying_key)?;
    assert!(
        is_sig_valid,
        "verify_head_signature MUST succeed for valid sign_head signature"
    );

    let wrong_keypair = DeletionProofKeyPair::generate();
    let is_wrong_sig_valid =
        AuditChain::verify_head_signature(&head_sig, &wrong_keypair.verifying_key)?;
    assert!(
        !is_wrong_sig_valid,
        "verify_head_signature MUST reject signature with wrong verifying key"
    );

    // 3. create_v3_with_audit_position in DeletionProof
    let tenant_id = TenantId::try_new(1).expect("valid tenant_id");
    let scope = DeletionScope::Document { doc_id, tenant_id };
    let cleanup_proof = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)
        .expect("valid cleanup proof");
    let audit_position = Some(head_sig.chain_index);

    let proof = DeletionProof::create_v3_with_audit_position(
        scope,
        vec![b"key_1".to_vec(), b"key_2".to_vec()],
        tx_id,
        vec![cleanup_proof],
        vec![],
        audit_position,
        1700000000,
        &[],
        keypair.signing_key(),
    )?;

    assert_eq!(proof.signature_version, 3);
    assert_eq!(proof.audit_chain_position, audit_position);
    assert!(proof.verify(&keypair.verifying_key)?);

    // 4. verify_wal_delete_receipt in DeletionProof
    let integrity_key = b"integrity_key_32_bytes_long_!!";
    let prev_hmac = [0x11u8; 32];
    let delete_event = b"doc_101_deleted_at_tx_202";

    let receipt = compute_wal_delete_receipt(&prev_hmac, delete_event, integrity_key)?;
    let is_receipt_valid =
        verify_wal_delete_receipt(&receipt, &prev_hmac, delete_event, integrity_key)?;
    assert!(
        is_receipt_valid,
        "verify_wal_delete_receipt MUST return true for valid receipt"
    );

    let is_tampered_receipt_valid =
        verify_wal_delete_receipt(&receipt, &prev_hmac, b"tampered_event", integrity_key)?;
    assert!(
        !is_tampered_receipt_valid,
        "verify_wal_delete_receipt MUST return false for tampered event payload"
    );

    // 8 & 9. new_in_memory & verify_integrity in RevocationLog
    let clock = Arc::new(SystemClock::new());
    let ed25519_sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
    let ed25519_vk = ed25519_sk.verifying_key();

    let rev_log = RevocationLog::new_in_memory(clock.clone(), Some(ed25519_sk.clone()), ed25519_vk);
    rev_log.verify_integrity()?;

    // 5, 6 & 7. get_wrapped_dek, get_wrapped_kek & is_record_active in KeyRegistry
    let registry = KeyRegistry::new_in_memory(clock, Some(ed25519_sk), ed25519_vk);
    let master_km = KeyManager::try_new("master_passphrase_test", b"salt_bytes_1234")?;
    let group_id = 888;
    let record_id = 999;

    let payload =
        registry.encrypt_record(&master_km, group_id, record_id, b"sensitive record bytes")?;

    assert!(
        registry.is_record_active(group_id, record_id),
        "is_record_active MUST return true for newly encrypted record"
    );

    let wrapped_kek_opt = registry.get_wrapped_kek(group_id);
    assert!(
        wrapped_kek_opt.is_some(),
        "get_wrapped_kek MUST return Some((wrapped_kek, kek_nonce)) for active group"
    );

    let wrapped_dek_opt = registry.get_wrapped_dek(group_id, record_id);
    assert!(
        wrapped_dek_opt.is_some(),
        "get_wrapped_dek MUST return Some((wrapped_dek, dek_nonce)) for active record"
    );
    let (wrapped_dek, dek_nonce) = wrapped_dek_opt.unwrap();
    assert_eq!(wrapped_dek, payload.wrapped_dek);
    assert_eq!(dek_nonce, payload.dek_nonce);

    // Revoke record and check is_record_active & get_wrapped_dek
    let revoked_rec = registry.revoke_record(group_id, record_id)?;
    assert!(revoked_rec, "revoke_record MUST return true");

    assert!(
        !registry.is_record_active(group_id, record_id),
        "is_record_active MUST return false after record revocation"
    );
    assert!(
        registry.get_wrapped_dek(group_id, record_id).is_none(),
        "get_wrapped_dek MUST return None after record revocation"
    );
    assert!(
        registry.get_wrapped_kek(group_id).is_some(),
        "get_wrapped_kek MUST still return Some after single record revocation"
    );

    // Revoke group and check get_wrapped_kek
    let revoked_grp = registry.revoke_group(group_id)?;
    assert!(revoked_grp, "revoke_group MUST return true");

    assert!(
        registry.get_wrapped_kek(group_id).is_none(),
        "get_wrapped_kek MUST return None after group revocation"
    );

    Ok(())
}
