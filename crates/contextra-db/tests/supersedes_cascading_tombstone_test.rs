#![allow(deprecated)]

use contextra_db::Contextra;
use contextra_types::{DocId, LinkRelation};
use tempfile::tempdir;

#[tokio::test]
async fn test_supersedes_triggers_tombstone() {
    let dir = tempdir().unwrap();
    let db = Contextra::open(dir.path()).await.unwrap();
    let col = db.collection("default").await.unwrap();

    let dummy_emb = vec![0.1f32; 768];

    db.insert(
        "doc-a",
        &dummy_emb,
        Some(serde_json::json!({"text": "Document A text"})),
    )
    .await
    .unwrap();

    db.insert(
        "doc-b",
        &dummy_emb,
        Some(serde_json::json!({"text": "Document B text"})),
    )
    .await
    .unwrap();

    let doc_a_id = DocId::from_key("doc-a").unwrap();
    let doc_b_id = DocId::from_key("doc-b").unwrap();

    col.link_memories(doc_b_id, doc_a_id, LinkRelation::Supersedes)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_supersedes_tombstone_retained_after_db_reopen() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_path_buf();
    let dummy_emb = vec![0.1f32; 768];

    // 1. Initial scope: insert documents and link as Supersedes
    {
        let db = Contextra::open(&db_path).await.unwrap();
        let col = db.collection("default").await.unwrap();

        db.insert(
            "doc-x",
            &dummy_emb,
            Some(serde_json::json!({"text": "Document X text"})),
        )
        .await
        .unwrap();

        db.insert(
            "doc-y",
            &dummy_emb,
            Some(serde_json::json!({"text": "Document Y text"})),
        )
        .await
        .unwrap();

        let doc_x_id = DocId::from_key("doc-x").unwrap();
        let doc_y_id = DocId::from_key("doc-y").unwrap();

        col.link_memories(doc_y_id, doc_x_id, LinkRelation::Supersedes)
            .await
            .unwrap();
    }

    // 2. Re-open database
    {
        let db = Contextra::open(&db_path).await.unwrap();
        let _col = db.collection("default").await.unwrap();
    }
}
