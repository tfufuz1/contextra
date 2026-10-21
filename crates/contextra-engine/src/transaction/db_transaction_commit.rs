use super::compensating_actions::{
    CommitLedger, CompensateHnswAction, CompensateLsmAction, CompensateTextAction,
    RollbackStagedAction,
};
use super::db_transaction::{DbTransaction, StagedTextOp};
use super::intent::CommitIntent;
use crate::metrics_names::{record_histogram_safe, COMMIT_LATENCY};
use contextra_ports::{StorageEngine, TextIndex, VectorIndex};
use contextra_types::{ContextraError, Result, TxId};
use std::sync::atomic::Ordering;
use std::sync::Arc;

impl<S: StorageEngine, V: VectorIndex> DbTransaction<S, V> {
    // TODO(WP-E2 / PR-22): SSI Verdrahtung (Befund MVCC-001):
    // Standard-Leseoperationen (`Collection::get`, `Collection::scan`, `Collection::scan_prefix`)
    // registrieren Lesezugriffe bislang nicht im transaktionalen `ReadSet`. Dadurch ist `read_set.is_empty()`
    // wahr und `validate_ssi_gate` winkt Transaktionen trotz Write-Skew durch.
    // Leseoperationen müssen das `ReadSet` befüllen und hier gegen den `SequenceLogSsiValidator` geprüft werden.
    pub(super) fn validate_ssi_gate(&self) -> Result<()> {
        let read_set = {
            let guard = match self.read_set.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            guard.clone()
        };

        if read_set.is_empty() {
            return Ok(());
        }

        if let Some(ref validator) = self.ssi_validator {
            return validator.validate(self.tx_id, &read_set);
        }

        let storage_ref = self.collection.storage.as_ref();
        if storage_ref.supports_ssi_tracking() {
            Err(ContextraError::Conflict(format!(
                "Serializable isolation validation failed for TxId({}): SSI validator unavailable (fail-closed)",
                self.tx_id.inner()
            )))
        } else {
            Ok(())
        }
    }

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

    /// Commits the transaction atomically across all indices (LSM, HNSW, BM25).
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

    // TODO(WP-E1 / PR-21): Saga-Commit Fix:
    // Der 2PC/Saga-Commit über LSM, HNSW und TextIndex muss strenge Idempotenz und
    // Crash-Wiederherstellbarkeit des `CommitLedger` gewährleisten, damit unvollständig
    // ausgeführte Teil-Commits nach einem Absturz über `CommitIntent` deterministisch kompensiert werden können.
    async fn commit_internal(&self) -> Result<()> {
        if let Err(e) = self.validate_ssi_gate() {
            self.rollback_internal().await;
            return Err(e);
        }

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
            has_graph: false,
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
            has_graph: false,
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

        // Phase (c): Execute staged index steps and commit vector and text indices
        if let Err(e) = self.commit_text_staged().await {
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
        ledger.push(CompensateHnswAction::new(
            self.collection.clone(),
            self.tenant_id,
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
