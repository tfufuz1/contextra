//! # Transaktions-Rollback beim zweiten Index-Schritt (WP-81)
//!
//! **Zweck:** Erzwingt einen Fehler im zweiten Index-Schritt (mittels Fault-Injektion
//! auf Storage-/Index-Ebene bzw. Staging-Rollback); danach ist keine Teil-Schreibung
//! sichtbar (weder Text noch Vektor noch Graph).
//!
//! Läuft mit `cargo test -p contextra-engine --test tx_second_step_rollback --no-run`.

#![cfg(not(loom))]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_engine::Collection;
use contextra_ports::{BoxFuture, StorageEngine};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::{Result, TxId};
use contextra_vector::HnswIndex;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

/// Fault-Hook Storage Decorator zur gezielten Fehlerinjektion beim Commit / 2. Indexschritt
struct FaultyStorage<S: StorageEngine> {
    inner: Arc<S>,
    fail_second_step_intent: AtomicBool,
}

impl<S: StorageEngine> FaultyStorage<S> {
    fn new(inner: Arc<S>) -> Self {
        Self {
            inner,
            fail_second_step_intent: AtomicBool::new(false),
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
        // Fehler im zweiten Index-Schritt (Intent Logging vor Vektor-/Text-Finalisierung)
        if self.fail_second_step_intent.load(Ordering::SeqCst) && key.starts_with(b"__tx_intent:") {
            return Box::pin(async move {
                Err(contextra_types::ContextraError::Io(
                    "Fault injected in second index-step transaction intent".into(),
                ))
            });
        }
        self.inner.put(tx_id, key, value)
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        self.inner.delete(tx_id, key)
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<u64>> {
        self.inner.commit(tx_id)
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.rollback(tx_id)
    }

    fn scan<'a>(
        &'a self,
        start: &'a [u8],
        end: Option<&'a [u8]>,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<(bytes::Bytes, bytes::Bytes)>>> {
        self.inner.scan(start, end, limit)
    }
}

#[tokio::test]
async fn test_tx_second_step_failure_rollback_leaves_no_partial_writes() -> Result<()> {
    let dir = tempdir().expect("Failed to create temporary directory");
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(lsm_config).await?);
    let fault_storage = Arc::new(FaultyStorage::new(lsm));
    let vector_index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?,
    );
    let text_index = Arc::new(
        contextra_text::TextIndex::new(dir.path().join("text"))
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?,
    );
    let graph_index = Arc::new(
        contextra_graph::CsrGraph::new(dir.path().join("graph"))
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?,
    );

    let col = Collection::new_with_backend(
        "rollback_fault_col".to_string(),
        fault_storage.clone(),
        vector_index.clone(),
        text_index.clone(),
        graph_index.clone(),
        4,
    );

    let doc_key = "partial_write_doc";

    // 1. Manuelle Transaktion beginnen und ersten Schritt (LSM + Text + Vektor-Staging via insert_op) ausführen
    let tx = col.begin_transaction()?;
    col.insert_op(
        &tx,
        doc_key,
        &[0.7, 0.1, 0.1, 0.1],
        Some(json!({
            "text": "Critical transaction data intended for multi-index commit.",
            "status": "pending"
        })),
    )
    .await?;

    // Fault aktivieren: Fehler im zweiten Index-Schritt provozieren
    fault_storage.fail_second_step_intent.store(true, Ordering::SeqCst);

    // Commit versuchen -> muss durch injizierten Fault fehlschlagen
    let commit_res = tx.commit().await;
    assert!(
        commit_res.is_err(),
        "Transaction commit must fail due to fault injection in second index step"
    );

    // 2. Verifikation: Nach Fehlschlag darf KEINE Teilschreibung in irgendeinem Index sichtbar sein
    // 2a. Storage (LSM): Dokument darf nicht existieren
    let retrieved = col.get(doc_key).await?;
    assert!(
        retrieved.is_none(),
        "Document must not exist in storage after failed commit"
    );

    // 2b. Vektor-Index: HNSW darf den Vektor nicht enthalten
    let stats = col.stats().await?;
    assert_eq!(
        stats.num_vectors, 0,
        "Vector index must contain 0 vectors after failed commit"
    );

    let vec_hits = col
        .query()
        .vector(&[0.7, 0.1, 0.1, 0.1])
        .k(5)
        .execute()
        .await?;
    assert!(
        vec_hits.iter().all(|r| r.id != doc_key),
        "Vector search must not return document from failed transaction"
    );

    // 2c. Text-Index (BM25): Text-Index darf keine Treffer liefern
    let text_hits = col
        .query()
        .text("Critical transaction data")
        .k(5)
        .execute()
        .await?;
    assert!(
        text_hits.iter().all(|r| r.id != doc_key),
        "Text search must not return document from failed transaction"
    );

    // 2d. Graph: Entity darf nicht als valider Knoten verbleiben
    if let Ok(eid) = contextra_types::EntityId::from_key(doc_key) {
        let graph_hits = col
            .query()
            .anchors(vec![eid])
            .k(5)
            .execute()
            .await;
        if let Ok(hits) = graph_hits {
            assert!(
                hits.iter().all(|r| r.id != doc_key),
                "Graph search must not return entity from failed transaction"
            );
        }
    }

    Ok(())
}
