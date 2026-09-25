use super::db_transaction::DbTransaction;
use super::intent::CommitIntent;
use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};
use contextra_types::{ContextraError, DocId, Result, TxId};
use std::sync::atomic::Ordering;
use std::sync::Arc;

impl<S: StorageEngine, V: VectorIndex> DbTransaction<S, V> {
    pub(super) fn trigger_kv_store_rollback(&self, doc_ids: &[DocId]) {
        if let Some(kv_store) = self.collection.kv_store() {
            let chunk_ids: Vec<u64> = doc_ids.iter().map(|d| d.inner()).collect();
            let tenant = contextra_types::TenantId::try_new(1).unwrap_or_default();
            kv_store.on_rollback(tenant, &chunk_ids);
        }
    }

    pub(super) async fn compensate_hnsw(&self, doc_ids: &[DocId]) {
        self.trigger_kv_store_rollback(doc_ids);
        let comp_tx = TxId::new(
            self.collection
                .next_tx
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );
        for &doc_id in doc_ids {
            if let Err(e) = self.collection.index.delete(comp_tx, doc_id).await {
                tracing::error!(
                    "[INV-DB-3] Compensating HNSW delete failed for doc_id {:?}: {}",
                    doc_id,
                    e
                );
            }
        }
        if let Err(e) = self.collection.index.commit(comp_tx).await {
            tracing::error!("[INV-DB-3] Compensating HNSW commit failed: {}", e);
        }
    }

    pub(super) async fn compensate_text(&self, doc_ids: &[DocId]) {
        let comp_tx = TxId::new(
            self.collection
                .next_tx
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );
        for &doc_id in doc_ids {
            if let Err(e) = self
                .collection
                .text_index
                .delete_document(comp_tx, doc_id)
                .await
            {
                tracing::error!(
                    "[INV-DB-3] Compensating text delete failed for doc_id {:?}: {}",
                    doc_id,
                    e
                );
            }
        }
        if let Err(e) = self.collection.text_index.commit(comp_tx).await {
            tracing::error!("[INV-DB-3] Compensating text commit failed: {}", e);
        }
    }

    pub(super) async fn compensate_lsm(&self, intent_key: &[u8], doc_ids: &[DocId]) {
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

        let mut success = false;
        let mut attempts = 0;
        let max_attempts = 3;

        while attempts < max_attempts && !success {
            attempts += 1;
            let rollback_tx = TxId::new(
                self.collection
                    .next_tx
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            );

            let mut comp_failed = false;

            for (f_key, old_val) in &f_keys {
                let res = match old_val {
                    Some(prev_bytes) => {
                        self.collection
                            .storage
                            .put(rollback_tx, f_key, prev_bytes)
                            .await
                    }
                    None => self.collection.storage.delete(rollback_tx, f_key).await,
                };
                if let Err(e) = res {
                    tracing::error!(
                        "[INV-DB-3] Compensating write/delete failed (forward): {}",
                        e
                    );
                    comp_failed = true;
                }
            }

            for (r_key, old_val) in &r_keys {
                if r_key.is_empty() {
                    continue;
                }
                let res = match old_val {
                    Some(prev_bytes) => {
                        self.collection
                            .storage
                            .put(rollback_tx, r_key, prev_bytes)
                            .await
                    }
                    None => self.collection.storage.delete(rollback_tx, r_key).await,
                };
                if let Err(e) = res {
                    tracing::error!(
                        "[INV-DB-3] Compensating write/delete failed (reverse): {}",
                        e
                    );
                    comp_failed = true;
                }
            }

            let abort_bytes = serde_json::to_vec(&CommitIntent::Aborted).unwrap_or_default();
            if let Err(e) = self
                .collection
                .storage
                .put(rollback_tx, intent_key, &abort_bytes)
                .await
            {
                tracing::error!("[INV-DB-3] Failed to write aborted intent marker: {}", e);
                comp_failed = true;
            }

            if let Err(e) = self.collection.storage.commit(rollback_tx).await {
                tracing::error!("[INV-DB-3] Compensating commit failed: {}", e);
                comp_failed = true;
            }

            if !comp_failed {
                success = true;
                tracing::info!(
                    "[INV-DB-3] Compensating transaction succeeded on attempt {}",
                    attempts
                );
            } else if attempts < max_attempts {
                tracing::warn!(
                    "[INV-DB-3] Compensating transaction attempt {} failed. Retrying in 100ms...",
                    attempts
                );
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }

        if !success {
            tracing::error!(
                target: "contextra.invariant",
                event = "split_brain",
                doc_ids = ?doc_ids,
                "[INV-DB-3] FATAL: Compensating transaction failed after {} attempts. \
                 Index DB potential split-brain detected! Repair-on-Open required.",
                max_attempts
            );
        }
    }

    pub(super) async fn rollback_internal(&self) {
        let doc_ids = {
            let guard = match self.staged_doc_ids.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            Arc::clone(&guard)
        };
        self.trigger_kv_store_rollback(&doc_ids);

        if let Err(e) = self.collection.graph_index.rollback(self.tx_id).await {
            tracing::error!("[INV-DB-3] Graph index rollback failed: {}", e);
        }
        if let Err(e) = self.collection.text_index.rollback(self.tx_id).await {
            tracing::error!("[INV-DB-3] Text index rollback failed: {}", e);
        }
        if let Err(e) = self.collection.index.rollback(self.tx_id).await {
            tracing::error!("[INV-DB-3] Vector index rollback failed: {}", e);
        }
        if let Err(e) = self.collection.storage.rollback(self.tx_id).await {
            tracing::error!("[INV-DB-3] Storage rollback failed: {}", e);
        }
    }

    /// Rolls back any uncommitted changes applied to all 4 sub-systems in reverse commit order.
    pub async fn rollback(self) -> Result<()> {
        self.committed.store(true, Ordering::Release);
        let doc_ids = {
            let guard = match self.staged_doc_ids.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            Arc::clone(&guard)
        };
        self.trigger_kv_store_rollback(&doc_ids);

        let graph_res = self.collection.graph_index.rollback(self.tx_id).await;
        let text_res = self.collection.text_index.rollback(self.tx_id).await;
        let index_res = self.collection.index.rollback(self.tx_id).await;
        let storage_res = self.collection.storage.rollback(self.tx_id).await;

        if let Err(ref e) = graph_res {
            tracing::error!("[INV-DB-3] Graph index rollback failed: {}", e);
        }
        if let Err(ref e) = text_res {
            tracing::error!("[INV-DB-3] Text index rollback failed: {}", e);
        }
        if let Err(ref e) = index_res {
            tracing::error!("[INV-DB-3] Vector index rollback failed: {}", e);
        }
        if let Err(ref e) = storage_res {
            tracing::error!("[INV-DB-3] Storage rollback failed: {}", e);
        }

        let mut errors = Vec::new();
        if let Err(e) = graph_res {
            errors.push(format!("Graph: {}", e));
        }
        if let Err(e) = text_res {
            errors.push(format!("Text: {}", e));
        }
        if let Err(e) = index_res {
            errors.push(format!("Vector: {}", e));
        }
        if let Err(e) = storage_res {
            errors.push(format!("Storage: {}", e));
        }

        if !errors.is_empty() {
            return Err(ContextraError::Transaction(format!(
                "Rollback failed: {}",
                errors.join(", ")
            )));
        }

        Ok(())
    }
}
