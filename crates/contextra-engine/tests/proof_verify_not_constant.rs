use contextra_engine::{DeletionLayer, DeletionProof, DeletionScope, LayerCleanupProof};
use contextra_types::{TenantId, TxId};

#[test]
fn test_proof_verify_with_wrong_key_fails() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(1).unwrap(),
    };
    let real_key = b"correct_key_32_bytes_long_secret!";
    let wrong_key = b"wrong___key_32_bytes_long_secret!";

    let proof = DeletionProof::create(
        scope,
        vec![b"key1".to_vec()],
        TxId(1),
        vec![
            #[allow(deprecated)]
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        ],
        vec![],
        real_key,
    )
    .unwrap();

    let is_valid = proof.verify(wrong_key.as_slice()).unwrap_or(false);
    assert!(
        !is_valid,
        "Verification with wrong key must return false or Err, but stub returned Ok(true)"
    );
}
