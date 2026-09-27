use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope,
    GraphRepairAttestation, LayerCleanupProof,
};
use contextra_types::{error::HnswDeletionError, ContextraError, DocId, TenantId, TxId};

#[test]
fn test_create_v3_with_empty_graph_repair_for_hnsw_layer_fails() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId::new(42),
        tenant_id,
    };

    let layer_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
    ];

    // Attempt to create v3 proof with empty graph_repair slice when HnswIndex is covered
    let res = DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec()],
        TxId::new(100),
        layer_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[], // Empty graph_repair -> MUST FAIL
        keypair.signing_key(),
    );

    assert!(
        res.is_err(),
        "create_v3 MUST fail when HnswIndex is in covered_layers and graph_repair is empty"
    );

    match res {
        Err(ContextraError::GraphRepairFailed(HnswDeletionError::VerificationFailed { remaining_pointers })) => {
            assert_eq!(remaining_pointers, 1);
        }
        other => panic!("Expected ContextraError::GraphRepairFailed(VerificationFailed), got {:?}", other),
    }
}

#[test]
fn test_create_v3_with_non_empty_graph_repair_for_hnsw_layer_succeeds() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId::new(42),
        tenant_id,
    };

    let layer_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
    ];

    let graph_repair = vec![GraphRepairAttestation {
        doc_id: DocId::new(42),
        verified_no_ghost_pointers: true,
        attested_at: 1700000000,
    }];

    let res = DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec()],
        TxId::new(100),
        layer_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &graph_repair,
        keypair.signing_key(),
    );

    assert!(
        res.is_ok(),
        "create_v3 MUST succeed when HnswIndex is in covered_layers and graph_repair is provided"
    );

    let proof = res.unwrap();
    assert!(proof.verify(&keypair.verifying_key).unwrap());
}
