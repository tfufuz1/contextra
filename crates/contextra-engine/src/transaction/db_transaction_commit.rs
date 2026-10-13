use super::compensating_actions::{
    CommitLedger, CompensateGraphAction, CompensateHnswAction, CompensateLsmAction,
    CompensateTextAction, RollbackStagedAction,
};
use super::db_transaction::{DbTransaction, StagedGraphEdgeOp, StagedGraphEntityOp, StagedTextOp};
use super::intent::CommitIntent;
use crate::metrics_names::{record_histogram_safe, COMMIT_LATENCY};
use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};
use contextra_types::{ContextraError, Result, TenantId, TxId};
use std::sync::atomic::Ordering;
use std::sync::Arc;

impl<S: StorageEngine, V: VectorIndex> DbTransaction<S, V> {
    pub(super) async fn commit_text_staged(&self) -> Result<()> {
        let seq_ops = {
            let mut guard = match self.staged_text_sequence.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };

        if !seq_ops.is_empty() {
            let mut doc_order = Vec::new();
            let mut last_op_map = ahash::AHashMap::new();

            for op in seq_ops {
                let doc_id = match &op {
                    StagedTextOp::Upsert(id, _) => *id,
                    StagedTextOp::Delete(id) => *id,
                };
                if !last_op_map.contains_key(&doc_id) {
                    doc_order.push(doc_id);
                }
                last_op_map.insert(doc_id, op);
            }

            for doc_id in doc_order {
                if let Some(op) = last_op_map.get(&doc_id) {
                    match op {
                        StagedTextOp::Delete(id) => {
                            self.collection
                                .text_index
                                .delete_document(self.tx_id, *id)
                                .await?;
                        }
                        StagedTextOp::Upsert(id, text) => {
                            self.collection
                                .text_index
                                .delete_document(self.tx_id, *id)
                                .await?;
                            self.collection
                                .text_index
                                .upsert_document(self.tx_id, *id, text)
                                .await?;
                        }
                    }
                }
            }
        } else {
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
        }

        Ok(())
    }

    pub(super) async fn commit_graph_staged(&self) -> Result<()> {
        let entity_seq = {
            let mut guard = match self.staged_graph_entity_sequence.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };

        if !entity_seq.is_empty() {
            let mut entity_order = Vec::new();
            let mut last_entity_map = ahash::AHashMap::new();

            for op in entity_seq {
                let eid = match &op {
                    StagedGraphEntityOp::Add(e) => e.id,
                    StagedGraphEntityOp::Delete(eid) => *eid,
                };
                if !last_entity_map.contains_key(&eid) {
                    entity_order.push(eid);
                }
                last_entity_map.insert(eid, op);
            }

            for eid in entity_order {
                if let Some(op) = last_entity_map.get(&eid) {
                    match op {
                        StagedGraphEntityOp::Delete(id) => {
                            self.collection
                                .graph_index
                                .remove_entity(self.tx_id, *id)
                                .await?;
                        }
                        StagedGraphEntityOp::Add(entity) => {
                            self.collection
                                .graph_index
                                .add_entity(self.tx_id, entity.clone())
                                .await?;
                        }
                    }
                }
            }
        } else {
            let entities = {
                let mut guard = match self.staged_graph_entities.lock() {
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
        }

        let edge_seq = {
            let mut guard = match self.staged_graph_edge_sequence.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            std::mem::take(&mut *guard)
        };

        if !edge_seq.is_empty() {
            let mut edge_order = Vec::new();
            let mut last_edge_map = ahash::AHashMap::new();

            for op in edge_seq {
                let key = match &op {
                    StagedGraphEdgeOp::Add(e) => (e.from, e.to),
                    StagedGraphEdgeOp::Delete(from, to) => (*from, *to),
                };
                if !last_edge_map.contains_key(&key) {
                    edge_order.push(key);
                }
                last_edge_map.insert(key, op);
            }

            for key in edge_order {
                if let Some(op) = last_edge_map.get(&key) {
                    match op {
                        StagedGraphEdgeOp::Delete(from, to) => {
                            self.collection
                                .graph_index
                                .remove_edge(self.tx_id, *from, *to)
                                .await?;
                        }
                        StagedGraphEdgeOp::Add(edge) => {
                            GraphIndex::add_edge(
                                &*self.collection.graph_index,
                                self.tx_id,
                                edge.clone(),
                            )
                            .await?;
                        }
                    }
                }
            }
        } else {
            let edges = {
                let mut guard = match self.staged_graph_edges.lock() {
                    Ok(g) => g,
                    Err(p) => p.into_inner(),
                };
                std::mem::take(&mut *guard)
            };

            for edge in edges {
                GraphIndex::add_edge(&*self.collection.graph_index, self.tx_id, edge).await?;
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

        Ok(())
    }

    /// Commits the transaction atomically across all 4 indices (LSM, HNSW, BM25, CSR).
    pub async fn commit(self) -> Result<()> {
        let start_nanos = self.collection.clock().monotonic_nanos();
        let res = self.commit_internal().await;
        let end_nanos = self.collection.clock().monotonic_nanos();
        let elapsed_secs = (end_nanos.saturating_sub(start_nanos)) as f64 / 1_000_000_000.0;
        let sink = self.collection.metrics_sink();

        let status_label = if res.is_ok() { "success" } else { "error" };
        record_histogram_safe(
            sink.as_ref(),
            COMMIT_LATENCY,
            elapsed_secs,
            &[("status", status_label)],
        );

        res
    }

    async fn commit_internal(&self) -> Result<()> {
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
            let seq = match self.staged_text_sequence.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            !ops.is_empty() || !dels.is_empty() || !seq.is_empty()
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
            let e_seq = match self.staged_graph_entity_sequence.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            !ents.is_empty()
                || !edgs.is_empty()
                || !e_dels.is_empty()
                || !ent_dels.is_empty()
                || !hyperedgs.is_empty()
                || !e_seq.is_empty()
        };

        let mut ledger = CommitLedger::new();
        ledger.push(RollbackStagedAction::new(
            self.collection.clone(),
            self.tx_id,
        ));

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
            let _ = ledger.execute_rollback().await;
            return Err(ContextraError::Transaction(format!(
                "Failed to write commit intent: {}",
                e
            )));
        }

        if let Err(e) = self.collection.storage.commit(intent_tx).await {
            let _ = ledger.execute_rollback().await;
            return Err(ContextraError::Transaction(format!(
                "Failed to commit intent transaction: {}",
                e
            )));
        }

        // Phase (b): Commit LSM storage first so document payload and metadata are durable before index staging/commit
        if let Err(storage_err) = self.collection.storage.commit(self.tx_id).await {
            let _ = ledger.execute_rollback().await;
            return Err(ContextraError::Transaction(format!(
                "Storage commit failed: {}",
                storage_err
            )));
        }

        ledger.push(CompensateLsmAction::new(
            self.collection.clone(),
            intent_key.clone(),
            Arc::clone(&doc_ids),
            f_keys,
            r_keys,
        ));

        // Persist stages_completed = 1 marker in pending intent to record LSM storage commit completion
        let stage1_intent = CommitIntent::Pending {
            doc_ids: Arc::clone(&doc_ids),
            has_text,
            has_graph,
            stages_completed: 1,
        };
        if let Ok(stage1_bytes) = serde_json::to_vec(&stage1_intent) {
            let stage1_tx = TxId::new(
                self.collection
                    .next_tx
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            );
            if let Ok(()) = self
                .collection
                .storage
                .put(stage1_tx, &intent_key, &stage1_bytes)
                .await
            {
                let _ = self.collection.storage.commit(stage1_tx).await;
            }
        }

        // Phase (c): Execute staged index steps and commit vector, text, graph indices
        if let Err(e) = self.commit_text_staged().await {
            let _ = ledger.execute_rollback().await;
            return Err(e);
        }

        if let Err(e) = self.commit_graph_staged().await {
            let _ = ledger.execute_rollback().await;
            return Err(e);
        }

        if let Err(index_err) = self.collection.index.commit(self.tx_id).await {
            let _ = ledger.execute_rollback().await;
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
            let _ = ledger.execute_rollback().await;
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
            let _ = ledger.execute_rollback().await;
            return Err(ContextraError::Transaction(format!(
                "Graph index commit failed: {}",
                graph_err
            )));
        }
        ledger.push(CompensateGraphAction {
            collection: self.collection.clone(),
            doc_ids: Arc::clone(&doc_ids),
        });

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
                tokio::time::sleep(std::time::Duration::from_millis(
                    50 * u64::try_from(attempt).unwrap_or(1),
                ))
                .await;
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
