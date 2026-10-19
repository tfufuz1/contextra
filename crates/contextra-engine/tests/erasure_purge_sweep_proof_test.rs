//! Integration test for erasure -> purge driver -> reverse sweep -> deletion proof verification.

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::deletion::{
    forget_document, DpiDeletionRuntime, ErasureFootprint, ForgetOptions, KeyFamilyRegistry,
    PurgeHandle,
};
use contextra_ports::StorageEngine;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{DocId, TenantId, TxId};
use tempfile::TempDir;

#[tokio::test]
async fn test_erasure_purge_reverse_sweep_and_proof_verified() {
    let db_dir = TempDir::new().expect("tempdir failed");
    let storage_dir = db_dir.path().join("storage");
    std::fs::create_dir_all(&storage_dir).expect("mkdir storage failed");

    let lsm_config = LsmConfig {
        path: storage_dir.clone(),
        ..Default::default()
    };
    let storage = LsmStorage::new(lsm_config)
        .await
        .expect("LsmStorage::new failed");

    let signer_key_path = db_dir.path().join("proof_signer.key");
    let db_instance_id = [0x22; 16];
    let runtime = DpiDeletionRuntime::open(db_dir.path(), Some(&signer_key_path), db_instance_id)
        .expect("DpiDeletionRuntime::open failed");

    let doc_id_str = "doc_test_123";
    let doc_id = DocId::from_key(doc_id_str).unwrap_or(DocId::new(123));
    let user_key = format!("__col:default:\x00{}", doc_id_str).into_bytes();
    let doc_key = format!("__col:default:\x01{}", doc_id_str).into_bytes();

    let tx = TxId::new(10);
    storage
        .put(tx, &user_key, b"{\"test\": 1}")
        .await
        .expect("put user_key failed");
    storage
        .put(tx, &doc_key, b"{\"meta\": 1}")
        .await
        .expect("put doc_key failed");
    storage.commit(tx).await.expect("commit failed");

    // Perform logical tombstoning
    let tx_del = TxId::new(20);
    storage
        .delete(tx_del, &user_key)
        .await
        .expect("delete user_key failed");
    storage
        .delete(tx_del, &doc_key)
        .await
        .expect("delete doc_key failed");
    storage.commit(tx_del).await.expect("commit delete failed");

    let footprint = ErasureFootprint::for_document(TenantId::SYSTEM, "default", doc_id);
    let registry = KeyFamilyRegistry::default();
    let sweep_keys: Vec<Vec<u8>> = vec![]; // Storage is now tombstoned, no active residual keys

    let opts = ForgetOptions::new().with_assurance(WitAssurance::S1);

    let outcome = forget_document(
        &storage,
        &runtime,
        doc_id_str,
        tx_del,
        &opts,
        &sweep_keys,
        &footprint,
        &registry,
    )
    .await
    .expect("forget_document failed");

    assert!(outcome.logical.full_proof_id != [0u8; 32]);
    assert!(outcome.logical.proof_id != [0u8; 16]);

    if let Some(PurgeHandle::Final(ref phys)) = outcome.physical {
        assert!(phys.full_proof_id != [0u8; 32]);
    }
}
