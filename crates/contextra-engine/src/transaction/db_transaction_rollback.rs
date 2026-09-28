use super::compensating_actions::{CommitLedger, RollbackStagedAction};
use super::db_transaction::DbTransaction;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{DocId, Result};
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

        let mut ledger = CommitLedger::new();
        ledger.push(RollbackStagedAction::new(self.collection.clone(), self.tx_id));
        ledger.execute_rollback().await;

        Ok(())
    }
}
