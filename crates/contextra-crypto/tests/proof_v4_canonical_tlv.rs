// FILE-CONTEXT (STAND 2026-10-13, ZWECK: Kanonische TLV-Kodierung und verify_strict-Prüfung für Löschbeweise Version 4, INVARIANTEN: Zero-Panic, Determinismus, Fail-Closed)

//! Tests für Version 4 Deletion Proof kanonische TLV-Kodierung, Salting und verify_strict.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation
)]

use contextra_crypto::deletion_proof::builder::ProofBuilder;
use contextra_crypto::deletion_proof::proof_v4::{encode_scope_tlv, ProofClaimV4, ProofV4};
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::types::DeletionScope;
use contextra_types::{CollectionId, DocId, TenantId};
use rand::rngs::OsRng;

fn create_valid_test_proof(signer: &ProofKeyPairSigner) -> ProofV4 {
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::try_new(1).expect("tenant_id"),
    };

    ProofBuilder::new(scope, "db-instance-tlv-test")
        .with_deleted_after_tx(100)
        .with_deleted_keys_hash([0x11; 32])
        .with_footprint_hash([0x22; 32])
        .add_claim(ProofClaimV4 {
            layer_id: 1,
            assurance: 2,
            claim_text_id: 3,
            witness_digest: [0x33; 32],
            scanned_at_tx: 100,
            evidence_tlv: vec![0x01, 0x02],
        })
        .finish(signer)
        .expect("builder finish")
}

#[test]
fn test_roundtrip_encoding_and_strict_verification() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let proof = create_valid_test_proof(&signer);

    // Verify valid signature with strict verification
    let verified = proof
        .verify_signature(&signer.public_key())
        .expect("verify signature");
    assert!(verified, "Valid signature MUST pass verify_strict");

    // Invalid public key fails verification
    let wrong_signer = ProofKeyPairSigner::generate(&mut OsRng);
    let wrong_verified = proof
        .verify_signature(&wrong_signer.public_key())
        .expect("verify signature");
    assert!(!wrong_verified, "Wrong public key MUST fail verification");
}

#[test]
fn test_encoding_determinism_identical_inputs() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);

    let p1 = create_valid_test_proof(&signer);
    let p2 = p1.clone();

    let bytes1 = p1.try_construct_canonical_payload().expect("p1 payload");
    let bytes2 = p2.try_construct_canonical_payload().expect("p2 payload");

    assert_eq!(
        bytes1, bytes2,
        "Two encodings of identical proofs MUST be byte-identical"
    );

    let sig_bytes1 = p1.try_signed_bytes().expect("p1 signed_bytes");
    let sig_bytes2 = p2.try_signed_bytes().expect("p2 signed_bytes");
    assert_eq!(
        sig_bytes1, sig_bytes2,
        "Signed bytes of identical proofs MUST be byte-identical"
    );
}

#[test]
fn test_field_and_scope_mutation_alters_payload_and_signature() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let base_proof = create_valid_test_proof(&signer);
    let base_payload = base_proof
        .try_construct_canonical_payload()
        .expect("base payload");

    // 1. Mutate stage
    let mut p_stage = base_proof.clone();
    p_stage.stage = 1;
    let payload_stage = p_stage
        .try_construct_canonical_payload()
        .expect("stage payload");
    assert_ne!(
        base_payload, payload_stage,
        "Mutating stage MUST alter canonical payload"
    );

    // 2. Mutate scope (Document ID 42 -> 43)
    let mut p_scope = base_proof.clone();
    p_scope.scope = DeletionScope::Document {
        doc_id: DocId(43),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };
    let payload_scope = p_scope
        .try_construct_canonical_payload()
        .expect("scope payload");
    assert_ne!(
        base_payload, payload_scope,
        "Mutating scope Document ID MUST alter canonical payload"
    );

    // 3. Mutate scope variant (Document -> Collection)
    let mut p_coll = base_proof.clone();
    p_coll.scope = DeletionScope::Collection {
        collection_id: CollectionId(42),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };
    let payload_coll = p_coll
        .try_construct_canonical_payload()
        .expect("collection scope payload");
    assert_ne!(
        base_payload, payload_coll,
        "Mutating scope variant MUST alter canonical payload"
    );

    // Verification fails on mutated proof
    let verify_mutated = p_stage
        .verify_signature(&signer.public_key())
        .expect("verify signature");
    assert!(
        !verify_mutated,
        "Signature verification MUST fail when payload is mutated without re-signing"
    );
}

#[test]
fn test_no_plaintext_ids_in_scope_tlv_and_salted_hashes() {
    let proof_id = [0x55u8; 32];

    let scope_doc = DeletionScope::Document {
        doc_id: DocId(0xDEADBEEF12345678),
        tenant_id: TenantId::try_new(0xCAFEBABE11223344).expect("tenant"),
    };

    let scope_tlv = encode_scope_tlv(&scope_doc, &proof_id).expect("encode scope tlv");

    // Plaintext bytes for DocId and TenantId (big-endian and little-endian)
    let doc_be = 0xDEADBEEF12345678u64.to_be_bytes();
    let doc_le = 0xDEADBEEF12345678u64.to_le_bytes();
    let tenant_be = 0xCAFEBABE11223344u64.to_be_bytes();
    let tenant_le = 0xCAFEBABE11223344u64.to_le_bytes();

    // Verify plaintext DocId and TenantId bytes do NOT appear anywhere in the scope TLV
    let contains_doc =
        scope_tlv.windows(8).any(|w| w == doc_be) || scope_tlv.windows(8).any(|w| w == doc_le);
    let contains_tenant = scope_tlv.windows(8).any(|w| w == tenant_be)
        || scope_tlv.windows(8).any(|w| w == tenant_le);

    assert!(
        !contains_doc,
        "Plaintext Document ID MUST NOT appear in scope TLV payload"
    );
    assert!(
        !contains_tenant,
        "Plaintext Tenant ID MUST NOT appear in scope TLV payload"
    );

    // Different proof_ids produce different salts and thus different salted ID hashes
    let proof_id_2 = [0xAAu8; 32];
    let scope_tlv_2 = encode_scope_tlv(&scope_doc, &proof_id_2).expect("encode scope tlv 2");
    assert_ne!(
        scope_tlv, scope_tlv_2,
        "Different proof_ids MUST produce distinct salted hashes"
    );
}

#[test]
fn test_non_canonical_high_s_signature_rejected_by_verify_strict() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let mut proof = create_valid_test_proof(&signer);

    // Sign legitimately
    proof.sign_with_signer(&signer).expect("sign");

    assert!(
        proof
            .verify_signature(&signer.public_key())
            .expect("verify"),
        "Legitimate signature MUST pass"
    );

    // Construct non-canonical signature by modifying S scalar in signature [32..64]
    // Curve25519 order L = 2^252 + 27742317777372353535851937790883648493
    // Setting highest bit in S component (byte 63 |= 0x80) creates a value S >= L
    let mut non_canonical_proof = proof.clone();
    non_canonical_proof.signature[63] |= 0x80;

    let res = non_canonical_proof
        .verify_signature(&signer.public_key())
        .expect("verify non-canonical");

    assert!(
        !res,
        "Non-canonical signature S >= L MUST be rejected by verify_strict"
    );
}
