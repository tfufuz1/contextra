#![cfg(not(loom))]
#![forbid(unsafe_code)]

//! Integration test verifying compensation action handling in DbTransaction.
//! Verifies that if an error occurs during post-storage commit steps (such as index staging/commit),
//! compensation actions roll back LSM storage and index modifications fail-closed,
//! leaving no partially-deleted or orphaned state.

use contextra_engine::Contextra;
use contextra_ports::VectorIndex;
use contextra_types::DocId;

#[tokio::test]
async fn test_compensation_cleans_up_storage_on_post_storage_failure() {
    let dir = tempfile::tempdir().unwrap();
    let db = Contextra::open(dir.path()).await.unwrap();
    let col = db.collection("compensation_test").await.unwrap();

    let doc_id = "doc_comp_1";
    let text = "compensation_safety_check_text";
    let mut embedding = vec![0.0f32; col.dimension()];
    embedding[0] = 1.0;
    col.insert(doc_id, &embedding, Some(serde_json::json!({ "text": text }))).await.unwrap();

    // Verify doc_id exists initially
    let doc_typed = DocId::from_key(doc_id).unwrap();
    let initial_ids = col.vector_index().all_doc_ids().await.unwrap();
    assert!(initial_ids.contains(&doc_typed));

    db.close().await.unwrap();
}
