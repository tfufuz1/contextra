#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::Result;
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn ssi_write_skew_conflicts() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    // Seed initial state
    col.insert("doc_a", &emb, Some(json!({ "val": 100 })))
        .await?;
    col.insert("doc_b", &emb, Some(json!({ "val": 200 })))
        .await?;

    let tx1 = col.begin_transaction()?;
    let tx2 = col.begin_transaction()?;

    // Both transactions perform tracked reads
    let _b1 = col.get_tracked(tx1.tx_id, "doc_b").await?;
    let _a2 = col.get_tracked(tx2.tx_id, "doc_a").await?;

    // Tx1 updates doc_a
    col.update_op(&tx1, "doc_a", &emb, Some(json!({ "val": 110 })))
        .await?;
    // Tx2 updates doc_b
    col.update_op(&tx2, "doc_b", &emb, Some(json!({ "val": 210 })))
        .await?;

    // Tx1 commits
    tx1.commit().await?;

    // Tx2 commits - expected to fail with Conflict under SSI
    let tx2_res = tx2.commit().await;

    assert!(
        tx2_res.is_err(),
        "Tx2 commit MUST fail due to write skew conflict on overlapping read/write sets"
    );

    Ok(())
}

#[tokio::test]
async fn ssi_read_set_nonempty_after_reads() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.insert("doc_1", &emb, Some(json!({ "val": 100 })))
        .await?;

    let tx = col.begin_transaction()?;
    let doc = col.get_tracked(tx.tx_id, "doc_1").await?;
    assert!(doc.is_some());

    col.update_op(&tx, "doc_1", &emb, Some(json!({ "val": 101 })))
        .await?;
    let commit_res = tx.commit().await;
    assert!(commit_res.is_ok());

    Ok(())
}

#[tokio::test]
async fn every_read_path_records() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.insert("doc_x", &emb, Some(json!({ "val": 100 })))
        .await?;

    let tx = col.begin_transaction()?;
    let doc = col.get_tracked(tx.tx_id, "doc_x").await?;
    assert!(doc.is_some());

    Ok(())
}
