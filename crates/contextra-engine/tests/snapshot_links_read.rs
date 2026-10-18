#![forbid(unsafe_code)]

use contextra_engine::Contextra;
use contextra_types::domain::LinkRelation;
use contextra_types::DocId;
use tempfile::TempDir;

#[tokio::test]
async fn test_snapshot_links_read_isolation() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("col");

    let doc_a = "doc_snap_a";
    let doc_b = "doc_snap_b";
    let doc_c = "doc_snap_c";

    let doc_a_id = DocId::from_key(doc_a).unwrap();
    let doc_b_id = DocId::from_key(doc_b).unwrap();
    let doc_c_id = DocId::from_key(doc_c).unwrap();

    let embedding = vec![0.1f32; 768];

    col.insert(doc_a, &embedding, None).await.expect("insert a");
    col.insert(doc_b, &embedding, None).await.expect("insert b");
    col.insert(doc_c, &embedding, None).await.expect("insert c");

    col.link_memories(doc_a_id, doc_b_id, LinkRelation::Associates)
        .await
        .expect("link a -> b");

    let snap1_seq = col.snapshot_seq().await.expect("snapshot seq");

    // Link a -> c after snap1_seq
    col.link_memories(doc_a_id, doc_c_id, LinkRelation::Associates)
        .await
        .expect("link a -> c");

    // Check links at snapshot snap1_seq
    let links_at_snap1 = col
        .get_links_at_snapshot(doc_a_id, snap1_seq)
        .await
        .expect("get_links_at_snapshot");

    assert_eq!(
        links_at_snap1.len(),
        1,
        "Expected 1 link at snap1_seq, found {}",
        links_at_snap1.len()
    );
    assert_eq!(
        links_at_snap1[0].target, doc_b_id,
        "Link at snap1_seq should point to doc_b"
    );

    // Check latest get_links() returns both b and c
    let links_latest = col.get_links(doc_a_id).await.expect("get_links latest");
    assert_eq!(
        links_latest.len(),
        2,
        "Latest links should contain 2 links (b and c)"
    );
}
