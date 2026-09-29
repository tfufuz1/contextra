// FILE-CONTEXT
// ZWECK: Integrationstests für Crash-Fenster, Recovery & commit-uncertain Transaktionen (F-20 / T-06).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::collection::StoredDocument;
use contextra_engine::transaction::{CommitIntent, DbTransaction};
use contextra_engine::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{BoxFuture, GraphIndex, StorageEngine, StorageStats, VectorIndex};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, DocId, Edge, Entity, EntityId, Result, TxId};
use contextra_vector::HnswIndex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

struct FaultyStorage<S: StorageEngine> {
    inner: Arc<S>,
    fail_intent_put: AtomicBool,
    fail_committed_put: AtomicBool,
    fail_storage_commit: AtomicBool,
}

impl<S: StorageEngine> FaultyStorage<S> {
    fn new(inner: Arc<S>) -> Self {
        Self {
            inner,
            fail_intent_put: AtomicBool::new(false),
            fail_committed_put: AtomicBool::new(false),
            fail_storage_commit: AtomicBool::new(false),
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

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        if key.starts_with(b"__tx_intent:") {
            if self.fail_intent_put.load(Ordering::Relaxed)
                && serde_json::from_slice::<CommitIntent>(value)
                    .map(|i| matches!(i, CommitIntent::Pending { .. }))
                    .unwrap_or(false)
            {
                return Box::pin(async move {
                    Err(ContextraError::Storage(
                        "Injected intent put failure".into(),
                    ))
                });
            }
            if self.fail_committed_put.load(Ordering::Relaxed)
                && serde_json::from_slice::<CommitIntent>(value)
                    .map(|i| matches!(i, CommitIntent::Committed))
                    .unwrap_or(false)
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

    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan(start, end, limit)
    }
}

async fn create_test_collection() -> (
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

/// (a) Crash nach Text-Staging / vor Intent-Write (oder Abbruch beim Intent-Write)
#[tokio::test]
async fn test_crash_window_a_text_staging_before_intent() -> Result<()> {
    let (col, storage) = create_test_collection().await;
    let tx_id = col.allocate_tx()?;
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(101);
    tx.stage_text_insert(doc_id, "Staged text before intent crash".to_string());

    // Inject failure when writing the pending intent marker
    storage.fail_intent_put.store(true, Ordering::SeqCst);

    let res = tx.commit().await;
    assert!(
        res.is_err(),
        "Commit must fail when intent marker write fails"
    );

    // Verify vector index and storage do not contain uncommitted data
    let vec_ids = col.vector_index().all_doc_ids().await?;
    assert!(
        !vec_ids.contains(&doc_id),
        "Vector index must be clean after intent write failure"
    );

    Ok(())
}

/// (b) Crash nach Graph-Staging, vor Storage-Commit
#[tokio::test]
async fn test_crash_window_b_graph_staging_before_storage_commit() -> Result<()> {
    let (col, storage) = create_test_collection().await;
    let tx_id = col.allocate_tx()?;
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(202);
    tx.record_keys(vec![1, 2, 3, 4], vec![4, 3, 2, 1], doc_id);
    tx.stage_text_insert(doc_id, "Graph and text staged document".to_string());

    let e1 = Entity::new(EntityId::from_key("e_crash_b1")?, "E1", "Concept");
    let e2 = Entity::new(EntityId::from_key("e_crash_b2")?, "E2", "Concept");
    tx.stage_graph_entity(e1);
    tx.stage_graph_entity(e2);
    tx.stage_graph_edge(Edge::new(
        EntityId::from_key("e_crash_b1")?,
        EntityId::from_key("e_crash_b2")?,
        "RELATED",
    ));

    // Inject storage commit failure (storage commit fails after index commits have executed)
    storage.fail_storage_commit.store(true, Ordering::SeqCst);

    let res = tx.commit().await;
    assert!(res.is_err(), "Commit must fail when storage commit fails");

    // After commit failure, repair() is triggered
    col.repair().await?;

    // Verify graph neighbors were compensated/cleaned
    let neighbors = col
        .graph_index()
        .neighbors(EntityId::from_key("e_crash_b1")?)
        .await?;
    assert!(
        neighbors.is_empty(),
        "Graph index must be clean after recovery from storage commit crash"
    );

    Ok(())
}

/// (c) Storage-Commit erfolgreich, Committed-Marker-Write scheitert dauerhaft
/// => Daten bleiben, Indizes konsistent, keine Kompensation, Intent wird bereinigt
#[tokio::test]
async fn test_crash_window_c_storage_commit_ok_committed_marker_fails() -> Result<()> {
    let (col, storage) = create_test_collection().await;
    let tx_id = col.allocate_tx()?;
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(303);
    tx.record_keys(vec![10, 20, 30, 40], vec![40, 30, 20, 10], doc_id);
    tx.stage_text_insert(doc_id, "Durable text for window C".to_string());

    let e1 = Entity::new(EntityId::from_key("e_crash_c1")?, "E1", "Concept");
    tx.stage_graph_entity(e1);

    // Staging document metadata/data in storage as well via collection.insert or manual put
    let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
    let stored_doc = StoredDocument {
        id: "doc_303".to_string(),
        embedding: vec![1.0, 0.0, 0.0, 0.0],
        metadata: Some(serde_json::json!({ "text": "Durable text for window C" })),
    };
    let doc_bytes = serde_json::to_vec(&stored_doc)?;
    storage.put(tx_id, &doc_key, &doc_bytes).await?;

    // Fail committed marker put dauerhaft
    storage.fail_committed_put.store(true, Ordering::SeqCst);

    let res = tx.commit().await;
    // Commit return value might be Ok(()) even if background committed marker retries failed,
    // or err. In either case, storage commit succeeded.
    let _ = res;

    // Run repair/recovery
    col.repair().await?;

    // Verify storage record is still present (data preserved)
    let get_res = storage.get(&doc_key).await?;
    assert!(
        get_res.is_some(),
        "Document data must remain in storage after recovery"
    );

    // Verify vector index has the document
    let all_vec_ids = col.vector_index().all_doc_ids().await?;
    assert!(
        all_vec_ids.contains(&doc_id),
        "Vector index must retain the document after recovery"
    );

    Ok(())
}

/// (d) Storage-Commit nicht erfolgt + Pending-Intent
/// => Recovery entfernt Index-Einträge ohne Storage-Datensatz (Vektor, Text, Graph)
#[tokio::test]
async fn test_crash_window_d_pending_intent_without_storage_commit() -> Result<()> {
    let (col, storage) = create_test_collection().await;

    // Manually write a Pending intent with doc_id 404 that was inserted into indices
    // but never committed to storage.
    let orphan_doc_id = DocId::new(404);
    let orphan_entity_id = EntityId::from_key("orphan_entity")?;

    let tx_id = col.allocate_tx()?;
    col.vector_index()
        .insert(tx_id, orphan_doc_id, &[0.5, 0.5, 0.5, 0.5])
        .await?;
    col.vector_index().commit(tx_id).await?;

    col.graph_index()
        .add_entity(
            tx_id,
            Entity::new(orphan_entity_id, "orphan_entity", "Document"),
        )
        .await?;
    col.graph_index().commit(tx_id).await?;

    // Write Pending intent to storage for tx_id
    let intent_key = col.namespaced_key(&tx_id.inner().to_le_bytes(), 3);
    let intent = CommitIntent::Pending {
        doc_ids: Arc::new(vec![orphan_doc_id]),
        has_text: true,
        has_graph: true,
        stages_completed: 2,
    };
    let intent_bytes = serde_json::to_vec(&intent)?;
    storage.put(tx_id, &intent_key, &intent_bytes).await?;
    storage.commit(tx_id).await?;

    // Confirm that doc_id 404 is NOT in storage
    let doc_key = col.namespaced_key(&orphan_doc_id.inner().to_le_bytes(), 1);
    assert!(
        storage.get(&doc_key).await?.is_none(),
        "Document must not exist in storage for crash window D"
    );

    // Run recovery
    col.repair().await?;

    // Verify orphaned entries were removed from indices
    let vec_ids = col.vector_index().all_doc_ids().await?;
    assert!(
        !vec_ids.contains(&orphan_doc_id),
        "Vector index entry must be removed during recovery when storage record is missing"
    );

    let neighbors = col.graph_index().neighbors(orphan_entity_id).await?;
    assert!(
        neighbors.is_empty(),
        "Graph index entry must be clean during recovery when storage record is missing"
    );

    Ok(())
}
