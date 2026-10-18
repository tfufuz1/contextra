#![cfg(not(loom))]
#![forbid(unsafe_code)]

//! Integration test verifying WP-L3.1 transaction semantics:
//! (a) insert(A); delete(A) in a single transaction leaves zero remnants in LSM, HNSW, or BM25 indices.
//! (b) delete(A); insert(A) in a single transaction leaves A with NEW content and zero old remnants.
//! (c) rollback after insert(A); delete(A) leaves state unchanged.

use contextra_engine::Contextra;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::DocId;

#[tokio::test]
async fn test_insert_and_delete_same_tx_leaves_no_remnants() {
    let dir = tempfile::tempdir().unwrap();
    let db = Contextra::open(dir.path()).await.unwrap();
    let col = db.collection("tx_test").await.unwrap();

    let doc_id = "doc_alpha";
    let text = "unique_alpha_needle_token";
    let mut embedding = vec![0.0f32; col.dimension()];
    embedding[0] = 1.0;

    // Begin transaction and execute insert_op then delete_op in the SAME transaction
    let mut db_tx = col.begin_transaction().unwrap();
    col.insert_op(
        &db_tx,
        doc_id,
        &embedding,
        Some(serde_json::json!({ "text": text })),
    )
    .await
    .unwrap();
    col.delete_op(&mut db_tx, doc_id).await.unwrap();
    db_tx.commit().await.unwrap();

    // 1. Storage check: key must not exist
    let user_key = col.user_key_prefix();
    let raw_entries = col.storage().scan_prefix(&user_key).await.unwrap();
    for (k, _v) in &raw_entries {
        let key_str = String::from_utf8_lossy(k);
        assert!(
            !key_str.contains(doc_id),
            "Storage still contains user key for deleted doc_id '{doc_id}': {key_str}"
        );
    }

    // 2. Vector index check: no doc_id in HNSW
    let doc_typed = DocId::from_key(doc_id).unwrap();
    let active_ids = col.vector_index().all_doc_ids().await.unwrap();
    assert!(
        !active_ids.contains(&doc_typed),
        "Vector index still contains doc_id '{doc_id}' after insert; delete in same tx"
    );

    // 3. Text index check: BM25 search yields zero hits
    let text_hits = col
        .text_index_ref()
        .search_bm25("unique_alpha_needle_token", 10, None)
        .await
        .unwrap();
    assert!(
        text_hits.is_empty(),
        "BM25 text index still returned hits for doc_id '{doc_id}' after insert; delete in same tx: {text_hits:?}"
    );

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_delete_then_insert_same_tx_overwrites_with_new_content() {
    let dir = tempfile::tempdir().unwrap();
    let db = Contextra::open(dir.path()).await.unwrap();
    let col = db.collection("tx_test_overwrite").await.unwrap();

    let doc_id = "doc_beta";
    let old_text = "old_beta_legacy_token";
    let new_text = "new_beta_fresh_token";
    let mut embedding_1 = vec![0.0f32; col.dimension()];
    embedding_1[0] = 1.0;
    let mut embedding_2 = vec![0.0f32; col.dimension()];
    embedding_2[1] = 1.0;

    // Initial insert and commit
    col.insert(
        doc_id,
        &embedding_1,
        Some(serde_json::json!({ "text": old_text })),
    )
    .await
    .unwrap();

    // Now in a single transaction: delete then insert new
    let db_tx = col.begin_transaction().unwrap();
    let mut db_tx_mut = db_tx;
    col.delete_op(&mut db_tx_mut, doc_id).await.unwrap();
    col.insert_op(
        &db_tx_mut,
        doc_id,
        &embedding_2,
        Some(serde_json::json!({ "text": new_text })),
    )
    .await
    .unwrap();
    db_tx_mut.commit().await.unwrap();

    // Verify old text yields no hits
    let old_hits = col
        .text_index_ref()
        .search_bm25(old_text, 10, None)
        .await
        .unwrap();
    assert!(
        old_hits.is_empty(),
        "BM25 text index still matched old text '{old_text}' after delete; insert in same tx"
    );

    // Verify new text yields hits
    let new_hits = col
        .text_index_ref()
        .search_bm25(new_text, 10, None)
        .await
        .unwrap();
    assert_eq!(
        new_hits.len(),
        1,
        "BM25 text index should return 1 hit for new text '{new_text}'"
    );

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_rollback_after_insert_delete_same_tx_restores_prior_state() {
    let dir = tempfile::tempdir().unwrap();
    let db = Contextra::open(dir.path()).await.unwrap();
    let col = db.collection("tx_test_rollback").await.unwrap();

    let doc_id = "doc_gamma";
    let initial_text = "initial_gamma_stable_token";
    let temp_text = "transient_gamma_temp_token";
    let mut embedding = vec![0.0f32; col.dimension()];
    embedding[0] = 1.0;

    col.insert(
        doc_id,
        &embedding,
        Some(serde_json::json!({ "text": initial_text })),
    )
    .await
    .unwrap();

    // Begin transaction: insert temp, then delete, then ROLLBACK
    let mut db_tx = col.begin_transaction().unwrap();
    col.insert_op(
        &db_tx,
        doc_id,
        &embedding,
        Some(serde_json::json!({ "text": temp_text })),
    )
    .await
    .unwrap();
    col.delete_op(&mut db_tx, doc_id).await.unwrap();
    db_tx.rollback().await.unwrap();

    // Verify initial text is still present
    let hits = col
        .text_index_ref()
        .search_bm25(initial_text, 10, None)
        .await
        .unwrap();
    assert_eq!(
        hits.len(),
        1,
        "Initial text '{initial_text}' should still be present after tx rollback"
    );

    db.close().await.unwrap();
}
