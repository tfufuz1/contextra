use super::db_transaction::DbTransaction;
use contextra_ports::{StorageEngine, TextIndex, VectorIndex};
use contextra_types::{ContextraError, DocId, Result};
use std::sync::atomic::Ordering;
use std::sync::Arc;

impl<S: StorageEngine, V: VectorIndex> DbTransaction<S, V> {
    pub(super) fn trigger_kv_store_rollback(&self, doc_ids: &[DocId]) -> Result<()> {
        if let Ok(kv_store) = self.collection.kv_store() {
            let chunk_ids: Vec<u64> = doc_ids
                .iter()
                .map(|d| Ok(d.inner()))
                .collect::<Result<Vec<u64>>>()?;
            kv_store.on_rollback(self.tenant_id, &chunk_ids);
        }
        Ok(())
    }

    pub(super) async fn rollback_internal(&self) {
        let doc_ids = {
            let guard = match self.staged_doc_ids.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            Arc::clone(&guard)
        };
        if let Err(e) = self.trigger_kv_store_rollback(&doc_ids) {
            tracing::error!("[INV-DB-3] Trigger KV store rollback failed: {}", e);
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

    /// Rolls back any uncommitted changes applied to sub-systems in reverse commit order.
    pub async fn rollback(self) -> Result<()> {
        self.committed.store(true, Ordering::Release);
        let doc_ids = {
            let guard = match self.staged_doc_ids.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            Arc::clone(&guard)
        };
        let kv_res = self.trigger_kv_store_rollback(&doc_ids);
        if let Err(ref e) = kv_res {
            tracing::error!("[INV-DB-3] KV store rollback failed: {}", e);
        }

        let text_res = self.collection.text_index.rollback(self.tx_id).await;
        let index_res = self.collection.index.rollback(self.tx_id).await;
        let storage_res = self.collection.storage.rollback(self.tx_id).await;

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
        if let Err(e) = kv_res {
            errors.push(format!("KV Store: {}", e));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback_docid_overflow_handling() {
        let valid_doc = DocId::new(42 as _);
        let res_valid = u64::try_from(valid_doc.inner());
        assert_eq!(res_valid, Ok(42));

        let overflow_raw: u128 = (u64::MAX as u128) + 100;
        assert!(u64::try_from(overflow_raw).is_err());
    }
}
