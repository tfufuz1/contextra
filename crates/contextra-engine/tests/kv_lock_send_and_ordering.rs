#![cfg(not(loom))]
use std::sync::Arc;
use tempfile::TempDir;

fn assert_send<F: Send>(_: F) {}

async fn create_test_db() -> (Arc<contextra_engine::Contextra>, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = contextra_engine::ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        ..Default::default()
    };
    let db = contextra_engine::Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");
    (Arc::new(db), tmp)
}

#[tokio::test]
async fn test_collection_futures_are_send() {
    let (db, _tmp) = create_test_db().await;
    let col = db.collection("default").await.expect("collection");
    let val = serde_json::json!({"key": "val"});

    // Assert that futures returned by collection operations are Send
    assert_send(col.relate_n_ary("test", &[("doc1", "role1"), ("doc2", "role2")], None));
    assert_send(col.relate("doc1", "doc2", "knows"));
    assert_send(col.insert("doc1", &[1.0, 0.0, 0.0, 0.0], None));
    assert_send(col.put_kv("key1", &val));
    assert_send(col.delete("key1"));
}

#[tokio::test]
async fn test_concurrent_lock_acquisition_deadlock_free() {
    let (db, _tmp) = create_test_db().await;
    let col = Arc::new(db.collection("default").await.expect("collection"));

    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
        .await
        .expect("insert 1");
    col.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None)
        .await
        .expect("insert 2");
    col.insert("doc-3", &[0.0, 0.0, 1.0, 0.0], None)
        .await
        .expect("insert 3");

    let col1 = col.clone();
    let col2 = col.clone();

    let task1 = tokio::spawn(async move {
        let participants = vec![
            ("doc-1", "role_a"),
            ("doc-2", "role_b"),
            ("doc-3", "role_c"),
        ];
        col1.relate_n_ary("rel_forward", &participants, Some("doc-1"))
            .await
    });

    let task2 = tokio::spawn(async move {
        let participants = vec![
            ("doc-3", "role_c"),
            ("doc-2", "role_b"),
            ("doc-1", "role_a"),
        ];
        col2.relate_n_ary("rel_backward", &participants, Some("doc-3"))
            .await
    });

    let res = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let (r1, r2) = tokio::join!(task1, task2);
        (r1.expect("task1 join"), r2.expect("task2 join"))
    })
    .await;

    assert!(
        res.is_ok(),
        "Concurrent relate_n_ary calls timed out (deadlock!)"
    );
    let (r1, r2) = res.unwrap();
    assert!(r1.is_ok());
    assert!(r2.is_ok());
}
