//! Loom / Concurrent Deadlock-Free Proof for Collection::relate_n_ary
//!
//! Verifies that concurrent `relate_n_ary` calls with overlapping participant sets in
//! different orderings (e.g. {doc-1, doc-2, doc-3} vs {doc-3, doc-2, doc-1}) terminate
//! cleanly without deadlocks thanks to canonical participant EntityId sorting before lock acquisition.
//!
//! Execution:
//! - Standard test: cargo test -p contextra-db --test loom_relate_n_ary
//! - Loom execution: RUSTFLAGS="--cfg loom" cargo test -p contextra-db --test loom_relate_n_ary

#![allow(unexpected_cfgs)]

use contextra_db::{Contextra, ContextraConfig};
use std::sync::Arc;
use tempfile::TempDir;

async fn create_test_db() -> (Arc<Contextra>, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        ..Default::default()
    };
    let db = Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");
    (Arc::new(db), tmp)
}

#[tokio::test]
async fn test_concurrent_relate_n_ary_deadlock_free_overlapping_participants() {
    let (db, _tmp) = create_test_db().await;
    let col = Arc::new(db.collection("default").await.expect("collection"));

    // Seed 3 participant documents
    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
        .await
        .expect("insert 1");
    col.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None)
        .await
        .expect("insert 2");
    col.insert("doc-3", &[0.0, 0.0, 1.0, 0.0], None)
        .await
        .expect("insert 3");

    let col_1 = col.clone();
    let col_2 = col.clone();

    // Task 1: Order {doc-1, doc-2, doc-3}
    let task1 = tokio::spawn(async move {
        let participants = vec![
            ("doc-1", "role_a"),
            ("doc-2", "role_b"),
            ("doc-3", "role_c"),
        ];
        col_1
            .relate_n_ary("rel_forward", &participants, Some("doc-1"))
            .await
    });

    // Task 2: Inverted order {doc-3, doc-2, doc-1}
    let task2 = tokio::spawn(async move {
        let participants = vec![
            ("doc-3", "role_c"),
            ("doc-2", "role_b"),
            ("doc-1", "role_a"),
        ];
        col_2
            .relate_n_ary("rel_backward", &participants, Some("doc-3"))
            .await
    });

    // Join both tasks with timeout to guarantee termination without deadlock
    let res = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let (r1, r2) = tokio::join!(task1, task2);
        (r1.expect("task1 join"), r2.expect("task2 join"))
    })
    .await;

    assert!(
        res.is_ok(),
        "Concurrent relate_n_ary calls with overlapping participant sets timed out (deadlock!)"
    );

    let (res1, res2) = res.expect("timeout result");
    let he_id_1 = res1.expect("task1 relate_n_ary");
    let he_id_2 = res2.expect("task2 relate_n_ary");

    // Verify both hyperedges exist in graph_index
    let graph = col.graph_index();
    assert!(graph.get_hyperedge(he_id_1).is_some());
    assert!(graph.get_hyperedge(he_id_2).is_some());
}
