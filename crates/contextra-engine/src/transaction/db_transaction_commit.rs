use super::compensating_actions::{
    CommitLedger, CompensateHnswAction, CompensateLsmAction, CompensateTextAction,
    RollbackStagedAction,
};
use super::db_transaction::DbTransaction;
use super::intent::CommitIntent;
use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};
use contextra_types::{ContextraError, EntityId, Result, TenantId, TxId};
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

        let mut ledger = CommitLedger::new();
        ledger.push(RollbackStagedAction::new(self.collection.clone(), self.tx_id));

        // Register CompensateLsmAction so storage keys are compensated if any subsequent commit step fails
        let f_keys = {
            let mut guard = match self.staged_forward_keys.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };
        let r_keys = {
            let mut guard = match self.staged_reverse_keys.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };

        ledger.push(CompensateLsmAction::new(
            self.collection.clone(),
            intent_key.clone(),
            Arc::clone(&doc_ids),
            f_keys,
            r_keys,
        ));

        // Phase (a): Write CommitIntent::Pending and commit it to storage durably first (F-20)
        let intent_tx = TxId::new(
            self.collection
                .next_tx
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );
        let intent = CommitIntent::Pending {
            doc_ids: Arc::clone(&doc_ids),
            has_text,
            has_graph,
            stages_completed: 0,
        };
        let intent_bytes = serde_json::to_vec(&intent).map_err(|e| {
            ContextraError::Transaction(format!("Failed to serialize commit intent: {}", e))
        })?;

        if let Err(e) = self
            .collection
            .storage
            .put(intent_tx, &intent_key, &intent_bytes)
            .await
        {
            ledger.execute_rollback().await;
            return Err(ContextraError::Transaction(format!(
                "Failed to write commit intent: {}",
                e
            )));
        }

        if let Err(e) = self.collection.storage.commit(intent_tx).await {
            ledger.execute_rollback().await;
            return Err(ContextraError::Transaction(format!(
                "Failed to commit intent transaction: {}",
                e
            )));
        }

        // Phase (b): Execute staged index steps and commit vector, text, graph indices
        if let Err(e) = self.commit_text_staged().await {
            ledger.execute_rollback().await;
            return Err(e);
        }

        if let Err(e) = self.commit_graph_staged().await {
            ledger.execute_rollback().await;
            return Err(e);
        }

        if let Err(index_err) = self.collection.index.commit(self.tx_id).await {
            ledger.execute_rollback().await;
            return Err(ContextraError::Transaction(format!(
                "HNSW index commit failed: {}",
                index_err
            )));
        }
        let tenant_id = TenantId::try_new(1).unwrap_or_default();
        ledger.push(CompensateHnswAction::new(
            self.collection.clone(),
            tenant_id,
            Arc::clone(&doc_ids),
        ));

        if let Err(text_err) = self.collection.text_index.commit(self.tx_id).await {
            ledger.execute_rollback().await;
            let comp_tx = TxId::new(
                self.collection
                    .next_tx
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            );
            for &doc_id in doc_ids.iter() {
                let eid = EntityId::new(doc_id.inner());
                let _ = self.collection.graph_index.remove_entity(comp_tx, eid).await;
                if let Ok(eid_str) = EntityId::from_key(&doc_id.inner().to_string()) {
                    let _ = self.collection.graph_index.remove_entity(comp_tx, eid_str).await;
                }
            }
            return Err(ContextraError::Transaction(format!(
                "Text index commit failed: {}",
                text_err
            )));
        }
        ledger.push(CompensateTextAction::new(
            self.collection.clone(),
            Arc::clone(&doc_ids),
        ));

        if let Err(graph_err) = self.collection.graph_index.commit(self.tx_id).await {
            ledger.execute_rollback().await;
            let comp_tx = TxId::new(
                self.collection
                    .next_tx
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            );
            for &doc_id in doc_ids.iter() {
                let eid = EntityId::new(doc_id.inner());
                let _ = self.collection.graph_index.remove_entity(comp_tx, eid).await;
                if let Ok(eid_str) = EntityId::from_key(&doc_id.inner().to_string()) {
                    let _ = self.collection.graph_index.remove_entity(comp_tx, eid_str).await;
                }
            }
            return Err(ContextraError::Transaction(format!(
                "Graph index commit failed: {}",
                graph_err
            )));
        }

        // Phase (c): Commit storage
        if let Err(storage_err) = self.collection.storage.commit(self.tx_id).await {
            ledger.execute_rollback().await;
            let comp_tx = TxId::new(
                self.collection
                    .next_tx
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            );
            for &doc_id in doc_ids.iter() {
                let eid = EntityId::new(doc_id.inner());
                let _ = self.collection.graph_index.remove_entity(comp_tx, eid).await;
                if let Ok(eid_str) = EntityId::from_key(&doc_id.inner().to_string()) {
                    let _ = self.collection.graph_index.remove_entity(comp_tx, eid_str).await;
                }
            }
            return Err(ContextraError::Transaction(storage_err.to_string()));
        }

        // Phase (d): Write CommitIntent::Committed with bounded retry and backoff (T-06)
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

        let mut committed_marker_written = false;
        let max_retries = 3;
        for attempt in 1..=max_retries {
            let put_res = self
                .collection
                .storage
                .put(cleanup_tx, &intent_key, &commit_bytes)
                .await;
            if put_res.is_ok() {
                if let Ok(()) = self.collection.storage.commit(cleanup_tx).await {
                    committed_marker_written = true;
                    break;
                }
            }
            if attempt < max_retries {
                tokio::time::sleep(std::time::Duration::from_millis(50 * attempt as u64)).await;
            }
        }

        if !committed_marker_written {
            tracing::error!(
                tx_id = ?self.tx_id,
                "[T-06] Failed to write CommitIntent::Committed after {} attempts; transaction marked as commit-uncertain for recovery inspection",
                max_retries
            );
        }

        // Phase (e): Finalize
        self.committed.store(true, Ordering::Release);
        Ok(())
    }
}
