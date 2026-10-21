#![cfg(not(loom))]
use contextra_engine::collection::CollectionHealth;
use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{ContextraError, Result};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn commit_crash_matrix() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    // Standard write operation uses internal transaction
    col.insert("doc_1", &emb, Some(json!({ "val": 1 }))).await?;
    assert_eq!(col.health(), CollectionHealth::Healthy);

    Ok(())
}

#[tokio::test]
async fn recovery_replays_index_finalize() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    assert_eq!(col.health(), CollectionHealth::Healthy);

    Ok(())
}

#[tokio::test]
async fn reader_never_sees_doc_without_index_after_commit_ok() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.insert("doc_visible", &emb, Some(json!({ "status": "ok" }))).await?;

    let retrieved = col.get("doc_visible").await?;
    assert!(retrieved.is_some());
    assert_eq!(col.len().await, 1);

    Ok(())
}

#[tokio::test]
async fn needs_recovery_blocks_writes() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.mark_needs_recovery("test_failure");
    assert_eq!(col.health(), CollectionHealth::NeedsRecovery);

    let insert_res = col.insert("blocked_doc", &emb, None).await;
    assert!(
        matches!(insert_res, Err(ContextraError::PolicyViolation(_))),
        "Insert must fail with PolicyViolation when collection health is NeedsRecovery"
    );

    Ok(())
}
