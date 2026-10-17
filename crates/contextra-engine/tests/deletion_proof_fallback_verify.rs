#![cfg(not(feature = "encryption-at-rest"))]
#![cfg(not(loom))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::{DeletionLayer, LayerCleanupProof, LayerWitness, WitnessEvidence};
use contextra_engine::{DeletionProof, DeletionScope};
use contextra_types::{CollectionId, TenantId, TxId};

fn test_proof_and_key() -> (DeletionProof, Vec<u8>) {
    let key = b"test-hmac-key-32-bytes-long-1234".to_vec();
    let scope = DeletionScope::Collection {
        collection_id: CollectionId(1),
        tenant_id: TenantId::try_new(42).expect("valid tenant"),
    };
    let deleted_keys = vec![b"key1".to_vec(), b"key2".to_vec()];
    let tx_id = TxId(100);
    let witness = LayerWitness::issue_from_storage_layer(
        DeletionLayer::SsTableAllLevels,
        &deleted_keys,
        tx_id,
        WitnessEvidence::SstableScan { files_scanned: 1 },
    )
    .expect("witness");
    let layer_proofs = vec![LayerCleanupProof::from_witness(&witness).expect("layer proof")];

    let proof = DeletionProof::create(scope, deleted_keys, tx_id, layer_proofs, vec![], &key)
        .expect("create proof");
    (proof, key)
}

#[test]
fn test_fresh_proof_with_correct_key_verifies() -> contextra_types::Result<()> {
    let (proof, key) = test_proof_and_key();
    assert!(
        proof.verify(&key)?,
        "Fresh proof with correct key must verify to true"
    );
    Ok(())
}

#[test]
fn test_wrong_key_returns_false() -> contextra_types::Result<()> {
    let (proof, _key) = test_proof_and_key();
    let wrong_key = b"wrong-hmac-key-32-bytes-long-123".to_vec();
    assert!(
        !proof.verify(&wrong_key)?,
        "Verification with wrong key must return false"
    );
    Ok(())
}

#[test]
fn test_bit_flip_in_signature_or_deleted_keys_hash_returns_false() -> contextra_types::Result<()> {
    let (proof, key) = test_proof_and_key();

    let mut tampered_sig = proof.clone();
    if let Some(byte) = tampered_sig.signature.get_mut(0) {
        *byte ^= 0x01;
    }
    assert!(
        !tampered_sig.verify(&key)?,
        "Tampered signature must return false"
    );

    let mut tampered_hash = proof.clone();
    tampered_hash.deleted_keys_hash[0] ^= 0x01;
    assert!(
        !tampered_hash.verify(&key)?,
        "Tampered deleted_keys_hash must return false"
    );

    Ok(())
}

#[test]
fn test_modified_deleted_after_tx_returns_false() -> contextra_types::Result<()> {
    let (proof, key) = test_proof_and_key();
    let mut tampered_tx = proof;
    tampered_tx.deleted_after_tx = TxId(101);
    assert!(
        !tampered_tx.verify(&key)?,
        "Modified tx_id must return false"
    );
    Ok(())
}

#[test]
fn test_modified_scope_different_tenant_returns_false() -> contextra_types::Result<()> {
    let (proof, key) = test_proof_and_key();
    let mut tampered_scope = proof;
    tampered_scope.scope = DeletionScope::Collection {
        collection_id: CollectionId(1),
        tenant_id: TenantId::try_new(999).expect("different tenant"),
    };
    assert!(
        !tampered_scope.verify(&key)?,
        "Modified tenant_id scope must return false"
    );
    Ok(())
}

#[test]
fn test_unsupported_signature_versions_return_false() -> contextra_types::Result<()> {
    let (proof, key) = test_proof_and_key();

    let mut v1_proof = proof.clone();
    v1_proof.signature_version = 1;
    assert!(
        !v1_proof.verify(&key)?,
        "signature_version 1 must return false"
    );

    let mut v3_proof = proof;
    v3_proof.signature_version = 3;
    assert!(
        !v3_proof.verify(&key)?,
        "signature_version 3 must return false"
    );

    Ok(())
}

#[test]
fn test_modified_or_added_wal_chain_receipt_returns_false() -> contextra_types::Result<()> {
    let (proof, key) = test_proof_and_key();

    let mut with_receipt = proof;
    with_receipt.wal_chain_receipt = Some([0xAA; 32]);
    assert!(
        !with_receipt.verify(&key)?,
        "Adding wal_chain_receipt after signing must return false"
    );

    Ok(())
}

#[test]
fn test_serde_json_roundtrip_remains_verifiable() -> contextra_types::Result<()> {
    let (proof, key) = test_proof_and_key();
    let json_str = serde_json::to_string(&proof)
        .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

    let roundtrip_proof: DeletionProof = serde_json::from_str(&json_str)
        .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

    assert!(
        roundtrip_proof.verify(&key)?,
        "serde_json roundtripped proof must verify successfully"
    );
    Ok(())
}

#[tokio::test]
async fn collection_delete_does_not_overclaim_layer() -> contextra_types::Result<()> {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let ctx = contextra_engine::Contextra::open_with_config(temp_dir.path(), contextra_engine::ContextraConfig::default())
        .await?;

    let tenant_id = TenantId::try_new(42).expect("valid tenant");
    let _col = ctx.collection_for_tenant("test_col", tenant_id).await?;

    let proof_key = vec![0x42u8; 32];
    let proof = ctx.drop_collection("test_col", tenant_id, &proof_key).await?;

    let covered_layers = proof.covered_layers;
    assert_eq!(
        covered_layers.len(),
        1,
        "Must contain exactly 1 layer proof for store API scan"
    );
    assert_eq!(
        &covered_layers[0],
        &DeletionLayer::LsmMemtable,
        "Must be labelled LsmMemtable for stage S0 logical scan"
    );
    assert_ne!(
        &covered_layers[0],
        &DeletionLayer::SsTableAllLevels,
        "Must NOT overclaim SsTableAllLevels"
    );

    Ok(())
}

#[test]
fn collection_delete_propagates_collection_id_error() {
    let res = CollectionId::try_new(0);
    assert!(
        res.is_err(),
        "CollectionId::try_new(0) must return error without unwrap_or fallback"
    );
}
