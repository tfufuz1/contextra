#![cfg(not(loom))]
#![forbid(unsafe_code)]

//! Integration test verifying that dropping a collection completely clears/resets
//! HNSW vector index, BM25 text index, and storage RAM state (WP-L3.1 & C5.2).

use contextra_engine::Contextra;
use contextra_ports::VectorIndex;

#[tokio::test]
async fn test_drop_collection_resets_ram_state_completely() {
    let dir = tempfile::tempdir().unwrap();
    let db = Contextra::open(dir.path()).await.unwrap();
    let col = db.collection("drop_ram_test").await.unwrap();

    let doc_id = "doc_drop_1";
    let text = "ram_reset_verification_needle";
    let mut embedding = vec![0.0f32; col.dimension()];
    embedding[0] = 1.0;

    // Insert document with text
    let db_tx = col.begin_transaction().unwrap();
    col.insert_op(
        &db_tx,
        doc_id,
        &embedding,
        Some(serde_json::json!({ "text": text })),
    )
    .await
    .unwrap();
    db_tx.commit().await.unwrap();

    // Confirm RAM states before drop
    let vec_len_before = col.vector_index().len().await;
    assert_eq!(
        vec_len_before, 1,
        "Vector index RAM should have 1 vector before drop"
    );

    let text_hits_before = col
        .text_index_ref()
        .search_bm25(text, 10, None)
        .await
        .unwrap();
    assert_eq!(
        text_hits_before.len(),
        1,
        "Text index RAM should have 1 hit before drop"
    );

    // Execute drop_collection
    col.drop_collection().await.unwrap();

    // Confirm RAM states after drop
    let vec_len_after = col.vector_index().len().await;
    assert_eq!(
        vec_len_after, 0,
        "Vector index RAM must be 0 after drop_collection"
    );

    let text_hits_after = col
        .text_index_ref()
        .search_bm25(text, 10, None)
        .await
        .unwrap();
    assert!(
        text_hits_after.is_empty(),
        "BM25 text index RAM postings must be empty after drop_collection"
    );

    db.close().await.unwrap();
}
