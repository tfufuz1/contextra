#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Barrier;

#[tokio::test]
async fn test_mvcc_isolation_uncommitted_writes_invisible() -> contextra_types::Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Arc::new(Contextra::open_with_config(tmp_dir.path(), config).await?);
    let col = db.collection("default").await?;

    // Step 1: Insert initial document doc1
    db.insert("doc_iso_1", &[0.1, 0.2, 0.3, 0.4], Some(json!({ "v": 1 })))
        .await?;

    // Record snapshot seq_no after initial commit
    let seq_before = db.create_snapshot().await?;

    // Use barriers to synchronize concurrent tasks
    let barrier1 = Arc::new(Barrier::new(2));
    let barrier2 = Arc::new(Barrier::new(2));

    let db_col = col.clone();
    let b1_tx = barrier1.clone();
    let b2_tx = barrier2.clone();

    // Task A: Performs an uncommitted insert_op inside a transaction
    let task_a = tokio::spawn(async move {
        let tx_a = db_col
            .begin_transaction()
            .expect("begin_transaction failed");
        db_col
            .insert_op(
                &tx_a,
                "doc_iso_1",
                &[0.9, 0.9, 0.9, 0.9],
                Some(json!({ "v": 2, "status": "uncommitted" })),
            )
            .await
            .expect("insert_op failed");

        db_col
            .insert_op(
                &tx_a,
                "doc_iso_2",
                &[0.5, 0.5, 0.5, 0.5],
                Some(json!({ "v": 99 })),
            )
            .await
            .expect("insert_op doc2 failed");

        // Signal Task B that uncommitted write is staged
        b1_tx.wait().await;

        // Wait for Task B to verify snapshot isolation before committing
        b2_tx.wait().await;

        tx_a.commit().await.expect("commit failed");
    });

    // Task B: Reads at snapshot point concurrently while Tx A is uncommitted
    let db_reader = db.clone();
    let b1_rx = barrier1.clone();
    let b2_rx = barrier2.clone();

    let task_b = tokio::spawn(async move {
        // Wait for Task A to stage its uncommitted write
        b1_rx.wait().await;

        // Read doc_iso_1 at snapshot - must see v: 1, NOT uncommitted v: 2
        let doc_snapshot = db_reader
            .get_at_snapshot("doc_iso_1", seq_before)
            .await
            .expect("get_at_snapshot failed");

        assert!(
            doc_snapshot.is_some(),
            "doc_iso_1 must be visible at snapshot point"
        );
        let doc_v1 = doc_snapshot.unwrap();
        assert_eq!(
            doc_v1
                .metadata
                .as_ref()
                .and_then(|m| m.get("v"))
                .and_then(|v| v.as_u64()),
            Some(1),
            "Concurrent read MUST NOT see uncommitted write from Task A"
        );

        // Uncommitted new key must be invisible at snapshot point
        let doc_uncommitted_key = db_reader
            .get_at_snapshot("doc_iso_2", seq_before)
            .await
            .expect("get_at_snapshot doc2 failed");

        assert!(
            doc_uncommitted_key.is_none(),
            "Uncommitted document 'doc_iso_2' MUST NOT be visible before Tx A commits"
        );

        // Signal Task A to proceed with commit
        b2_rx.wait().await;
    });

    // Await both tasks
    let (res_a, res_b) = tokio::join!(task_a, task_b);
    res_a.unwrap();
    res_b.unwrap();

    // Post-commit check: read committed state
    let doc_committed = db.get("doc_iso_1").await?;
    assert!(doc_committed.is_some());
    let doc_v2 = doc_committed.unwrap();
    assert_eq!(
        doc_v2
            .metadata
            .as_ref()
            .and_then(|m| m.get("v"))
            .and_then(|v| v.as_u64()),
        Some(2),
        "Committed write from Task A MUST be visible post-commit"
    );

    let doc_new_key = db.get("doc_iso_2").await?;
    assert!(
        doc_new_key.is_some(),
        "Newly inserted document 'doc_iso_2' MUST be visible after Tx A commit"
    );

    // Clean close
    let db_owned = Arc::try_unwrap(db)
        .map_err(|_| contextra_types::ContextraError::Internal("Arc unwrap failed".into()))?;
    db_owned.close().await?;
    Ok(())
}
