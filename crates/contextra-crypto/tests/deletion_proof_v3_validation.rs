#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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
        Err(ContextraError::GraphRepairFailed(HnswDeletionError::VerificationFailed {
            remaining_pointers,
        })) => {
            assert_eq!(remaining_pointers, 1);
        }
        other => panic!(
            "Expected ContextraError::GraphRepairFailed(VerificationFailed), got {:?}",
            other
        ),
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

    let layer_proofs =
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap()];

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

#[test]
fn test_valid_signature_with_unknown_version_is_rejected_fail_closed() {
    use contextra_crypto::ed25519_proof::{DeletionProofError, SignatureVersion};
    use contextra_crypto::error::CryptoError;

    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId::new(42),
        tenant_id,
    };

    let layer_proofs =
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()];

    // 1. Create a valid Ed25519 Version 3 proof
    let valid_v3_proof = DeletionProof::create_v3(
        scope.clone(),
        vec![b"key1".to_vec()],
        TxId::new(100),
        layer_proofs.clone(),
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    // Verify original v3 proof is valid
    assert_eq!(valid_v3_proof.signature_version, 3);
    assert_eq!(
        valid_v3_proof.signature_version_typed(),
        Ok(SignatureVersion::V3)
    );
    assert!(valid_v3_proof.verify(&keypair.verifying_key).unwrap());
    assert!(valid_v3_proof.verify_external(&keypair.verifying_key).is_ok());

    // 2. Test that mutating signature_version to any unknown byte (0, 4, 128, 255)
    // causes immediate failure before signature validation is attempted.
    for unknown_byte in [0, 4, 128, 255] {
        let mut tampered_version_proof = valid_v3_proof.clone();
        tampered_version_proof.signature_version = unknown_byte;

        // signature_version_typed() returns explicit DeletionProofError::UnsupportedVersion
        assert_eq!(
            tampered_version_proof.signature_version_typed(),
            Err(DeletionProofError::UnsupportedVersion(unknown_byte))
        );

        // verify() MUST fail with an Err containing the version message (cannot be bypassed by valid sig)
        let verify_res = tampered_version_proof.verify(&keypair.verifying_key);
        assert!(
            matches!(verify_res, Err(ContextraError::Internal(ref msg)) if msg.contains(&format!("Unsupported DeletionProof signature_version: {unknown_byte}"))),
            "verify() MUST reject unknown version byte {unknown_byte} fail-closed"
        );

        // verify_external() MUST fail with UnsupportedProofVersion
        let ext_res = tampered_version_proof.verify_external(&keypair.verifying_key);
        assert!(
            matches!(ext_res, Err(CryptoError::UnsupportedProofVersion(v)) if v == unknown_byte),
            "verify_external() MUST return UnsupportedProofVersion for unknown byte {unknown_byte}"
        );
    }

    // 3. Test HMAC Version 2 proof with valid signature but unknown mutated version
    let hmac_key = b"hmac_key_32_bytes_long_secret!!";
    let valid_v2_proof = DeletionProof::create(
        scope,
        vec![b"key1".to_vec()],
        TxId::new(100),
        layer_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        hmac_key,
    )
    .unwrap();

    assert_eq!(valid_v2_proof.signature_version, 2);
    assert!(valid_v2_proof.verify(hmac_key).unwrap());

    for unknown_byte in [0, 4, 128, 255] {
        let mut tampered_v2 = valid_v2_proof.clone();
        tampered_v2.signature_version = unknown_byte;

        let verify_res = tampered_v2.verify(hmac_key);
        assert!(
            matches!(verify_res, Err(ContextraError::Internal(ref msg)) if msg.contains(&format!("Unsupported DeletionProof signature_version: {unknown_byte}"))),
            "verify() for HMAC proof MUST reject unknown version byte {unknown_byte} fail-closed"
        );
    }
}
