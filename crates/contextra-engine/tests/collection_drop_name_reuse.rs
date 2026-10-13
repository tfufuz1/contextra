#![cfg(not(loom))]
#![forbid(unsafe_code)]

//! Integration test verifying collection drop name reuse and clean re-creation (C5.2).

use contextra_engine::Contextra;

#[tokio::test]
async fn test_drop_collection_name_reuse_yields_fresh_empty_collection() {
    let dir = tempfile::tempdir().unwrap();
    let db = Contextra::open(dir.path()).await.unwrap();

    let col_name = "reusable_col";
    let col = db.collection(col_name).await.unwrap();

    let doc_id = "doc_reuse_1";
    let text = "legacy_residual_data_needle";
    let mut embedding = vec![0.0f32; col.dimension()];
    embedding[0] = 1.0;

    col.insert(doc_id, &embedding, Some(serde_json::json!({ "text": text }))).await.unwrap();
    assert_eq!(col.len().await, 1);

    // Drop collection
    col.drop_collection().await.unwrap();

    // Re-create collection with same name
    let col_new = db.collection(col_name).await.unwrap();

    // Verify col_new is completely empty and contains no old residual data
    assert_eq!(
        col_new.len().await,
        0,
        "Re-created collection must be empty"
    );

    let hits = col_new.text_index_ref().search_bm25(text, 10, None).await.unwrap();
    assert!(
        hits.is_empty(),
        "Re-created collection must not match text from dropped collection"
    );

    db.close().await.unwrap();
}
