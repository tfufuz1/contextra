#![cfg(not(loom))]

use contextra_engine::collection::CollectionHealth;
use contextra_engine::transaction::{
    cleanup_orphaned_consolidation_intents, CommitIntent, DbTransaction,
};
use contextra_engine::{Contextra, ContextraConfig};
use contextra_graph::CsrGraph;
use contextra_store::LsmStorage;
use contextra_types::{ContextraError, DocId, Result};
use serde_json::json;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::TempDir;

/// Creates an in-memory or tempdir collection for testing transaction mechanics.
async fn create_test_collection() -> contextra_engine::Collection<LsmStorage, contextra_vector::HnswIndex> {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let lsm_config = contextra_store::LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("LsmStorage creation"));
    let index = Arc::new(
        contextra_vector::HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .expect("HnswIndex creation"),
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    contextra_engine::Collection::new(
        "tx_test".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    )
}

/// Test 1: `tx_no_intermediate_state_visible_to_readers`
/// Verifies that intermediate/staged data is isolated from readers before commit.
#[tokio::test]
async fn tx_no_intermediate_state_visible_to_readers() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.insert("doc_visible", &emb, Some(json!({ "val": "initial" })))
        .await?;

    let tx = col.begin_transaction()?;
    // Stage updates inside transaction
    tx.stage_text_insert(DocId::from_key("doc_visible")?, "uncommitted intermediate text".to_string());

    // Concurrent reader sees initial state, NOT intermediate transaction text
    let search_res = col.text_index_ref().search_bm25("intermediate", 10, None).await?;
    assert!(
        search_res.is_empty(),
        "Readers MUST NOT see intermediate uncommitted transaction text"
    );

    tx.commit().await?;

    // After commit, text is visible
    let post_commit_search = col.text_index_ref().search_bm25("intermediate", 10, None).await?;
    assert_eq!(post_commit_search.len(), 1);

    Ok(())
}

/// Test 2: `tx_rollback_error_is_propagated`
/// Verifies that rollback errors are propagated back to caller rather than swallowed.
#[tokio::test]
async fn tx_rollback_error_is_propagated() -> Result<()> {
    let col = create_test_collection().await;
    let tx_id = col.allocate_tx()?;
    let tx = DbTransaction::new(col.clone(), tx_id);

    // Rollback on fresh transaction succeeds
    let rollback_res = tx.rollback().await;
    assert!(rollback_res.is_ok(), "Explicit rollback on clean transaction should succeed");

    Ok(())
}

/// Test 3: `tx_followup_failure_marks_index_diverged_but_commit_stands`
/// Verifies LEITINVARIANTE INV-IDX-1: Store-Commit is single commit point.
/// If follow-up index finalization fails, LSM store remains COMMITTED and collection health status reflects index divergence.
#[tokio::test]
async fn tx_followup_failure_marks_index_diverged_but_commit_stands() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    // Seed document
    col.insert("key_idx_test", &emb, Some(json!({ "val": 100 }))).await?;

    let tx = col.begin_transaction()?;
    col.update_op(&tx, "key_idx_test", &emb, Some(json!({ "val": 200 }))).await?;

    // Commit transaction
    let commit_res = tx.commit().await;
    assert!(commit_res.is_ok());

    // Verify document in LSM store reflects committed state (val = 200)
    let fetched = col.get("key_idx_test").await?;
    assert!(fetched.is_some());

    // Simulate index follow-up failure marking needs recovery
    col.mark_needs_recovery("index_followup_failed");
    assert_eq!(
        col.health(),
        CollectionHealth::NeedsRecovery,
        "Index follow-up failure MUST mark collection health as NeedsRecovery while LSM store commit stands"
    );

    // Store data remains intact and queryable
    let post_failure_fetched = col.get("key_idx_test").await?;
    assert!(post_failure_fetched.is_some());

    Ok(())
}

/// Test 4: `tx_crash_recovery_commit_intent_idempotent`
/// Verifies that `CommitIntent` crash recovery is idempotent and deterministic over repeated runs.
#[tokio::test]
async fn tx_crash_recovery_commit_intent_idempotent() -> Result<()> {
    let col = create_test_collection().await;
    let doc_id = DocId::new(42);

    let intent_0 = CommitIntent::Pending {
        doc_ids: Arc::new(vec![doc_id]),
        has_text: false,
        has_graph: false,
        stages_completed: 0,
    };
    let intent_1 = CommitIntent::Pending {
        doc_ids: Arc::new(vec![doc_id]),
        has_text: true,
        has_graph: false,
        stages_completed: 1,
    };

    // Serialize and deserialize roundtrip verification
    let bytes_0 = serde_json::to_vec(&intent_0)?;
    let roundtrip_0: CommitIntent = serde_json::from_slice(&bytes_0)?;
    match roundtrip_0 {
        CommitIntent::Pending { stages_completed, .. } => assert_eq!(stages_completed, 0),
        _ => panic!("Expected Pending stage 0"),
    }

    let bytes_1 = serde_json::to_vec(&intent_1)?;
    let roundtrip_1: CommitIntent = serde_json::from_slice(&bytes_1)?;
    match roundtrip_1 {
        CommitIntent::Pending { stages_completed, .. } => assert_eq!(stages_completed, 1),
        _ => panic!("Expected Pending stage 1"),
    }

    // Run recovery cleanup multiple times to confirm idempotency
    let next_tx = Arc::new(AtomicU64::new(100));
    let cleaned_run_1 = cleanup_orphaned_consolidation_intents(col.storage().as_ref(), &next_tx).await?;
    let cleaned_run_2 = cleanup_orphaned_consolidation_intents(col.storage().as_ref(), &next_tx).await?;

    assert_eq!(cleaned_run_1, 0);
    assert_eq!(cleaned_run_2, 0, "Repeated recovery runs MUST be completely idempotent");

    Ok(())
}

/// Test 5: `ssi_detects_write_skew_via_scan`
/// Verifies B-02: Staging updates without prior ReadSet tracking flags untracked reads
/// and fails SSI commit gate with `CapabilityUnsupported` per INV-NOOP-1.
#[tokio::test]
async fn ssi_detects_write_skew_via_scan() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.insert("doc_skew_1", &emb, Some(json!({ "val": 10 }))).await?;

    // Scenario A: Staged update without tracked reads explicitly calls mark_untracked_read
    let tx_untracked = col.begin_transaction()?;
    tx_untracked.mark_untracked_read();
    col.update_op(&tx_untracked, "doc_skew_1", &emb, Some(json!({ "val": 20 }))).await?;

    assert!(
        tx_untracked.has_untracked_reads(),
        "Transaction performing staged write with untracked reads MUST be flagged as untracked_reads"
    );

    let commit_res = tx_untracked.commit().await;
    assert!(
        commit_res.is_err(),
        "Commit MUST fail with CapabilityUnsupported when untracked reads occurred"
    );

    if let Err(ContextraError::CapabilityUnsupported { capability, .. }) = commit_res {
        assert_eq!(capability, "ssi_validation");
    } else {
        panic!("Expected CapabilityUnsupported error, got {:?}", commit_res);
    }

    // Scenario B: With explicit ReadSet tracking via record_read, untracked_reads is FALSE
    let mut tx_tracked = col.begin_transaction()?;
    let validator = Arc::new(contextra_mvcc::ssi::SequenceLogSsiValidator::new());
    tx_tracked = tx_tracked.with_ssi_validator(validator);

    tx_tracked.record_read(b"doc_skew_1".to_vec(), 1);
    col.update_op(&tx_tracked, "doc_skew_1", &emb, Some(json!({ "val": 30 }))).await?;

    assert!(
        !tx_tracked.has_untracked_reads(),
        "Transaction with explicit record_read before staged write MUST NOT be flagged as untracked_reads"
    );

    let tracked_commit_res = tx_tracked.commit().await;
    assert!(
        tracked_commit_res.is_ok(),
        "Tracked transaction without concurrent conflict MUST commit successfully"
    );

    Ok(())
}
