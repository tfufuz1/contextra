#![cfg(not(loom))]
// FILE-CONTEXT
// ZWECK: Targeted crash simulation tests for multi-index 2-Phase Commit consistency.
// INVARIANTEN: No orphaned index entries (Text/Graph/Vector) can exist without a corresponding storage record.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::collection::{StoredDocument, StoredDocumentMeta};
use contextra_engine::transaction::{CommitIntent, DbTransaction};
use contextra_engine::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats, VectorIndex};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, DocId, Entity, EntityId, Result, TxId};
use contextra_vector::HnswIndex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

struct FaultyStorage<S: StorageEngine> {
    inner: Arc<S>,
    fail_intent_put: AtomicBool,
    fail_storage_commit: AtomicBool,
    fail_committed_put: AtomicBool,
    storage_commit_count: AtomicU64,
}

impl<S: StorageEngine> FaultyStorage<S> {
    fn new(inner: Arc<S>) -> Self {
        Self {
            inner,
            fail_intent_put: AtomicBool::new(false),
            fail_storage_commit: AtomicBool::new(false),
            fail_committed_put: AtomicBool::new(false),
            storage_commit_count: AtomicU64::new(0),
        }
    }
}

impl<S: StorageEngine> StorageEngine for FaultyStorage<S> {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.inner.get(key)
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.inner.get_at_seq(key, seq)
    }

    fn get_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.inner.get_tracked(tx_id, key)
    }

    fn scan_prefix_tracked<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan_prefix_tracked(tx_id, prefix)
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan_prefix_at(prefix, seq_no)
    }

    fn supports_ssi_tracking(&self) -> bool {
        self.inner.supports_ssi_tracking()
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        if key.starts_with(b"__tx_intent:") {
            if serde_json::from_slice::<CommitIntent>(value)
                .map(|i| matches!(i, CommitIntent::Pending { .. }))
                .unwrap_or(false)
                && self.fail_intent_put.load(Ordering::Relaxed)
            {
                return Box::pin(async move {
                    Err(ContextraError::Storage(
                        "Injected intent put failure".into(),
                    ))
                });
            }
            if serde_json::from_slice::<CommitIntent>(value)
                .map(|i| matches!(i, CommitIntent::Committed))
                .unwrap_or(false)
                && self.fail_committed_put.load(Ordering::Relaxed)
            {
                return Box::pin(async move {
                    Err(ContextraError::Storage(
                        "Injected committed marker put failure".into(),
                    ))
                });
            }
        }
        self.inner.put(tx_id, key, value)
    }

    fn put_if_absent<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        self.inner.put_if_absent(tx_id, key, value)
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        self.inner.delete(tx_id, key)
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        if self.fail_storage_commit.load(Ordering::Relaxed) {
            return Box::pin(async move {
                Err(ContextraError::Storage(
                    "Injected storage commit failure".into(),
                ))
            });
        }
        self.storage_commit_count.fetch_add(1, Ordering::SeqCst);
        self.inner.commit(tx_id)
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.rollback(tx_id)
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.rollback_to_tx(tx_id)
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        self.inner.flush()
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        self.inner.stats()
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        self.inner.last_seq_no()
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        self.inner.last_tx_id()
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.inner.pin_checkpoint(seq_no)
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.inner.unpin_checkpoint(seq_no)
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan_prefix(prefix)
    }

    fn scan_prefix_bounded<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        self.inner.scan_prefix_bounded(prefix, limit, cursor)
    }

    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan(start, end, limit)
    }
}

async fn create_faulty_collection() -> (
    Collection<FaultyStorage<LsmStorage>, HnswIndex>,
    Arc<FaultyStorage<LsmStorage>>,
) {
    let dir = tempdir().unwrap();
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
    let storage = Arc::new(FaultyStorage::new(lsm));
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage.clone(),
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );
    (col, storage)
}

/// Test that if storage commit fails (Phase b), index staging is aborted/rolled back,
/// leaving zero orphaned index entries.
#[tokio::test]
async fn test_storage_commit_failure_leaves_no_orphaned_index_entries() -> Result<()> {
    let (col, storage) = create_faulty_collection().await;
    let tx_id = col.allocate_tx()?;
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(501);
    tx.stage_text_insert(doc_id, "Critical crash window test document text".to_string());
    tx.stage_graph_entity(Entity::new(
        EntityId::from_key("501")?,
        "Node501",
        "Document",
    ));

    // Inject storage commit failure in Phase (b)
    storage.fail_storage_commit.store(true, Ordering::SeqCst);

    let commit_res = tx.commit().await;
    assert!(
        commit_res.is_err(),
        "Commit must fail when storage commit fails"
    );

    // Run repair() to simulate startup recovery
    col.repair().await?;

    // Verify zero orphaned entries in text index
    let text_hits = col.query().text("Critical").execute().await?;
    assert!(
        text_hits.is_empty(),
        "Text index must not contain entries when storage commit failed"
    );

    // Verify zero orphaned entries in graph index
    let neighbors = col
        .graph_index()
        .neighbors(EntityId::from_key("501")?)
        .await?;
    assert!(
        neighbors.is_empty(),
        "Graph index must not contain entity when storage commit failed"
    );

    Ok(())
}

/// Test that if crash occurs after storage commit but before index commit,
/// recovery (`repair()`) forward-populates the index entries from storage, ensuring consistency.
#[tokio::test]
async fn test_crash_after_storage_commit_recovers_indices_from_storage() -> Result<()> {
    let (col, storage) = create_faulty_collection().await;

    let doc_id = DocId::new(602);
    let tx_id = col.allocate_tx()?;
    let tx = DbTransaction::new(col.clone(), tx_id);

    // Stage document payload in LSM storage and text operations in transaction
    let user_key = col.namespaced_key(b"602", 0);
    let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
    let stored_doc = StoredDocument {
        id: "602".to_string(),
        embedding: vec![1.0, 0.0, 0.0, 0.0],
        metadata: Some(serde_json::json!({"text": "Storage committed recovery doc"})),
    };
    let stored_meta = StoredDocumentMeta::from(&stored_doc);

    col.storage().put(tx_id, &user_key, &serde_json::to_vec(&stored_doc)?).await?;
    col.storage().put(tx_id, &doc_key, &serde_json::to_vec(&stored_meta)?).await?;
    tx.record_keys(user_key.clone(), doc_key.clone(), doc_id);
    tx.stage_text_insert(doc_id, "Storage committed recovery doc".to_string());

    // Inject committed marker put failure to simulate a crash right after storage commit (Phase b) before completion
    storage.fail_committed_put.store(true, Ordering::SeqCst);

    // Execute transaction commit (Phase b succeeds, Phase d marker fails)
    let _ = tx.commit().await;

    // Verify that storage contains document 602
    assert!(storage.get(&user_key).await?.is_some());
    assert!(storage.get(&doc_key).await?.is_some());

    // Manually trigger repair() to simulate recovery pass after crash/restart
    col.repair().await?;

    // Verify vector index was forward-recovered
    let vec_hits_after = col.vector_index().search(&[1.0, 0.0, 0.0, 0.0], 10).await?;
    assert!(
        vec_hits_after.iter().any(|h| h.doc_id == doc_id),
        "Vector index entry for doc 602 must be recovered from storage"
    );

    // Verify text index was forward-recovered
    let text_hits = col.query().text("Storage").execute().await?;
    assert!(
        !text_hits.is_empty(),
        "Text index entry for doc 602 must be recovered from storage"
    );

    Ok(())
}
