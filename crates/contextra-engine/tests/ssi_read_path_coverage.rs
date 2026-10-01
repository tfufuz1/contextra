#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig};
use contextra_testkit::InMemoryStorageEngine;
use contextra_types::{DocId, Result};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

/// Test 1: Single Get Tracked / get_at_seq_tracked SSI Write-Skew Prevention
#[tokio::test]
async fn test_ssi_coverage_get_tracked() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.insert("res_a", &emb, Some(json!({ "val": 100 }))).await?;
    col.insert("res_b", &emb, Some(json!({ "val": 200 }))).await?;

    let tx1 = col.begin_transaction()?;
    let tx2 = col.begin_transaction()?;

    // Both transactions read res_a and res_b using get_tracked
    let _a1 = col.get_tracked(tx1.tx_id, "res_a").await?;
    let _b1 = col.get_tracked(tx1.tx_id, "res_b").await?;

    let _a2 = col.get_tracked(tx2.tx_id, "res_a").await?;
    let _b2 = col.get_tracked(tx2.tx_id, "res_b").await?;

    // Tx1 writes res_a, Tx2 writes res_b
    col.update_op(&tx1, "res_a", &emb, Some(json!({ "val": 110 })))
        .await?;
    col.update_op(&tx2, "res_b", &emb, Some(json!({ "val": 210 })))
        .await?;

    tx1.commit().await?;
    let tx2_res = tx2.commit().await;

    assert!(
        tx2_res.is_err(),
        "Tx2 commit must fail due to write skew on overlapping tracked read sets"
    );

    Ok(())
}

/// Test 2: Link Memories SSI Write-Skew Prevention
#[tokio::test]
async fn test_ssi_coverage_link_memories() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.insert("mem_x", &emb, Some(json!({ "title": "Memory X" })))
        .await?;
    col.insert("mem_y", &emb, Some(json!({ "title": "Memory Y" })))
        .await?;

    let doc_x = DocId::from_key("mem_x")?;
    let doc_y = DocId::from_key("mem_y")?;

    // Tx1 linking mem_x -> mem_y while Tx2 updates mem_x concurrently
    let tx1 = col.begin_transaction()?;
    let doc1_tracked = col.get_tracked(tx1.tx_id, "mem_x").await?;
    assert!(doc1_tracked.is_some());

    col.update("mem_x", &emb, Some(json!({ "title": "Memory X Updated" })))
        .await?;

    // link_memories internally calls get_at_seq_tracked
    col.link_memories(
        doc_x,
        doc_y,
        contextra_types::domain::LinkRelation::Elaborates,
    )
    .await?;

    col.update_op(
        &tx1,
        "mem_y",
        &emb,
        Some(json!({ "title": "Tx1 Overwrite Y" })),
    )
    .await?;

    // Tx1 commit should fail because mem_x was concurrently updated
    let tx1_res = tx1.commit().await;
    assert!(
        tx1_res.is_err(),
        "Tx1 commit must fail because mem_x was concurrently updated after Tx1 read it"
    );

    Ok(())
}

/// Test 3: Document Importance Update SSI Write-Skew
#[tokio::test]
async fn test_ssi_coverage_update_importance() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.insert("doc_imp", &emb, Some(json!({ "title": "Importance Test" })))
        .await?;

    let tx1 = col.begin_transaction()?;
    let _doc = col.get_tracked(tx1.tx_id, "doc_imp").await?;

    // Concurrently update importance (which executes tracked get and commit)
    col.update_document_importance("doc_imp", 0.9, "model_v1")
        .await?;

    // Now Tx1 tries to update doc_imp and commit
    col.update_op(
        &tx1,
        "doc_imp",
        &emb,
        Some(json!({ "title": "Tx1 Overwrite" })),
    )
    .await?;

    let tx1_res = tx1.commit().await;
    assert!(
        tx1_res.is_err(),
        "Tx1 commit must fail due to concurrent importance update committed on doc_imp"
    );

    Ok(())
}

/// Test 4: Startup behavior for engines without SSI tracking support.
/// Asserts that Collection initialization with an in-memory storage engine returning supports_ssi_tracking() = false
/// proceeds without panicking or returning an error.
#[tokio::test]
async fn test_non_ssi_storage_engine_startup_behavior() -> Result<()> {
    use contextra_ports::StorageEngine;

    let non_ssi_storage = Arc::new(InMemoryStorageEngine::new());
    assert!(
        !non_ssi_storage.supports_ssi_tracking(),
        "InMemoryStorageEngine must return false for supports_ssi_tracking()"
    );

    let index = Arc::new(
        contextra_vector::HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            distance_metric: contextra_types::DistanceMetric::Cosine,
            ..Default::default()
        })
        .expect("Failed to create HnswIndex"),
    );
    let graph = Arc::new(contextra_graph::CsrGraph::new());
    let next_tx = Arc::new(std::sync::atomic::AtomicU64::new(1));

    let col = contextra_engine::Collection::new(
        "non_ssi_test".to_string(),
        non_ssi_storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    assert_eq!(col.name(), "non_ssi_test");
    assert!(
        !col.storage().supports_ssi_tracking(),
        "Engine collection created with non-SSI storage engine proceeds normally without rejecting startup"
    );

    Ok(())
}
