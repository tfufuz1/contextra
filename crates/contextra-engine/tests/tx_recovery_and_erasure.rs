//! # Integrationstest: tx recovery and erasure
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

use contextra_engine::transaction::{recover_pending_intents, CommitIntent};
use contextra_engine::{Collection, Contextra, ContextraConfig};
use contextra_ports::StorageEngine;
use contextra_types::{DocId, TxId};
use serde_json::json;
use std::sync::atomic::AtomicU64;
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
        let text_search_hits = col
            .query()
            .text("test")
            .k(100)
            .execute()
            .await
            .map(|hits| hits.len())
            .unwrap_or(0);
        assert_eq!(
            text_search_hits, 0,
            "Text index ghost hits detected after rollback/erasure"
        );
    }

    Ok(())
}

// ANCHOR[contextra-engine:TST39-001]
/// Testblock zur Überprüfung von `test_tx_no_orphaned_intents_after_commit_and_rollback`.
#[tokio::test]
async fn test_tx_no_orphaned_intents_after_commit_and_rollback() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_intents_test").await?;

    // Successful transaction
    let tx1 = col.begin_transaction()?;
    col.insert_op(
        &tx1,
        "doc_commit",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "text": "committed document" })),
    )
    .await?;
    tx1.commit().await?;

    // Rolled-back transaction
    let tx2 = col.begin_transaction()?;
    col.insert_op(
        &tx2,
        "doc_abort",
        &[0.5, 0.6, 0.7, 0.8],
        Some(json!({ "text": "aborted document" })),
    )
    .await?;
    tx2.rollback().await?;

    // Scan intent namespace in storage
    let storage = db.inner_storage();
    let pending_scan = storage.scan_prefix(b"__tx_intent:").await?;

    for (k, v) in pending_scan {
        if let Ok(intent) = serde_json::from_slice::<CommitIntent>(&v) {
            assert!(
                !matches!(intent, CommitIntent::Pending { .. }),
                "No orphaned Pending intent key {:?} should remain after completion",
                String::from_utf8_lossy(&k)
            );
        }
    }

    db.close().await?;
    Ok(())
}

// ANCHOR[contextra-engine:TST39-002]
/// Testblock zur Überprüfung von `test_tx_erasure_across_all_indices`.
#[tokio::test]
async fn test_tx_erasure_across_all_indices() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_erasure").await?;

    // Insert doc
    col.insert(
        "erased_doc",
        &[0.2, 0.4, 0.6, 0.8],
        Some(json!({ "text": "erasure target text" })),
    )
    .await?;
    assert_index_doc_count_invariant(&col, 1).await?;

    // Erase/delete
    col.delete("erased_doc").await?;

    // Invariant holds after erasure: docs(LSM) == docs(HNSW) == docs(Text) == 0
    assert_index_doc_count_invariant(&col, 0).await?;

    // Search query returns zero hits
    let results = col
        .query()
        .text("erasure")
        .embedding(&[0.2, 0.4, 0.6, 0.8])
        .k(5)
        .execute()
        .await?;
    assert!(
        results.is_empty(),
        "Erased document must not appear in search results"
    );

    db.close().await?;
    Ok(())
}

// ANCHOR[contextra-engine:TST39-003]
/// Testblock zur Überprüfung von `test_tx_erasure_tombstone_consistency`.
#[tokio::test]
async fn test_tx_erasure_tombstone_consistency() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_tombstone").await?;

    col.insert(
        "keep_doc",
        &[0.1, 0.1, 0.1, 0.1],
        Some(json!({ "text": "keep this document" })),
    )
    .await?;
    col.insert(
        "erase_doc",
        &[0.9, 0.9, 0.9, 0.9],
        Some(json!({ "text": "erase this document" })),
    )
    .await?;

    assert_index_doc_count_invariant(&col, 2).await?;

    // Erase one
    col.delete("erase_doc").await?;

    assert_index_doc_count_invariant(&col, 1).await?;

    let search_kept = col.get("keep_doc").await?;
    assert!(search_kept.is_some());

    let search_erased = col.get("erase_doc").await?;
    assert!(search_erased.is_none());

    db.close().await?;
    Ok(())
}

// ANCHOR[contextra-engine:TST39-004]
/// Testblock zur Überprüfung von `test_tx_tenant_isolation_during_rollback`.
#[tokio::test]
async fn test_tx_tenant_isolation_during_rollback() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    let col_a = db.collection("tenant_col_a").await?;
    let col_b = db.collection("tenant_col_b").await?;

    // Tenant B inserts document
    col_b
        .insert(
            "tenant_b_doc",
            &[0.3, 0.3, 0.3, 0.3],
            Some(json!({ "text": "tenant b text" })),
        )
        .await?;
    assert_index_doc_count_invariant(&col_b, 1).await?;

    // Tenant A starts transaction and rolls back
    let tx_a = col_a.begin_transaction()?;
    col_a
        .insert_op(
            &tx_a,
            "tenant_a_doc",
            &[0.7, 0.7, 0.7, 0.7],
            Some(json!({ "text": "tenant a text" })),
        )
        .await?;
    tx_a.rollback().await?;

    // Tenant A doc rolled back
    assert_index_doc_count_invariant(&col_a, 0).await?;

    // Tenant B doc remains untouched
    assert_index_doc_count_invariant(&col_b, 1).await?;
    assert!(col_b.get("tenant_b_doc").await?.is_some());

    db.close().await?;
    Ok(())
}

// ANCHOR[contextra-engine:TST39-005]
/// Testblock zur Überprüfung von `test_tx_multi_document_rollback`.
#[tokio::test]
async fn test_tx_multi_document_rollback() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_multi_doc").await?;

    let tx = col.begin_transaction()?;

    for i in 1..=5 {
        let doc_key = format!("multi_doc_{}", i);
        let vec = vec![i as f32 * 0.1, 0.2, 0.3, 0.4];
        let txt = format!("multi document content {}", i);
        col.insert_op(&tx, &doc_key, &vec, Some(json!({ "text": txt })))
            .await?;
        let doc_id = DocId::from_key(&doc_key)?;
        tx.stage_text_insert(doc_id, txt);
    }

    // Rollback 5-document transaction
    tx.rollback().await?;

    // Invariant: docs(LSM) == docs(HNSW) == docs(Text) == 0
    assert_index_doc_count_invariant(&col, 0).await?;

    for i in 1..=5 {
        let doc_key = format!("multi_doc_{}", i);
        assert!(col.get(&doc_key).await?.is_none());
    }

    db.close().await?;
    Ok(())
}

// ANCHOR[contextra-engine:TST39-006]
/// Testblock zur Überprüfung von `test_tx_idempotency_of_compensation_and_recovery`.
#[tokio::test]
async fn test_tx_idempotency_of_compensation_and_recovery() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let storage = db.inner_storage();
    let next_tx = Arc::new(AtomicU64::new(100));

    // Store a pending intent
    let intent_key = b"__tx_intent:999".to_vec();
    let doc_id = DocId::new(999);
    let pending = CommitIntent::Pending {
        doc_ids: Arc::new(vec![doc_id]),
        has_text: true,
        has_graph: false,
        stages_completed: 0,
    };
    let intent_bytes = serde_json::to_vec(&pending)?;
    storage
        .put(TxId::new(1), &intent_key, &intent_bytes)
        .await?;
    storage.commit(TxId::new(1)).await?;

    // Run recovery 1st time
    let rec1 = recover_pending_intents(storage.as_ref(), &next_tx).await?;
    assert_eq!(rec1, 1);

    // Run recovery 2nd time
    let rec2 = recover_pending_intents(storage.as_ref(), &next_tx).await?;
    assert_eq!(
        rec2, 0,
        "Second recovery pass should find 0 pending intents"
    );

    // Run recovery 3rd time
    let rec3 = recover_pending_intents(storage.as_ref(), &next_tx).await?;
    assert_eq!(rec3, 0, "Third recovery pass should find 0 pending intents");

    let val = storage.get(&intent_key).await?;
    assert!(val.is_some());
    let final_intent: CommitIntent = serde_json::from_slice(&val.unwrap())?;
    assert!(matches!(final_intent, CommitIntent::Aborted));

    db.close().await?;
    Ok(())
}
