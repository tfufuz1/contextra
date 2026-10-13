// FILE-CONTEXT (STAND: 2026-10-13, ZWECK: Cross-Version-Replay und Domänentrennungs-Test für Proof v3 und v4, INVARIANTEN: INV-DELETION-1, Fail-Closed)

use contextra_crypto::deletion_proof::{DeletionProof, DeletionScope, DurabilityProof};
use contextra_types::{DocId, TenantId, TxId};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

#[test]
fn test_cross_version_replay_v3_signature_rejected_as_v4() {
    let mut signing_key_bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut signing_key_bytes);
    let signing_key = SigningKey::from_bytes(&signing_key_bytes);
    let verifying_key = signing_key.verifying_key();

    let scope = DeletionScope::Document {
        doc_id: DocId(102),
        tenant_id: TenantId::try_new(1).expect("valid tenant"),
    };
    let deleted_keys = vec![b"key1".to_vec(), b"key2".to_vec()];
    let deleted_after_tx = TxId(100);
    let durability = DurabilityProof::new(100);

    #[allow(deprecated)]
    let proof_v3 = DeletionProof::create_v3(
        scope,
        deleted_keys,
        deleted_after_tx,
        vec![],
        vec![],
        2000,
        &[],
        Some(&durability),
        &signing_key,
    )
    .expect("v3 proof creation");

    // Authentic v3 proof must verify successfully with verifying_key
    let v3_verified = proof_v3
        .verify(&verifying_key)
        .expect("v3 verification succeeded");
    assert!(v3_verified, "Authentic v3 proof must verify under v3");

    // Cross-version replay attack: alter signature_version to V4 (4)
    let mut cross_version_v4_proof = proof_v3.clone();
    cross_version_v4_proof.signature_version = 4;

    let v4_replayed_verified = cross_version_v4_proof
        .verify(&verifying_key)
        .expect("v4 verification call completed");
    assert!(
        !v4_replayed_verified,
        "Cross-version replay: v3 signature MUST fail when evaluated as v4 due to domain separation tag mismatch"
    );
}
