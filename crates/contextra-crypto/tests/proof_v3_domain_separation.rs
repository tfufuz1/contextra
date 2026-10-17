// FILE-CONTEXT (STAND 2026-10-13, ZWECK: Verifikation der Domänentrennung v3 vs v4 und Cross-Version Replay-Schutz, INVARIANTEN: INV-DELETION-1, INV-DELETION-2, F7-Domain-Separation)

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, DurabilityProof,
    ExcludedScope, LayerCleanupProof,
};
use contextra_types::{DocId, TenantId, TxId};

#[test]
#[allow(deprecated)]
fn test_proof_v3_v4_domain_tag_prefix_separation() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(100).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId::from(42u64),
        tenant_id,
    };

    let layer_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0).unwrap(),
    ];

    let durability = DurabilityProof::new(100);

    let v3_proof = DeletionProof::create_v3(
        scope,
        vec![b"doc_key_1".to_vec()],
        TxId(100),
        layer_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        Some(&durability),
        keypair.signing_key(),
    )
    .expect("v3 proof creation should succeed with valid layer cleanup proofs");

    assert_eq!(v3_proof.signature_version, 3);
    assert!(
        v3_proof.verify(&keypair.verifying_key).unwrap(),
        "v3 proof must verify under its correct v3 domain tag"
    );
}

#[test]
#[allow(deprecated)]
fn test_cross_version_replay_prevention() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(100).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId::from(42u64),
        tenant_id,
    };

    let layer_proofs =
        vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap()];

    let durability = DurabilityProof::new(100);

    let v3_proof = DeletionProof::create_v3(
        scope,
        vec![b"doc_key_1".to_vec()],
        TxId(100),
        layer_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        Some(&durability),
        keypair.signing_key(),
    )
    .expect("v3 proof creation should succeed");

    // Replay attack: attempt to pass a v3 signature off as a v4 proof.
    let mut replayed_as_v4 = v3_proof.clone();
    replayed_as_v4.signature_version = 4;

    let verify_result = replayed_as_v4.verify(&keypair.verifying_key);
    match verify_result {
        Ok(valid) => {
            assert!(
                !valid,
                "Cross-version replay MUST fail: v3 signature cannot verify under v4 domain tag"
            );
        }
        Err(err) => {
            // Rejection via error is also acceptable fail-closed behavior
            assert!(
                err.to_string().contains("VerificationKey")
                    || err.to_string().contains("InvalidProofSignature")
                    || err.to_string().contains("version")
            );
        }
    }
}

#[test]
fn test_v3_physical_cleanup_layer_claim_enforcement() {
    // Attempting to construct LayerCleanupProof with remaining live entries MUST fail (INV-DELETION-1)
    let invalid_layer_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 5);
    assert!(
        invalid_layer_proof.is_err(),
        "LayerCleanupProof MUST refuse construction if remaining live entries > 0"
    );

    // Valid zero-remaining construction succeeds
    let valid_layer_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0);
    assert!(
        valid_layer_proof.is_ok(),
        "LayerCleanupProof construction MUST succeed when remaining live entries == 0"
    );
}
