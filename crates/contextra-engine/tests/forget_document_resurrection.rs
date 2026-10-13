#![forbid(unsafe_code)]

use contextra_engine::collection::crud::DocEraseOptions;
use contextra_engine::Contextra;
use tempfile::TempDir;

#[tokio::test]
async fn test_rewrite_higher_tx_survives() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");

    let col = db.collection("default").await.expect("col");

    let doc_id = "doc_rewrite_1";
    let embedding_v1 = vec![0.1f32; 768];
    let embedding_v2 = vec![0.2f32; 768];

    col.insert(doc_id, &embedding_v1, None).await.expect("insert v1");

    let opts = DocEraseOptions::default();
    col.erase_document(doc_id, &opts).await.expect("erase v1");

    assert!(col.get(doc_id).await.expect("get").is_none());

    // Re-insert same doc_id at higher tx
    col.insert(doc_id, &embedding_v2, None).await.expect("re-insert v2");

    let re_read = col.get(doc_id).await.expect("get").expect("found v2");
    assert_eq!(re_read.id, doc_id, "Re-written document with higher TxId MUST survive");

    db.close().await.unwrap();
}
