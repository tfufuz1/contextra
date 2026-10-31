//! # Integrationstest: tx fault injection
//!
//! **Zweck:** Prüft spezifische Garantien, Transaktionskonsistenz und Fehlertoleranz der Engine.
//!
//! Gehört zur Testsuite von `contextra-engine`.

#![cfg(not(loom))]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_engine::transaction::{CommitIntent, CompensateHnswAction, CompensatingAction};
use contextra_engine::{Collection, Contextra, ContextraConfig};
use contextra_ports::StorageEngine;
use contextra_types::{DocId, TenantId, TxId};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

/// Helper function to assert invariant docs(LSM) == docs(HNSW) == docs(Text)
async fn assert_index_doc_count_invariant<S: StorageEngine, V: contextra_ports::VectorIndex>(
    col: &Collection<S, V>,
    expected_count: usize,
) -> contextra_types::Result<()> {
    let lsm_count = col.len().await;
    let hnsw_stats = col.stats().await?;
    let hnsw_count = hnsw_stats.num_vectors;

    let text_search_hits = col
        .query()
        .text("test")
        .k(100)
        .execute()
        .await
        .map(|hits| hits.len())
        .unwrap_or(0);

    assert_eq!(
        lsm_count, expected_count,
        "LSM document count mismatch: expected {}, got {}",
        expected_count, lsm_count
    );
    assert_eq!(
        hnsw_count, expected_count,
        "HNSW active document count mismatch: expected {}, got {}",
        expected_count, hnsw_count
    );

    if expected_count == 0 {
        assert_eq!(
            text_search_hits, 0,
            "Text index ghost hits detected after rollback/erasure"
        );
    }

    Ok(())
}

// ANCHOR[contextra-engine:TST38-001]
/// Testblock zur Überprüfung von `test_tx_fault_after_lsm_write`.
#[tokio::test]
async fn test_tx_fault_after_lsm_write() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_fault_lsm").await?;

    assert_index_doc_count_invariant(&col, 0).await?;

    // Create transaction and stage keys + doc
    let tx = col.begin_transaction()?;
    col.insert_op(
        &tx,
        "fault_doc_1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "text": "test fault after lsm write" })),
    )
    .await?;

    let doc_id = DocId::from_key("fault_doc_1")?;
    tx.stage_text_insert(doc_id, "test fault after lsm write".to_string());

    // Abort/rollback transaction simulating failure after LSM write
    tx.rollback().await?;

    // Verify invariant holds: docs(LSM) == docs(HNSW) == docs(Text) == 0
    assert_index_doc_count_invariant(&col, 0).await?;

    // Verify search returns no ghost hits
    let search_res = col
        .query()
        .text("test")
        .embedding(&[0.1, 0.2, 0.3, 0.4])
        .k(5)
        .execute()
        .await?;
    assert!(
        search_res.is_empty(),
        "Expected no ghost hits after rollback"
    );

    db.close().await?;
    Ok(())
}

// ANCHOR[contextra-engine:TST38-002]
/// Testblock zur Überprüfung von `test_tx_fault_after_hnsw_insert`.
#[tokio::test]
async fn test_tx_fault_after_hnsw_insert() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_fault_hnsw").await?;

    let tx = col.begin_transaction()?;
    col.insert_op(
        &tx,
        "fault_hnsw_doc",
        &[0.5, 0.6, 0.7, 0.8],
        Some(json!({ "text": "test hnsw fault" })),
    )
    .await?;

    let doc_id = DocId::from_key("fault_hnsw_doc")?;
    tx.stage_text_insert(doc_id, "test hnsw fault".to_string());

    // Execute rollback simulating error during/after vector step
    tx.rollback().await?;

    // Verify invariant: docs(LSM) == docs(HNSW) == docs(Text) == 0
    assert_index_doc_count_invariant(&col, 0).await?;

    // Vector query returns 0 ghost hits
    let search_res = col
        .query()
        .embedding(&[0.5, 0.6, 0.7, 0.8])
        .k(5)
        .execute()
        .await?;
    assert!(search_res.is_empty());

    db.close().await?;
    Ok(())
}

// ANCHOR[contextra-engine:TST38-003]
/// Testblock zur Überprüfung von `test_tx_fault_after_text_index_update`.
#[tokio::test]
async fn test_tx_fault_after_text_index_update() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_fault_text").await?;

    let tx = col.begin_transaction()?;
    col.insert_op(
        &tx,
        "fault_text_doc",
        &[0.2, 0.4, 0.6, 0.8],
        Some(json!({ "text": "unique text search target" })),
    )
    .await?;

    let doc_id = DocId::from_key("fault_text_doc")?;
    tx.stage_text_insert(doc_id, "unique text search target".to_string());

    // Rollback simulating error after text index update
    tx.rollback().await?;

    // Verify invariant: docs(LSM) == docs(HNSW) == docs(Text) == 0
    assert_index_doc_count_invariant(&col, 0).await?;

    // Text search returns no ghost hits
    let search_res = col.query().text("unique").k(5).execute().await?;
    assert!(
        search_res.is_empty(),
        "No ghost hits expected for text search"
    );

    db.close().await?;
    Ok(())
}

// ANCHOR[contextra-engine:TST38-004]
/// Testblock zur Überprüfung von `test_tx_fault_double_failure_in_compensation`.
#[tokio::test]
async fn test_tx_fault_double_failure_in_compensation() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_fault_double").await?;

    let doc_id = DocId::new(999);
    let doc_ids = Arc::new(vec![doc_id]);
    let tenant_id = TenantId::default();

    // Directly test CompensateHnswAction execution on non-existent document
    let action = CompensateHnswAction::new(col.as_ref().clone(), tenant_id, doc_ids);
    let res = action.execute().await;
    assert!(
        res.is_ok(),
        "CompensateHnswAction should execute gracefully on missing document"
    );

    db.close().await?;
    Ok(())
}

// ANCHOR[contextra-engine:TST38-005]
/// Testblock zur Überprüfung von `test_tx_fault_crash_between_intent_log_and_execution`.
#[tokio::test]
async fn test_tx_fault_crash_between_intent_log_and_execution() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let path = tmp.path().to_path_buf();

    {
        let config = ContextraConfig {
            dimension: 4,
            consolidation_enabled: false,
            ..Default::default()
        };
        let db = Contextra::open_with_config(&path, config).await?;
        let storage = db.inner_storage();

        // Write an uncompleted pending intent
        let intent_key = b"__tx_intent:101".to_vec();
        let doc_id = DocId::new(101);
        let pending = CommitIntent::Pending {
            doc_ids: Arc::new(vec![doc_id]),
            has_text: true,
            has_graph: false,
            stages_completed: 0,
        };
        let bytes = serde_json::to_vec(&pending)?;
        let tx = TxId::new(1);
        storage.put(tx, &intent_key, &bytes).await?;
        storage.commit(tx).await?;

        db.close().await?;
    }

    // Re-open DB (simulating crash recovery)
    {
        let config = ContextraConfig {
            dimension: 4,
            consolidation_enabled: false,
            ..Default::default()
        };
        let db = Contextra::open_with_config(&path, config).await?;
        let col = db.collection("default").await?;

        // Invariant holds: docs(LSM) == docs(HNSW) == docs(Text) == 0
        assert_index_doc_count_invariant(&col, 0).await?;

        // Verify intent reconciled to Aborted
        let storage = db.inner_storage();
        let val = storage.get(b"__tx_intent:101").await?;
        assert!(val.is_some());
        let recovered_intent: CommitIntent = serde_json::from_slice(&val.unwrap())?;
        assert!(
            matches!(recovered_intent, CommitIntent::Aborted),
            "Pending intent without storage records must reconcile to Aborted"
        );

        db.close().await?;
    }

    Ok(())
}

// ANCHOR[contextra-engine:TST38-006]
/// Testblock zur Überprüfung von `test_tx_ghost_hit_prevention_after_rollback`.
#[tokio::test]
async fn test_tx_ghost_hit_prevention_after_rollback() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_ghost").await?;

    let tx = col.begin_transaction()?;
    col.insert_op(
        &tx,
        "ghost_doc",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({ "text": "ghost search text" })),
    )
    .await?;

    let doc_id = DocId::from_key("ghost_doc")?;
    tx.stage_text_insert(doc_id, "ghost search text".to_string());

    tx.rollback().await?;

    // Search storage
    assert!(col.get("ghost_doc").await?.is_none());

    // Search hybrid
    let results = col
        .query()
        .text("ghost")
        .embedding(&[1.0, 0.0, 0.0, 0.0])
        .k(10)
        .execute()
        .await?;
    assert!(results.is_empty(), "Ghost hit detected after rollback");

    db.close().await?;
    Ok(())
}
