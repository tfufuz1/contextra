#![cfg(not(loom))]
// FILE-CONTEXT
// ZWECK: Equivalence test for transaction crash recovery when interrupted at stage 1.
// INVARIANTEN: If transaction committed LSM (stages_completed = 1) but crashed before vector/text index commit,
// repair_on_open / repair() must reconcile indices with storage and mark intent Committed.

use contextra_engine::collection::{StoredDocument, StoredDocumentMeta};
use contextra_engine::transaction::CommitIntent;
use contextra_engine::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{DocId, Result};
use contextra_vector::HnswIndex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_tx_recovery_equivalence_stage1_crash() -> Result<()> {
    let dir = tempdir().map_err(|e| contextra_types::ContextraError::Storage(e.to_string()))?;
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(lsm_config).await?);
    let index = Arc::new(HnswIndex::try_new(contextra_vector::HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        lsm.clone(),
        index.clone(),
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    let doc_str = "doc-tx-rec-1";
    let doc_id = DocId::from_key(doc_str)?;
    let tx_id = col.allocate_tx()?;

    let user_key = col.namespaced_key(doc_str.as_bytes(), 0);
    let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
    let intent_key = col.namespaced_key(&tx_id.inner().to_le_bytes(), 3);

    let stored_doc = StoredDocument {
        id: doc_str.to_string(),
        embedding: vec![1.0, 0.0, 0.0, 0.0],
        metadata: Some(serde_json::json!({"text": "Recovery equivalence text"})),
    };
    let stored_meta = StoredDocumentMeta::from(&stored_doc);

    // 1. Stage document payload & metadata in LSM storage under tx_id and commit LSM
    col.storage()
        .put(tx_id, &user_key, &stored_doc.encode_v2()?)
        .await?;
    col.storage()
        .put(tx_id, &doc_key, &serde_json::to_vec(&stored_meta)?)
        .await?;
    col.storage().commit(tx_id).await?;

    // 2. Write Pending intent marker with stages_completed = 1 (simulating crash after stage 1 LSM commit)
    let intent = CommitIntent::Pending {
        doc_ids: Arc::new(vec![doc_id]),
        has_text: true,
        has_graph: false,
        stages_completed: 1,
    };
    let intent_bytes = serde_json::to_vec(&intent)?;
    let intent_tx = col.allocate_tx()?;
    col.storage().put(intent_tx, &intent_key, &intent_bytes).await?;
    col.storage().commit(intent_tx).await?;

    // Confirm that before repair, vector search and text search do not return the document
    let vec_before = col.vector_index().search(&[1.0, 0.0, 0.0, 0.0], 10).await?;
    assert!(
        !vec_before.iter().any(|h| h.doc_id == doc_id),
        "Vector index must be unpopulated before recovery"
    );

    let text_before = col.query().text("Equivalence").execute().await?;
    assert!(
        text_before.is_empty(),
        "Text index must be unpopulated before recovery"
    );

    // 3. Trigger repair (simulating startup repair_on_open)
    col.repair().await?;

    // 4. Verify post-recovery consistency across document storage, vector index, and text index
    assert!(
        col.storage().get(&user_key).await?.is_some(),
        "Storage document must remain intact after recovery"
    );

    let vec_after = col.vector_index().search(&[1.0, 0.0, 0.0, 0.0], 10).await?;
    assert!(
        vec_after.iter().any(|h| h.doc_id == doc_id),
        "Vector index entry for doc_id must be recovered"
    );

    let text_after = col.query().text("Equivalence").execute().await?;
    assert!(
        !text_after.is_empty(),
        "Text index entry for doc_id must be recovered"
    );

    Ok(())
}
