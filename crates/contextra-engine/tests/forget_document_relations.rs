#![forbid(unsafe_code)]

use contextra_engine::collection::crud::DocEraseOptions;
use contextra_engine::Contextra;
use contextra_types::DocId;
use tempfile::TempDir;

#[tokio::test]
async fn test_bilateral_relations_removed_unrelated_preserved() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");

    let col = db.collection("default").await.expect("col");

    let doc_a = "doc_rel_a";
    let doc_b = "doc_rel_b";
    let doc_c = "doc_rel_c";

    let doc_a_id = DocId::from_key(doc_a).unwrap();
    let doc_b_id = DocId::from_key(doc_b).unwrap();
    let doc_c_id = DocId::from_key(doc_c).unwrap();

    let embedding = vec![0.1f32; 768];

    col.insert(doc_a, &embedding, None).await.expect("insert a");
    col.insert(doc_b, &embedding, None).await.expect("insert b");
    col.insert(doc_c, &embedding, None).await.expect("insert c");

    col.relate_bidirectional(doc_a, doc_b, "links").await.expect("relate a-b");
    col.relate_bidirectional(doc_b, doc_c, "links").await.expect("relate b-c");

    col.link_memories(doc_a_id, doc_b_id, contextra_types::domain::LinkRelation::Elaborates)
        .await
        .expect("link a-b");
    col.link_memories(doc_b_id, doc_c_id, contextra_types::domain::LinkRelation::Elaborates)
        .await
        .expect("link b-c");

    // Erase Document A
    let opts = DocEraseOptions::default();
    let erasure = col.erase_document(doc_a, &opts).await.expect("erase a");

    println!("ERASURE KEYS for A: {:?}", erasure.keys.iter().map(|k| String::from_utf8_lossy(k)).collect::<Vec<_>>());

    let get_a = col.get(doc_a).await.expect("get a");
    println!("GET A AFTER ERASE: {:?}", get_a);

    // Verify A is erased
    assert!(get_a.is_none(), "Document A must be deleted");

    // Verify B and C are preserved
    assert!(col.get(doc_b).await.expect("get b").is_some());
    assert!(col.get(doc_c).await.expect("get c").is_some());

    // Verify B -> C links are intact
    let b_links = col.get_links(doc_b_id).await.expect("get_links b");
    assert!(!b_links.is_empty(), "Links from B to C must remain intact");

    db.close().await.unwrap();
}
