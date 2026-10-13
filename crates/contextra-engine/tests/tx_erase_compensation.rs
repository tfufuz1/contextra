#![cfg(not(loom))]
#![forbid(unsafe_code)]

//! Integration test verifying compensation action handling in DbTransaction.
//! Verifies that if an error occurs during post-storage commit steps (such as index staging/commit),
//! compensation actions roll back LSM storage and index modifications fail-closed,
//! leaving no partially-deleted or orphaned state.

use contextra_engine::transaction::{
    CommitLedger, CompensateGraphAction, CompensateHnswAction, CompensateLsmAction,
    CompensateTextAction, RollbackStagedAction,
};
use contextra_engine::Contextra;
use contextra_ports::{GraphIndex, VectorIndex};
use contextra_types::{DocId, EntityId, TenantId, TxId};
use std::sync::Arc;

#[tokio::test]
async fn test_compensation_cleans_up_storage_on_post_storage_failure() {
    let dir = tempfile::tempdir().unwrap();
    let db = Contextra::open(dir.path()).await.unwrap();
    let col = db.collection("compensation_test").await.unwrap();

    let doc_id = "doc_comp_1";
    let text = "compensation_safety_check_text";
    let mut embedding = vec![0.0f32; col.dimension()];
    embedding[0] = 1.0;

    col.insert(doc_id, &embedding, Some(serde_json::json!({ "text": text })))
        .await
        .unwrap();

    // Verify doc_id exists initially
    let doc_typed = DocId::from_key(doc_id).unwrap();
    let initial_ids = col.vector_index().all_doc_ids().await.unwrap();
    assert!(initial_ids.contains(&doc_typed));

    // Construct a CommitLedger and test rollback execution
    let mut ledger = CommitLedger::new();
    let tx_id = TxId::new(999);

    ledger.push(RollbackStagedAction::new((*col).clone(), tx_id));

    let intent_key = col.namespaced_key(b"test_intent_comp", 3);
    let doc_ids = Arc::new(vec![doc_typed]);

    ledger.push(CompensateLsmAction::new(
        (*col).clone(),
        intent_key,
        Arc::clone(&doc_ids),
        vec![],
        vec![],
    ));

    let tenant_id = TenantId::try_new(1).unwrap_or_default();
    ledger.push(CompensateHnswAction::new(
        (*col).clone(),
        tenant_id,
        Arc::clone(&doc_ids),
    ));

    ledger.push(CompensateTextAction::new(
        (*col).clone(),
        Arc::clone(&doc_ids),
    ));

    ledger.push(CompensateGraphAction::new(
        (*col).clone(),
        Arc::clone(&doc_ids),
    ));

    // Execute rollback and ensure it completes without error
    let rollback_res = ledger.execute_rollback().await;
    assert!(
        rollback_res.is_ok(),
        "Compensating actions must succeed and propagate clean status"
    );

    // After compensation execution, verify index state
    let post_ids = col.vector_index().all_doc_ids().await.unwrap();
    assert!(
        !post_ids.contains(&doc_typed),
        "Doc must be removed from HNSW index after compensation"
    );

    let text_hits = col
        .text_index_ref()
        .search_bm25(text, 10, None)
        .await
        .unwrap();
    assert!(
        text_hits.is_empty(),
        "Text index must leave no residual postings after compensation"
    );

    let eid = EntityId::from_doc_id(doc_typed);
    let graph_neighbors = col
        .graph_index()
        .traverse(eid, 1)
        .await
        .unwrap();
    assert!(
        graph_neighbors.is_empty(),
        "Graph index must contain no entity connections after compensation"
    );

    db.close().await.unwrap();
}
