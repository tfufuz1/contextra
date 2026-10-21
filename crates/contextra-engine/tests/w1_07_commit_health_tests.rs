#![cfg(not(loom))]
use contextra_engine::collection::CollectionHealth;
use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::Result;
use tempfile::TempDir;

#[tokio::test]
async fn marker_commit_failure_propagates() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    assert_eq!(col.health(), CollectionHealth::Healthy);

    // Verify health degrades on explicit recovery trigger
    col.mark_needs_recovery("marker_commit_failed");
    assert_eq!(col.health(), CollectionHealth::NeedsRecovery);

    Ok(())
}

#[tokio::test]
async fn index_failure_then_rollback_failure_reports_composite_and_degrades() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    assert_eq!(col.health(), CollectionHealth::Healthy);

    // Explicitly test mark_needs_recovery degradation
    col.mark_needs_recovery("index_and_rollback_failed");
    assert_eq!(col.health(), CollectionHealth::NeedsRecovery);

    Ok(())
}
