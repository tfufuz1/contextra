//! # Integrationstest: multi index transaction
//!
//! **Zweck:** Prüft spezifische Garantien, Transaktionskonsistenz und Fehlertoleranz der Engine.
//!
//! Gehört zur Testsuite von `contextra-engine`.

#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{DocId, Edge, Entity, EntityId};
use serde_json::json;
use tempfile::TempDir;

// ANCHOR[contextra-engine:TST27-001]
/// Testblock zur Überprüfung von `test_db_transaction_multi_index_staging_commit_and_rollback`.
#[tokio::test]
async fn test_db_transaction_multi_index_staging_commit_and_rollback() -> contextra_types::Result<()>
{
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_multi_index_col").await?;

    // 1. Commit Scenario: Staging document insert, graph entity, and graph edge, then committing
    let mut tx_commit = col.begin_transaction()?;

    col.insert_op(
        &mut tx_commit,
        "tx_doc_commit",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "text": "Transaktions-Text für Commit-Test" })),
    )
    .await?;

    let doc_id_committed = DocId::from_key("tx_doc_commit")?;
    tx_commit.stage_text_insert(
        doc_id_committed,
        "Transaktions-Text für Commit-Test".to_string(),
    );

    // Execute commit
    tx_commit.commit().await?;

    // Verify committed document in storage
    let doc_comm = col.get("tx_doc_commit").await?;
    assert!(
        doc_comm.is_some(),
        "Committed transaction document must exist in storage"
    );

    // 2. Rollback Scenario: Staging text, then rolling back
    let mut tx_rollback = col.begin_transaction()?;

    col.insert_op(
        &mut tx_rollback,
        "tx_doc_rollback",
        &[0.5, 0.6, 0.7, 0.8],
        Some(json!({ "text": "Transaktions-Text für Rollback-Test" })),
    )
    .await?;

    let doc_id_rolled_back = DocId::from_key("tx_doc_rollback")?;
    tx_rollback.stage_text_insert(
        doc_id_rolled_back,
        "Transaktions-Text für Rollback-Test".to_string(),
    );

    // Execute rollback
    tx_rollback.rollback().await?;

    // Verify document DOES NOT exist in storage after rollback
    let rb_doc = col.get("tx_doc_rollback").await?;
    assert!(
        rb_doc.is_none(),
        "Storage must NOT contain rolled-back transaction document"
    );

    db.close().await?;
    Ok(())
}
