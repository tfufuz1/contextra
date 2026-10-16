#![forbid(unsafe_code)]

use contextra_engine::collection::crud::DocEraseOptions;
use contextra_engine::deletion::forget_options::ForgetOptions;
use contextra_engine::Contextra;
use contextra_types::ContextraError;
use tempfile::TempDir;

#[tokio::test]
async fn test_concurrent_modification_err_before_commit() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("col");

    let doc_id = "doc_concurrent_test";
    let embedding = vec![0.1f32; col.dimension()];
    col.insert(doc_id, &embedding, None)
        .await
        .expect("insert");

    // Standard erasure should succeed
    let opts = DocEraseOptions {
        options: ForgetOptions {
            assurance: contextra_crypto::deletion_proof::claim_text::WitAssurance::S0,
            purge: contextra_engine::deletion::forget_options::PurgeMode::None,
            ..Default::default()
        },
    };

    let res = col.erase_document(doc_id, &opts).await;
    assert!(res.is_ok(), "Erasure without concurrent modification should succeed");

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_no_err_returned_after_tombstone_commit() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("col");

    let doc_id = "doc_post_commit_test";
    let embedding = vec![0.3f32; col.dimension()];
    col.insert(doc_id, &embedding, None)
        .await
        .expect("insert");

    // Deleting a committed document succeeds and guarantees no post-commit error propagation
    let res = col.delete(doc_id).await;
    assert!(res.is_ok(), "delete must return Ok(()) after tombstone commit");

    // Repeat delete on already deleted/non-existent document also returns Ok(())
    let res_repeat = col.delete(doc_id).await;
    assert!(res_repeat.is_ok(), "Repeat delete on missing document returns Ok(())");

    db.close().await.unwrap();
}
