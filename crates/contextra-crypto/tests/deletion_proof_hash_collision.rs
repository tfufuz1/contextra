// FILE-CONTEXT
// ZWECK: Tests verifying collision prevention via length-prefixed hashing for DeletionProof.
// INVARIANTEN: Key sets ["ab", "c"] and ["a", "bc"] MUST produce distinct deleted_keys_hash values.
// STAND: TS:2026-09-27T00:00:00Z

use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, ExcludedScope, LayerCleanupProof,
};
use contextra_types::{DocId, TenantId, TxId};

#[test]
fn test_length_prefixed_hash_prevents_collision_ab_c_vs_a_bc() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(1).expect("valid tenant_id"),
    };
    let hmac_key = b"0123456789abcdef0123456789abcdef";

    let proof1 = DeletionProof::create(
        scope.clone(),
        vec![b"ab".to_vec(), b"c".to_vec()],
        TxId::new(100),
        vec![],
        vec![],
        hmac_key,
    )
    .expect("creation of proof1 should succeed");

    let proof2 = DeletionProof::create(
        scope,
        vec![b"a".to_vec(), b"bc".to_vec()],
        TxId::new(100),
        vec![],
        vec![],
        hmac_key,
    )
    .expect("creation of proof2 should succeed");

    assert_ne!(
        proof1.deleted_keys_hash, proof2.deleted_keys_hash,
        "[\"ab\", \"c\"] and [\"a\", \"bc\"] MUST yield distinct deleted_keys_hash values"
    );
}

#[test]
fn test_deletion_proof_input_order_determinism() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(1).expect("valid tenant_id"),
    };
    let hmac_key = b"0123456789abcdef0123456789abcdef";

    let keys_order_1 = vec![b"zebra".to_vec(), b"apple".to_vec(), b"banana".to_vec()];
    let keys_order_2 = vec![b"apple".to_vec(), b"banana".to_vec(), b"zebra".to_vec()];

    let proof1 = DeletionProof::create(
        scope.clone(),
        keys_order_1,
        TxId::new(100),
        vec![],
        vec![],
        hmac_key,
    )
    .expect("creation of proof1 should succeed");

    let proof2 = DeletionProof::create(
        scope,
        keys_order_2,
        TxId::new(100),
        vec![],
        vec![],
        hmac_key,
    )
    .expect("creation of proof2 should succeed");

    assert_eq!(
        proof1.deleted_keys_hash, proof2.deleted_keys_hash,
        "Different initial input key orderings MUST yield identical deleted_keys_hash values"
    );
}

#[test]
fn test_v2_hmac_proof_creation_and_verification_regression() {
    let scope = DeletionScope::Document {
        doc_id: DocId::new(42),
        tenant_id: TenantId::try_new(10).expect("valid tenant_id"),
    };
    let hmac_key = b"0123456789abcdef0123456789abcdef";
    let layer_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)
            .expect("valid layer proof");

    let proof = DeletionProof::create(
        scope,
        vec![b"k1".to_vec(), b"k2".to_vec()],
        TxId::new(100),
        vec![layer_proof],
        vec![ExcludedScope::LlmParameterMemory],
        hmac_key,
    )
    .expect("v2 proof creation should succeed");

    assert_eq!(proof.signature_version, 2);
    assert!(
        proof.verify(hmac_key).expect("verify should return Ok"),
        "v2 HMAC proof MUST verify successfully"
    );
}
