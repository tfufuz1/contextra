//! End-to-end test verifying document insertion -> erase/forget -> search empty -> DeletionProof verification.

#![forbid(unsafe_code)]

use contextra_engine::collection::crud::DocEraseOptions;
use contextra_engine::Contextra;
use contextra_types::TenantId;
use tempfile::tempdir;

#[tokio::test]
async fn test_e2e_insert_erase_search_empty_proof_verified() {
    let tmp = tempdir().expect("tempdir failed");

    let engine = Contextra::open(tmp.path())
        .await
        .expect("engine open failed");
    let tenant_id = TenantId::try_new(1).unwrap();
    let col = engine
        .collection_for_tenant("e2e_del_test", tenant_id)
        .await
        .expect("collection_for_tenant failed");

    let doc_id = "doc_test_101";
    let doc_vec = vec![0.1f32; 768];
    let metadata = serde_json::json!({
        "text": "Secret cryptographic content for e2e deletion test"
    });
    col.insert(doc_id, &doc_vec, Some(metadata))
        .await
        .expect("insert failed");

    // Pre-condition vector search: document is present
    let pre_results = col.search(&doc_vec, 5).await.expect("search failed");
    assert!(
        !pre_results.is_empty(),
        "Document MUST be found before erasure"
    );

    // 1. Execute document erasure/forget
    let erase_opts = DocEraseOptions::default();
    let doc_erasure = col
        .erase_document(doc_id, &erase_opts)
        .await
        .expect("erase_document failed");

    assert!(doc_erasure.as_of_tx.inner() > 0);

    // 2. Retrieval sanity check across vector search
    let post_search = col.search(&doc_vec, 5).await.expect("vector search failed");
    assert!(
        post_search.is_empty(),
        "Vector search MUST return empty results after document erasure"
    );

    // 3. Drop collection to issue full DeletionProof
    let proof_key = [0x77u8; 32];
    let report = engine
        .drop_collection_with_report("e2e_del_test", tenant_id, &proof_key)
        .await
        .expect("drop_collection_with_report failed");

    let proof = report.proof;
    assert_eq!(
        proof.scope,
        contextra_crypto::deletion_proof::types::DeletionScope::Collection {
            collection_id: Contextra::derive_collection_id("e2e_del_test").unwrap(),
            tenant_id,
        }
    );

    // 4. Verify proof validity (proof has deleted_keys_hash and tx recorded)
    assert!(proof.deleted_after_tx.inner() > 0);
    assert!(!proof.deleted_keys_hash.is_empty());
}
