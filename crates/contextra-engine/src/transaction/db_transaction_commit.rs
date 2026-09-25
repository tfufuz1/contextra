use super::db_transaction::DbTransaction;
use super::intent::CommitIntent;
use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};
use contextra_types::{ContextraError, Result, TxId};
use std::sync::atomic::Ordering;
use std::sync::Arc;

impl<S: StorageEngine, V: VectorIndex> DbTransaction<S, V> {
    pub(super) async fn commit_text_staged(&self) -> Result<()> {
        let text_deletes = {
            let mut guard = match self.staged_text_deletes.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };
        let text_ops = {
            let mut guard = match self.staged_text_ops.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };

        for doc_id in text_deletes {
            self.collection
                .text_index
                .delete_document(self.tx_id, doc_id)
                .await?;
        }

        for (doc_id, text) in text_ops {
            self.collection
                .text_index
                .upsert_document(self.tx_id, doc_id, &text)
                .await?;
        }

        Ok(())
    }

    pub(super) async fn commit_graph_staged(&self) -> Result<()> {
        let entities = {
            let mut guard = match self.staged_graph_entities.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };
        let edges = {
            let mut guard = match self.staged_graph_edges.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };

        for entity in entities {
            self.collection
                .graph_index
                .add_entity(self.tx_id, entity)
                .await?;
        }

        for edge in edges {
            GraphIndex::add_edge(&*self.collection.graph_index, self.tx_id, edge).await?;
        }

        let hyperedges = {
            let mut guard = match self.staged_hyperedges.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };

        for hyperedge in hyperedges {
            self.collection.graph_index.insert_hyperedge(hyperedge);
        }

        let edge_deletes = {
            let mut guard = match self.staged_graph_edge_deletes.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };

        for (from, to) in edge_deletes {
            self.collection
                .graph_index
                .remove_edge(self.tx_id, from, to)
                .await?;
        }

        let entity_deletes = {
            let mut guard = match self.staged_graph_entity_deletes.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };

        for entity_id in entity_deletes {
            self.collection
                .graph_index
                .remove_entity(self.tx_id, entity_id)
                .await?;
        }

        Ok(())
    }

    /// Commits the transaction atomically across all 4 indices (LSM, HNSW, BM25, CSR).
    pub async fn commit(self) -> Result<()> {
        let intent_key = self
            .collection
            .namespaced_key(&self.tx_id.inner().to_le_bytes(), 3);

        let doc_ids = {
            let guard = match self.staged_doc_ids.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            Arc::clone(&guard)
        };

        let has_text = {
            let ops = match self.staged_text_ops.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            let dels = match self.staged_text_deletes.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            !ops.is_empty() || !dels.is_empty()
        };

        let has_graph = {
            let ents = match self.staged_graph_entities.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            let edgs = match self.staged_graph_edges.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            let e_dels = match self.staged_graph_edge_deletes.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            let ent_dels = match self.staged_graph_entity_deletes.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            let hyperedgs = match self.staged_hyperedges.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            !ents.is_empty()
                || !edgs.is_empty()
                || !e_dels.is_empty()
                || !ent_dels.is_empty()
                || !hyperedgs.is_empty()
        };

        if let Err(e) = self.commit_text_staged().await {
            self.rollback_internal().await;
            return Err(e);
        }

        if let Err(e) = self.commit_graph_staged().await {
            self.rollback_internal().await;
            return Err(e);
        }

        let intent = CommitIntent::Pending {
            doc_ids: Arc::clone(&doc_ids),
            has_text,
            has_graph,
            stages_completed: 0,
        };
        let intent_bytes = serde_json::to_vec(&intent).map_err(|e| {
            ContextraError::Transaction(format!("Failed to serialize commit intent: {}", e))
        })?;

        self.collection
            .storage
            .put(self.tx_id, &intent_key, &intent_bytes)
            .await?;

        if let Err(storage_err) = self.collection.storage.commit(self.tx_id).await {
            self.rollback_internal().await;
            return Err(ContextraError::Transaction(storage_err.to_string()));
        }

        if let Err(index_err) = self.collection.index.commit(self.tx_id).await {
            if let Err(e) = self.collection.index.rollback(self.tx_id).await {
                tracing::error!(
                    tx_id = ?self.tx_id,
                    error = ?e,
                    "[INV-DB-3] Failed to rollback vector index after commit failure"
                );
            }
            if let Err(e) = self.collection.graph_index.rollback(self.tx_id).await {
                tracing::error!(
                    "[INV-DB-3] CRITICAL: Failed to rollback graph_index after HNSW commit failure: {}",
                    e
                );
            }
            if let Err(e) = self.collection.text_index.rollback(self.tx_id).await {
                tracing::error!(
                    "[INV-DB-3] CRITICAL: Failed to rollback text_index after HNSW commit failure: {}",
                    e
                );
            }
            self.compensate_lsm(&intent_key, &doc_ids).await;
            return Err(ContextraError::Transaction(format!(
                "HNSW index commit failed, storage rolled back via compensating tx. Error: {}",
                index_err
            )));
        }

        if let Err(text_err) = self.collection.text_index.commit(self.tx_id).await {
            if let Err(e) = self.collection.graph_index.rollback(self.tx_id).await {
                tracing::error!(
                    "[INV-DB-3] CRITICAL: Failed to rollback graph_index after text commit failure: {}",
                    e
                );
            }
            if let Err(e) = self.collection.text_index.rollback(self.tx_id).await {
                tracing::error!(
                    "[INV-DB-3] CRITICAL: Failed to rollback text_index after text commit failure: {}",
                    e
                );
            }
            self.compensate_hnsw(&doc_ids).await;
            self.compensate_lsm(&intent_key, &doc_ids).await;
            return Err(ContextraError::Transaction(format!(
                "Text index commit failed, storage & HNSW rolled back via compensating tx. Error: {}",
                text_err
            )));
        }

        if let Err(graph_err) = self.collection.graph_index.commit(self.tx_id).await {
            if let Err(e) = self.collection.graph_index.rollback(self.tx_id).await {
                tracing::error!(
                    "[INV-DB-3] CRITICAL: Failed to rollback graph_index after graph commit failure: {}",
                    e
                );
            }
            self.compensate_text(&doc_ids).await;
            self.compensate_hnsw(&doc_ids).await;
            self.compensate_lsm(&intent_key, &doc_ids).await;
            return Err(ContextraError::Transaction(format!(
                "Graph index commit failed, storage, HNSW & text index rolled back via compensating tx. Error: {}",
                graph_err
            )));
        }

        let cleanup_tx = TxId::new(
            self.collection
                .next_tx
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );
        let commit_bytes = match serde_json::to_vec(&CommitIntent::Committed) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("Failed to serialize CommitIntent::Committed: {}", e);
                b"{}".to_vec()
            }
        };
        if let Err(e) = self
            .collection
            .storage
            .put(cleanup_tx, &intent_key, &commit_bytes)
            .await
        {
            tracing::warn!("Failed to write committed intent marker: {}", e);
        }

        if let Err(e) = self.collection.storage.commit(cleanup_tx).await {
            tracing::warn!("Failed to commit cleanup transaction: {}", e);
        }

        self.committed.store(true, Ordering::Release);
        Ok(())
    }
}
