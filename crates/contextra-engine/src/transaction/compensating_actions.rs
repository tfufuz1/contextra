use super::intent::{CommitIntent, StagedKeyOp};
use crate::Collection;
use contextra_types::{DocId, ContextraError, Result, TenantId, TxId};
use contextra_ports::{BoxFuture, GraphIndex, StorageEngine, TextIndex, VectorIndex};
use std::sync::Arc;

/// Trait representing an undo operation to be executed during transaction rollback/compensation.
pub trait CompensatingAction: Send + Sync {
    fn execute<'a>(&'a self) -> BoxFuture<'a, Result<()>>;
}

/// A ledger of compensating actions collected during 2PC, executed in reverse order on failure.
#[derive(Default)]
pub struct CommitLedger {
    actions: Vec<Box<dyn CompensatingAction>>,
}

impl CommitLedger {
    pub fn new() -> Self {
        Self {
            actions: Vec::with_capacity(8),
        }
    }

    pub fn push<A: CompensatingAction + 'static>(&mut self, action: A) {
        self.actions.push(Box::new(action));
    }

    pub async fn execute_rollback(&mut self) {
        while let Some(action) = self.actions.pop() {
            if let Err(e) = action.execute().await {
                tracing::error!("[INV-DB-3] CompensatingAction execution failed: {}", e);
            }
        }
    }
}

pub struct CompensateLsmAction<S: StorageEngine, V: VectorIndex> {
    pub(super) collection: Collection<S, V>,
    pub(super) intent_key: Vec<u8>,
    pub(super) doc_ids: Arc<Vec<DocId>>,
    pub(super) f_keys: Vec<StagedKeyOp>,
    pub(super) r_keys: Vec<StagedKeyOp>,
}

impl<S: StorageEngine, V: VectorIndex> CompensateLsmAction<S, V> {
    #[allow(dead_code)]
    pub(super) fn new(
        collection: Collection<S, V>,
        intent_key: Vec<u8>,
        doc_ids: Arc<Vec<DocId>>,
        f_keys: Vec<StagedKeyOp>,
        r_keys: Vec<StagedKeyOp>,
    ) -> Self {
        Self {
            collection,
            intent_key,
            doc_ids,
            f_keys,
            r_keys,
        }
    }
}

impl<S: StorageEngine, V: VectorIndex> CompensatingAction for CompensateLsmAction<S, V> {
    fn execute<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
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

                for (f_key, old_val) in &self.f_keys {
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

                for (r_key, old_val) in &self.r_keys {
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

                let abort_bytes = match serde_json::to_vec(&CommitIntent::Aborted) {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::error!("[INV-DB-3] Failed to serialize Aborted intent: {}", e);
                        b"{}".to_vec()
                    }
                };
                if let Err(e) = self
                    .collection
                    .storage
                    .put(rollback_tx, &self.intent_key, &abort_bytes)
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
                    doc_ids = ?self.doc_ids,
                    "[INV-DB-3] FATAL: Compensating transaction failed after {} attempts. \
                     Index DB potential split-brain detected! Repair-on-Open required.",
                    max_attempts
                );
                return Err(ContextraError::Transaction(
                    "CompensateLsmAction failed".into(),
                ));
            }

            Ok(())
        })
    }
}

pub struct CompensateHnswAction<S: StorageEngine, V: VectorIndex> {
    pub(super) collection: Collection<S, V>,
    pub(super) tenant_id: TenantId,
    pub(super) doc_ids: Arc<Vec<DocId>>,
}

impl<S: StorageEngine, V: VectorIndex> CompensateHnswAction<S, V> {
    #[allow(dead_code)]
    pub(super) fn new(
        collection: Collection<S, V>,
        tenant_id: TenantId,
        doc_ids: Arc<Vec<DocId>>,
    ) -> Self {
        Self {
            collection,
            tenant_id,
            doc_ids,
        }
    }
}

impl<S: StorageEngine, V: VectorIndex> CompensatingAction for CompensateHnswAction<S, V> {
    fn execute<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if let Some(kv_store) = self.collection.kv_store() {
                let chunk_ids: Vec<u64> = self.doc_ids.iter().map(|d| d.inner()).collect();
                kv_store.on_rollback(self.tenant_id, &chunk_ids);
            }
            let comp_tx = TxId::new(
                self.collection
                    .next_tx
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            );
            for &doc_id in self.doc_ids.iter() {
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
                return Err(e);
            }
            Ok(())
        })
    }
}

pub struct CompensateTextAction<S: StorageEngine, V: VectorIndex> {
    pub(super) collection: Collection<S, V>,
    pub(super) doc_ids: Arc<Vec<DocId>>,
}

impl<S: StorageEngine, V: VectorIndex> CompensateTextAction<S, V> {
    #[allow(dead_code)]
    pub(super) fn new(collection: Collection<S, V>, doc_ids: Arc<Vec<DocId>>) -> Self {
        Self { collection, doc_ids }
    }
}

impl<S: StorageEngine, V: VectorIndex> CompensatingAction for CompensateTextAction<S, V> {
    fn execute<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let comp_tx = TxId::new(
                self.collection
                    .next_tx
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            );
            for &doc_id in self.doc_ids.iter() {
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
                return Err(e);
            }
            Ok(())
        })
    }
}

pub struct RollbackStagedAction<S: StorageEngine, V: VectorIndex> {
    pub(super) collection: Collection<S, V>,
    pub(super) tx_id: TxId,
}

impl<S: StorageEngine, V: VectorIndex> RollbackStagedAction<S, V> {
    #[allow(dead_code)]
    pub(super) fn new(collection: Collection<S, V>, tx_id: TxId) -> Self {
        Self { collection, tx_id }
    }
}

impl<S: StorageEngine, V: VectorIndex> CompensatingAction for RollbackStagedAction<S, V> {
    fn execute<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
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
            Ok(())
        })
    }
}
