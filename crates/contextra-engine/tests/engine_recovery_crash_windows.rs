// FILE-CONTEXT
// ZWECK: Integrationstests für Crash-Fenster-Recovery und commit-uncertain Transaktionen (F-20 / T-06).
// INVARIANTEN: Verifiziert Crash-Szenarien und stellt sicher, dass repair() unvollständige Transaktionen korrekt behandelt.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::transaction::{CommitIntent, DbTransaction};
use contextra_engine::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{BoxFuture, GraphIndex, StorageEngine, StorageStats, VectorIndex};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, DocId, Entity, EntityId, Result, TxId};
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
    tempfile::TempDir,
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
    (col, storage, dir)
}

/// (a) Crash nach Text-Staging, vor Intent-Write:
/// Text-Einträge werden gestagt, aber vor Commit bzw. Intent-Write stürzt der Prozess ab.
/// Nach Wiederanlauf / repair() dürfen keine phantomen Text-Index-Einträge existieren.
#[tokio::test]
async fn test_crash_after_text_staging_before_intent_write() -> Result<()> {
    let (col, storage, _dir) = create_faulty_collection().await;
    let tx_id = col.allocate_tx()?;
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(101);
    tx.stage_text_insert(doc_id, "Staged text before crash".to_string());

    // Injected failure on intent write prevents tx.commit() from completing Phase (a)
    storage.fail_intent_put.store(true, Ordering::SeqCst);
    let commit_res = tx.commit().await;
    assert!(commit_res.is_err());

    // Run repair()
    col.repair().await?;

    // Text index query must return no hits
    let hits = col.query().text("Staged").execute().await?;
    assert!(
        hits.is_empty(),
        "Text query must not return entries from uncommitted tx after crash"
    );

    Ok(())
}

/// (b) Crash nach Graph-Staging, vor Storage-Commit:
/// Simuliert einen Crash, nachdem Graph/Text/Vector-Staging und Phase (a) Intent-Write stattfand,
/// aber vor dem Storage-Commit (Phase c).
#[tokio::test]
async fn test_crash_after_graph_staging_before_storage_commit() -> Result<()> {
    let (col, storage, _dir) = create_faulty_collection().await;

    // Manually write a Pending intent for doc_id 202
    let doc_id = DocId::new(202);
    let intent_tx = col.allocate_tx()?;
    let intent_key = col.namespaced_key(&intent_tx.inner().to_le_bytes(), 3);
    let intent = CommitIntent::Pending {
        doc_ids: Arc::new(vec![doc_id]),
        has_text: true,
        has_graph: true,
        stages_completed: 0,
    };
    let intent_bytes = serde_json::to_vec(&intent)?;
    storage.put(intent_tx, &intent_key, &intent_bytes).await?;
    storage.commit(intent_tx).await?;

    // Stage/insert into vector, text, and graph indices to simulate Phase (b) having run
    let stage_tx = col.allocate_tx()?;
    col.vector_index()
        .insert(stage_tx, doc_id, &[1.0, 0.0, 0.0, 0.0])
        .await?;
    col.vector_index().commit(stage_tx).await?;

    let e1 = Entity::new(EntityId::from_key("202")?, "Node202", "Doc");
    col.graph_index().add_entity(stage_tx, e1).await?;
    col.graph_index().commit(stage_tx).await?;

    // Storage commit NEVER happened for doc_id 202!
    // Now run repair() to recover from this crash window
    col.repair().await?;

    // Verify recovery compensated (deleted) the index entries
    let vec_hits = col.vector_index().search(&[1.0, 0.0, 0.0, 0.0], 10).await?;
    assert!(
        vec_hits.iter().all(|h| h.doc_id != doc_id),
        "Vector index entry for doc_id 202 must be compensated after crash before storage commit"
    );

    let text_hits = col.query().text("Graph").execute().await?;
    assert!(
        text_hits.is_empty(),
        "Text query for doc_id 202 must be empty after crash before storage commit"
    );

    let neighbors = col
        .graph_index()
        .neighbors(EntityId::from_key("202")?)
        .await?;
    assert!(
        neighbors.is_empty(),
        "Graph index entry for entity must be compensated"
    );

    // Intent key must be cleaned up from storage
    let intent_check = storage.get(&intent_key).await?;
    assert!(
        intent_check.is_none(),
        "Pending intent marker must be cleaned up after recovery"
    );

    Ok(())
}

/// (c) Storage-Commit erfolgreich, `Committed`-Marker-Write scheitert dauerhaft (T-06):
/// Nach Recovery: Daten bleiben in Storage und Indizes, KEINE Kompensation, Intent wird bereinigt.
#[tokio::test]
async fn test_storage_commit_success_committed_marker_failure_recovery() -> Result<()> {
    let (col, storage, _dir) = create_faulty_collection().await;

    // Inject failure for the Committed marker write in Phase (d)
    storage.fail_committed_put.store(true, Ordering::SeqCst);

    // col.insert() performs a full insert transaction
    col.insert(
        "303",
        &[0.1, 0.2, 0.3, 0.4],
        Some(serde_json::json!({"text": "T06 persistent marker failure text"})),
    )
    .await?;

    let doc_id = DocId::from_key("303")?;
    let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);

    // Document is in storage
    let doc_val_before = storage.get(&doc_key).await?;
    assert!(
        doc_val_before.is_some(),
        "Document must be present in storage"
    );

    // Now run repair() (e.g. on DB restart)
    col.repair().await?;

    // Document must still be present in storage and text index (NO compensation)
    let doc_val_after = storage.get(&doc_key).await?;
    assert!(
        doc_val_after.is_some(),
        "Data must remain in storage after recovery when storage commit succeeded"
    );

    let hits_after = col.query().text("persistent").execute().await?;

    assert!(
        !hits_after.is_empty(),
        "Text query results must remain intact after recovery for committed storage transaction"
    );

    // Pending intent marker must be cleaned up
    let intent_prefix = col.namespaced_key(&[], 3);
    let remaining_intents = storage.scan_prefix(&intent_prefix).await?;
    assert!(
        remaining_intents.is_empty(),
        "Pending intent marker must be cleaned up during recovery"
    );

    Ok(())
}

/// (d) Storage-Commit nicht erfolgt + Pending-Intent:
/// Recovery entfernt Index-Einträge ohne Storage-Datensatz (Vektor, Text, Graph) und bereinigt den Intent.
#[tokio::test]
async fn test_pending_intent_without_storage_commit_cleans_indices() -> Result<()> {
    let (col, storage, _dir) = create_faulty_collection().await;

    let doc_id = DocId::new(404);
    let intent_tx = col.allocate_tx()?;
    let intent_key = col.namespaced_key(&intent_tx.inner().to_le_bytes(), 3);
    let intent = CommitIntent::Pending {
        doc_ids: Arc::new(vec![doc_id]),
        has_text: true,
        has_graph: true,
        stages_completed: 0,
    };
    let intent_bytes = serde_json::to_vec(&intent)?;
    storage.put(intent_tx, &intent_key, &intent_bytes).await?;
    storage.commit(intent_tx).await?;

    // Put entries in vector, graph indices
    let idx_tx = col.allocate_tx()?;
    col.vector_index()
        .insert(idx_tx, doc_id, &[0.0, 1.0, 0.0, 0.0])
        .await?;
    col.vector_index().commit(idx_tx).await?;

    let e = Entity::new(EntityId::from_key("404")?, "Node404", "Doc");
    col.graph_index().add_entity(idx_tx, e).await?;
    col.graph_index().commit(idx_tx).await?;

    // Verify storage has NO record for doc_id 404
    let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
    assert!(storage.get(&doc_key).await?.is_none());

    // Run repair()
    col.repair().await?;

    // Verify vector index entry was removed
    let vec_hits = col.vector_index().search(&[0.0, 1.0, 0.0, 0.0], 10).await?;
    assert!(
        vec_hits.iter().all(|h| h.doc_id != doc_id),
        "Vector index entry without storage record must be removed by repair()"
    );

    // Verify intent key was cleaned up
    assert!(
        storage.get(&intent_key).await?.is_none(),
        "Pending intent marker must be cleaned up by repair()"
    );

    Ok(())
}
