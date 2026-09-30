use contextra_engine::transaction::{CommitIntent, DbTransaction};
use contextra_engine::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
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
    committed_put_count: AtomicU64,
    intent_put_count: AtomicU64,
}

impl<S: StorageEngine> FaultyStorage<S> {
    fn new(inner: Arc<S>) -> Self {
        Self {
            inner,
            fail_intent_put: AtomicBool::new(false),
            fail_committed_put: AtomicBool::new(false),
            committed_put_count: AtomicU64::new(0),
            intent_put_count: AtomicU64::new(0),
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
            if serde_json::from_slice::<CommitIntent>(value)
                .map(|i| matches!(i, CommitIntent::Pending { .. }))
                .unwrap_or(false)
            {
                self.intent_put_count.fetch_add(1, Ordering::SeqCst);
                if self.fail_intent_put.load(Ordering::Relaxed) {
                    return Box::pin(async move {
                        Err(ContextraError::Storage(
                            "Injected intent put failure".into(),
                        ))
                    });
                }
            }
            if self.fail_committed_put.load(Ordering::Relaxed)
                && serde_json::from_slice::<CommitIntent>(value)
                    .map(|i| matches!(i, CommitIntent::Committed))
                    .unwrap_or(false)
            {
                self.committed_put_count.fetch_add(1, Ordering::SeqCst);
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

#[tokio::test]
async fn test_f19_intent_put_failure_triggers_rollback() -> Result<()> {
    let (col, storage) = create_faulty_collection().await;
    let tx_id = col.allocate_tx().unwrap();
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(999);
    tx.stage_text_insert(doc_id, "test text for F19 failure".to_string());

    let e1 = Entity::new(EntityId::from_key("node_f19_1")?, "Node1", "Concept");
    let e2 = Entity::new(EntityId::from_key("node_f19_2")?, "Node2", "Concept");
    tx.stage_graph_entity(e1);
    tx.stage_graph_entity(e2);
    tx.stage_graph_edge(Edge::new(
        EntityId::from_key("node_f19_1")?,
        EntityId::from_key("node_f19_2")?,
        "CONNECTS",
    ));

    // Inject failure on intent put
    storage.fail_intent_put.store(true, Ordering::SeqCst);

    let commit_res = tx.commit().await;
    assert!(
        commit_res.is_err(),
        "Commit should fail when intent put fails"
    );

    // Verify graph index was rolled back
    let neighbors = col
        .graph_index()
        .neighbors(EntityId::from_key("node_f19_1")?)
        .await?;
    assert!(
        neighbors.is_empty(),
        "Graph index must be rolled back after intent put failure (F-19)"
    );

    Ok(())
}

#[tokio::test]
async fn test_f20_pending_intent_written_first() -> Result<()> {
    let (col, storage) = create_faulty_collection().await;
    let tx_id = col.allocate_tx().unwrap();
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(777);
    tx.record_keys(vec![10, 20], vec![20, 10], doc_id);
    tx.stage_text_insert(doc_id, "F20 intent test".to_string());

    let _ = tx.commit().await?;

    let intent_count = storage.intent_put_count.load(Ordering::SeqCst);
    assert!(
        intent_count >= 1,
        "Pending intent marker must be written to storage during commit (F-20)"
    );

    Ok(())
}

#[tokio::test]
async fn test_t06_committed_marker_write_failure_retries() -> Result<()> {
    let (col, storage) = create_faulty_collection().await;
    let tx_id = col.allocate_tx().unwrap();
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(888);
    tx.record_keys(vec![1, 2, 3], vec![3, 2, 1], doc_id);

    // Fail committed marker put
    storage.fail_committed_put.store(true, Ordering::SeqCst);

    let _res = tx.commit().await;

    let put_attempts = storage.committed_put_count.load(Ordering::SeqCst);
    assert!(
        put_attempts >= 3,
        "Committed marker failure must trigger retries (T-06), attempts = {}",
        put_attempts
    );

    Ok(())
}
