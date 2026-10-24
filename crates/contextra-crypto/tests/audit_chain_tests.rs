// FILE-CONTEXT
// ZWECK: Tests for Blake3 Audit Hash Chain, Ed25519 head signature verification, crypto-shredded record commitments, and DeletionProof linkage.
// INVARIANTEN: Tampering with any entry breaks chain verification. Shredded salt renders commitments indistinguishable under dictionary attacks. INV-DELETION-1 strictly enforced.

#![forbid(unsafe_code)]

use contextra_crypto::{
    compute_record_commitment, AuditChain, DataClass, DeletionLayer, DeletionProof,
    DeletionProofKeyPair, DeletionScope, EncryptedCommitmentSalt, ExcludedScope, KeyManager,
    KeyRegistry, LayerCleanupProof,
};
use contextra_types::{DocId, TenantId, TxId};

#[test]
fn test_audit_chain_integrity_and_tamper_detection() {
    let mut chain = AuditChain::new();
    let rules_hash = [0xAAu8; 32];

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Internal,
            DocId(101),
            TxId(1),
            None,
        )
        .unwrap();

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Confidential,
            DocId(102),
            TxId(2),
            Some([0xBBu8; 32]),
        )
        .unwrap();

    chain
        .append(
            "schema_v1",
            rules_hash,
            DataClass::Personal,
            DocId(103),
            TxId(3),
            None,
        )
        .unwrap();

    assert_eq!(chain.len(), 3);
    assert!(chain.verify_chain().unwrap());

    // Tamper with middle entry's doc_id
    let mut tampered_chain = chain.clone();
    tampered_chain.entries.get_mut(1).unwrap().doc_id = DocId(999);
    assert!(!tampered_chain.verify_chain().unwrap());

    // Tamper with middle entry's prev_hash
    let mut tampered_chain2 = chain.clone();
    tampered_chain2.entries.get_mut(1).unwrap().prev_hash = [0xFFu8; 32];
    assert!(!tampered_chain2.verify_chain().unwrap());

    // Tamper with rules_hash
    let mut tampered_chain3 = chain.clone();
    tampered_chain3.entries.get_mut(0).unwrap().rules_hash = [0x11u8; 32];
    assert!(!tampered_chain3.verify_chain().unwrap());
}

#[test]
fn test_audit_chain_head_signature_verification() {
    let mut chain = AuditChain::new();
    let keypair = DeletionProofKeyPair::generate();

    chain
        .append(
            "schema_v1",
            [0x11u8; 32],
            DataClass::Public,
            DocId(1),
            TxId(10),
            None,
        )
        .unwrap();

    let head_sig = chain.sign_head(keypair.signing_key()).unwrap();
    assert_eq!(head_sig.chain_index, 0);
    assert_eq!(head_sig.head_hash, chain.head_hash());

    // Verify with valid key
    assert!(AuditChain::verify_head_signature(&head_sig, &keypair.verifying_key).unwrap());

    // Verify with wrong key
    let wrong_keypair = DeletionProofKeyPair::generate();
    assert!(!AuditChain::verify_head_signature(&head_sig, &wrong_keypair.verifying_key).unwrap());

    // Tamper with signature
    let mut tampered_sig = head_sig.clone();
    tampered_sig.signature[0] ^= 0xFF;
    assert!(!AuditChain::verify_head_signature(&tampered_sig, &keypair.verifying_key).unwrap());

    // Tamper with head_hash in signature structure
    let mut tampered_hash_sig = head_sig;
    tampered_hash_sig.head_hash[0] ^= 0xFF;
    assert!(
        !AuditChain::verify_head_signature(&tampered_hash_sig, &keypair.verifying_key).unwrap()
    );
}

#[test]
fn test_commitment_shredding_and_dictionary_attack_resilience() {
    let km = KeyManager::try_new("master-passphrase-audit", b"audit-salt-1234").unwrap();
    let registry = KeyRegistry::new();
    let group_id = 42;

    let real_salt = b"secret_random_salt_32_bytes_long!";
    let real_attr = b"user@example.com";

    // 1. Compute commitment = Blake3(salt || attr)
    let commitment = compute_record_commitment(real_salt, real_attr);

    // 2. Encrypt salt under group_id in registry
    let enc_salt = EncryptedCommitmentSalt::encrypt(&registry, &km, group_id, real_salt).unwrap();

    // Verification before shredding: Salt can be decrypted and commitment verified
    let decrypted_salt = enc_salt.decrypt(&registry).unwrap();
    assert_eq!(decrypted_salt, real_salt);
    assert_eq!(
        compute_record_commitment(&decrypted_salt, real_attr),
        commitment
    );

    // 3. Shred the salt key in registry
    assert!(registry.revoke_subkey(group_id));

    // Decryption of salt now fails because sub-key is destroyed
    assert!(enc_salt.decrypt(&registry).is_err());

    // 4. Dictionary Attack Simulation:
    // Without knowledge of `real_salt`, an attacker testing candidates `cand_a` and `cand_b`
    // cannot evaluate Blake3(salt || candidate) or distinguish `cand_a` from `cand_b`.
    let cand_a = b"user@example.com";
    let cand_b = b"other_user@example.com";

    // Without salt, two candidate attributes tested with any dummy/guessed salt values yield distinct hashes
    // that are completely uncorrelated to the recorded commitment.
    let dummy_salt_guess_1 = [0x01u8; 32];
    let dummy_salt_guess_2 = [0x02u8; 32];

    assert_ne!(
        compute_record_commitment(&dummy_salt_guess_1, cand_a),
        commitment
    );
    assert_ne!(
        compute_record_commitment(&dummy_salt_guess_2, cand_b),
        commitment
    );
    assert_ne!(
        compute_record_commitment(&dummy_salt_guess_1, cand_a),
        compute_record_commitment(&dummy_salt_guess_1, cand_b)
    );
}

#[test]
fn test_deletion_proof_linked_to_audit_chain_position() {
    let keypair = DeletionProofKeyPair::generate();
    let mut chain = AuditChain::new();

    let entry = chain
        .append(
            "schema_v1",
            [0x22u8; 32],
            DataClass::Restricted,
            DocId(500),
            TxId(100),
            None,
        )
        .unwrap();

    let audit_pos = entry.index;

    let scope = DeletionScope::Document {
        doc_id: DocId(500),
        tenant_id: TenantId::try_new(1).unwrap(),
    };

    let cleanup_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0).unwrap(),
    ];

    let proof = DeletionProof::create_v3_with_audit_position(
        scope,
        vec![b"doc_500_key".to_vec()],
        TxId(100),
        cleanup_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        Some(audit_pos),
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    assert_eq!(proof.audit_chain_position, Some(audit_pos));
    assert!(proof.verify_external(&keypair.verifying_key).is_ok());

    // Tampering with audit_chain_position invalidates signature
    let mut tampered = proof;
    tampered.audit_chain_position = Some(999);
    assert!(tampered.verify_external(&keypair.verifying_key).is_err());
}

#[test]
fn test_inv_deletion_1_physical_cleanup_enforced() {
    // LayerCleanupProof fails if remaining live entries exist (> 0)
    let incomplete_cleanup =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 1);
    assert!(incomplete_cleanup.is_err());

    // Verification closure returning false fails
    let failed_verification =
        LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || Ok(false));
    assert!(failed_verification.is_err());
}
