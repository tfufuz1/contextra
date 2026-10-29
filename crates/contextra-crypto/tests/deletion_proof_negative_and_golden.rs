// FILE-CONTEXT
// ZWECK: Negative and golden vector integration tests for DeletionProof and DeletionProofVerifier.
// INVARIANTEN: Comprehensive coverage of negative scenarios (tampering, reordering, truncation, wrong key, tenant mismatch, unknown version, empty chain) and golden vector serialization stability.

#![forbid(unsafe_code)]

use contextra_crypto::audit_chain::{
    compute_record_commitment, AuditChain, AuditChainHeadSignature, DataClass,
};
use contextra_crypto::deletion_proof::{
    hash_deleted_keys_length_prefixed, DeletionLayer, DeletionProof, DeletionProofVerifier,
    DeletionScope, ExcludedScope, LayerCleanupProof, VerificationErrorReason,
};
use contextra_crypto::ed25519_proof::DeletionProofKeyPair;
use contextra_types::{DocId, TenantId, TxId};
use rand::rngs::StdRng;
use rand::SeedableRng;

// -----------------------------------------------------------------------------
// 1. Golden Vector Tests for DeletionProof & DeletionProofKeyPair
// -----------------------------------------------------------------------------

#[test]
fn test_golden_vector_deletion_proof_keypair_deterministic() {
    let mut rng = StdRng::seed_from_u64(0xDEADBEEF_12345678);
    let keypair = DeletionProofKeyPair::generate_with_rng(&mut rng);

    let vk_bytes = keypair.verifying_key_bytes();

    let expected_vk_bytes: [u8; 32] = [
        173, 23, 222, 243, 73, 218, 189, 168, 83, 75, 132, 28, 92, 250, 78, 132, 138, 235, 179,
        244, 59, 99, 172, 240, 193, 227, 151, 202, 173, 187, 247, 162,
    ];

    assert_eq!(
        vk_bytes, expected_vk_bytes,
        "Golden vector mismatch for DeletionProofKeyPair key generation"
    );
}

#[test]
fn test_golden_vector_deleted_keys_hash() {
    let keys = vec![
        b"tenant_100_doc_001".to_vec(),
        b"tenant_100_doc_002".to_vec(),
        b"tenant_100_doc_003".to_vec(),
    ];

    let hash = hash_deleted_keys_length_prefixed(&keys);

    let expected_hash: [u8; 32] = [
        154, 183, 101, 119, 218, 251, 250, 187, 66, 78, 119, 10, 78, 120, 37, 73, 134, 131, 17, 180,
        236, 196, 18, 173, 116, 212, 62, 23, 237, 50, 213, 14,
    ];

    assert_eq!(
        hash, expected_hash,
        "Golden vector mismatch for length-prefixed deleted_keys_hash"
    );
}

#[test]
fn test_golden_vector_v3_proof_payload_signature_stability() {
    let mut rng = StdRng::seed_from_u64(42);
    let keypair = DeletionProofKeyPair::generate_with_rng(&mut rng);

    let proof = DeletionProof::create_full_v3(
        DeletionScope::Document {
            doc_id: DocId::from(88),
            tenant_id: TenantId(42),
        },
        vec![b"golden_key_1".to_vec(), b"golden_key_2".to_vec()],
        TxId(1001),
        vec![
            LayerCleanupProof::for_test(DeletionLayer::LsmMemtable),
            LayerCleanupProof::for_test(DeletionLayer::SsTableAllLevels),
        ],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        Some([0x11u8; 32]),
        Some(999),
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .expect("create golden v3 proof");

    let vk_bytes = keypair.verifying_key_bytes();
    let result = DeletionProofVerifier::verify_standalone(&proof, &vk_bytes);
    assert!(result.is_valid, "Golden v3 proof must verify successfully");

    assert_eq!(proof.signature.len(), 64);
}

// -----------------------------------------------------------------------------
// 2. Negative Verification Tests (>= 20 Scenarios)
// -----------------------------------------------------------------------------

#[test]
fn test_neg_01_tampered_tenant_id() {
    let keypair = DeletionProofKeyPair::generate();
    let mut proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.scope = DeletionScope::Tenant { tenant_id: TenantId(2) }; // tamper
    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_02_tampered_tx_id() {
    let keypair = DeletionProofKeyPair::generate();
    let mut proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.deleted_after_tx = TxId(999); // tamper
    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_03_tampered_timestamp() {
    let keypair = DeletionProofKeyPair::generate();
    let mut proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.timestamp += 1; // tamper
    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_04_tampered_deleted_keys_hash() {
    let keypair = DeletionProofKeyPair::generate();
    let mut proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.deleted_keys_hash[0] ^= 0x01; // tamper
    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_05_tampered_covered_layers() {
    let keypair = DeletionProofKeyPair::generate();
    let mut proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.covered_layers.push(DeletionLayer::SsTableAllLevels); // tamper
    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_06_tampered_wal_receipt() {
    let keypair = DeletionProofKeyPair::generate();
    let mut proof = DeletionProof::create_with_wal_receipt_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        Some([0x42u8; 32]),
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.wal_chain_receipt = Some([0x99u8; 32]); // tamper
    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_07_tampered_signature_byte_flip() {
    let keypair = DeletionProofKeyPair::generate();
    let mut proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.signature[10] ^= 0xAA; // tamper signature
    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_08_truncated_signature_length() {
    let keypair = DeletionProofKeyPair::generate();
    let mut proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.signature.pop(); // truncate signature from 64 to 63 bytes
    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_09_wrong_public_key() {
    let keypair_a = DeletionProofKeyPair::generate();
    let keypair_b = DeletionProofKeyPair::generate();

    let proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair_a.signing_key(),
    )
    .unwrap();

    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair_b.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_10_invalid_malformed_verifying_key() {
    let keypair = DeletionProofKeyPair::generate();
    let proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    let invalid_vk_bytes = [0xFFu8; 32];
    let res = DeletionProofVerifier::verify_standalone(&proof, &invalid_vk_bytes);
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidKey(_)) | Some(VerificationErrorReason::InvalidSignature(_))));
}

#[test]
fn test_neg_11_tenant_mismatch() {
    let keypair = DeletionProofKeyPair::generate();
    let proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(100) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    let res = DeletionProofVerifier::verify_standalone_for_tenant(
        &proof,
        TenantId(200),
        &keypair.verifying_key_bytes(),
    );
    assert!(!res.is_valid);
    assert!(matches!(
        res.error_reason,
        Some(VerificationErrorReason::TenantMismatch {
            expected: TenantId(200),
            actual: TenantId(100)
        })
    ));
}

#[test]
fn test_neg_12_cross_tenant_replay_attack() {
    let keypair = DeletionProofKeyPair::generate();
    let proof_tenant_a = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(100) },
        vec![b"doc_a".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    let res = DeletionProofVerifier::verify_standalone_for_tenant(
        &proof_tenant_a,
        TenantId(200),
        &keypair.verifying_key_bytes(),
    );
    assert!(!res.is_valid);
}

#[test]
fn test_neg_13_unknown_signature_version() {
    let keypair = DeletionProofKeyPair::generate();
    let mut proof = DeletionProof::create_v3(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    proof.signature_version = 99; // unknown version
    let res = DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
    assert!(!res.is_valid);
    assert!(matches!(
        res.error_reason,
        Some(VerificationErrorReason::UnsupportedVersion(99))
    ));
}

#[test]
fn test_neg_14_legacy_v1_rejected_by_standalone_verifier() {
    let key = [0x42u8; 32];
    let proof_v1 = DeletionProof::create(
        DeletionScope::Tenant { tenant_id: TenantId(1) },
        vec![b"k1".to_vec()],
        TxId(10),
        vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
        vec![ExcludedScope::ConsolidatedAndDistilled],
        &key,
    )
    .unwrap();

    let dummy_vk = [0u8; 32];
    let res = DeletionProofVerifier::verify_standalone(&proof_v1, &dummy_vk);
    assert!(!res.is_valid);
    assert!(matches!(
        res.error_reason,
        Some(VerificationErrorReason::UnsupportedVersion(1)) | Some(VerificationErrorReason::UnsupportedVersion(2))
    ));
}

#[test]
fn test_neg_15_audit_chain_empty() {
    let keypair = DeletionProofKeyPair::generate();
    let empty_chain = AuditChain::new();
    let head_sig = AuditChainHeadSignature {
        chain_index: 0,
        head_hash: [0u8; 32],
        signature: vec![0u8; 64],
    };

    let res = DeletionProofVerifier::verify_audit_chain_and_head(
        &empty_chain,
        &head_sig,
        &keypair.verifying_key_bytes(),
    );
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::EmptyChain)));
}

#[test]
fn test_neg_16_audit_chain_reordered_entries() {
    let mut rng = StdRng::seed_from_u64(12345);
    let keypair = DeletionProofKeyPair::generate_with_rng(&mut rng);

    let mut chain = AuditChain::new();

    let salt1 = [0x11u8; 32];
    let comm1 = compute_record_commitment(&salt1, b"payload1");
    chain.append("schema1", [0u8; 32], DataClass::Internal, DocId(1), TxId(100), Some(comm1)).unwrap();

    let salt2 = [0x22u8; 32];
    let comm2 = compute_record_commitment(&salt2, b"payload2");
    chain.append("schema2", [0u8; 32], DataClass::Internal, DocId(2), TxId(101), Some(comm2)).unwrap();

    // Swap entries 0 and 1
    chain.entries.swap(0, 1);

    let head_sig = chain.sign_head(keypair.signing_key()).unwrap();
    let res = DeletionProofVerifier::verify_audit_chain_and_head(
        &chain,
        &head_sig,
        &keypair.verifying_key_bytes(),
    );
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::ChainBroken { .. })));
}

#[test]
fn test_neg_17_audit_chain_truncated_without_head_update() {
    let mut rng = StdRng::seed_from_u64(12345);
    let keypair = DeletionProofKeyPair::generate_with_rng(&mut rng);

    let mut chain = AuditChain::new();

    for i in 0..3 {
        let salt = [i as u8 + 1; 32];
        let comm = compute_record_commitment(&salt, format!("payload_{i}").as_bytes());
        chain.append("schema1", [0u8; 32], DataClass::Internal, DocId(i + 1), TxId(100 + i), Some(comm)).unwrap();
    }

    let original_head_sig = chain.sign_head(keypair.signing_key()).unwrap();

    let mut truncated_chain = chain.clone();
    truncated_chain.entries.pop();

    let res = DeletionProofVerifier::verify_audit_chain_and_head(
        &truncated_chain,
        &original_head_sig,
        &keypair.verifying_key_bytes(),
    );
    assert!(!res.is_valid);
    assert!(matches!(
        res.error_reason,
        Some(VerificationErrorReason::ChainBroken { .. }) | Some(VerificationErrorReason::HeadMismatch { .. })
    ));
}

#[test]
fn test_neg_18_audit_chain_tampered_entry_payload_commitment() {
    let mut rng = StdRng::seed_from_u64(12345);
    let keypair = DeletionProofKeyPair::generate_with_rng(&mut rng);

    let mut chain = AuditChain::new();
    let salt = [0x11u8; 32];
    let comm = compute_record_commitment(&salt, b"payload");
    chain.append("schema1", [0u8; 32], DataClass::Internal, DocId(1), TxId(100), Some(comm)).unwrap();

    if let Some(ref mut c) = chain.entries[0].commitment {
        c[0] ^= 0xFF;
    }

    let head_sig = chain.sign_head(keypair.signing_key()).unwrap();
    let res = DeletionProofVerifier::verify_audit_chain_and_head(
        &chain,
        &head_sig,
        &keypair.verifying_key_bytes(),
    );
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::ChainBroken { .. })));
}

#[test]
fn test_neg_19_audit_chain_head_signature_forged_key() {
    let keypair_signer = DeletionProofKeyPair::generate();
    let keypair_verifier = DeletionProofKeyPair::generate();

    let mut chain = AuditChain::new();
    let salt = [0x11u8; 32];
    let comm = compute_record_commitment(&salt, b"payload");
    chain.append("schema1", [0u8; 32], DataClass::Internal, DocId(1), TxId(100), Some(comm)).unwrap();

    let head_sig = chain.sign_head(keypair_signer.signing_key()).unwrap();

    let res = DeletionProofVerifier::verify_audit_chain_and_head(
        &chain,
        &head_sig,
        &keypair_verifier.verifying_key_bytes(),
    );
    assert!(!res.is_valid);
    assert!(matches!(res.error_reason, Some(VerificationErrorReason::InvalidSignature(_)) | Some(VerificationErrorReason::HeadMismatch { .. })));
}
