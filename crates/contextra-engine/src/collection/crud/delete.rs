use crate::collection::Collection;
use contextra_core::{DocId, EntityId, Result, StorageEngine, VectorIndex};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Deletes a document from the collection by its ID.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn delete(&self, id: &str) -> Result<()> {
        let _guard = self.kv_locks.lock_for(id).await;
        let mut db_tx = self.begin_transaction()?;

        match self.delete_op(&mut db_tx, id).await {
            Ok(_) => {
                db_tx.commit().await?;
                self.check_and_trigger_community_detection(1);
                Ok(())
            }
            Err(e) => {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!("[INV-DB-3] Failed to rollback delete: {}", rollback_err);
                }
                Err(e)
            }
        }
    }

    pub async fn delete_op(
        &self,
        db_tx: &mut crate::transaction::DbTransaction<S, V>,
        id: &str,
    ) -> Result<()> {
        // Find doc keys and tombstone them
        let doc_id = DocId::from_key(id)?;
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        let user_key = self.namespaced_key(id.as_bytes(), 0);

        let old_user_val = self.storage.get_at_seq(&user_key, u64::MAX).await?;
        let old_doc_val = self.storage.get_at_seq(&doc_key, u64::MAX).await?;

        let tx = db_tx.tx_id;

        db_tx.stage_text_delete(doc_id);

        if let Ok(eid) = EntityId::from_key(id) {
            db_tx.stage_graph_entity_delete(eid);
        }

        self.storage.delete(tx, &user_key).await?;
        self.storage.delete(tx, &doc_key).await?;

        db_tx.record_keys_with_old_values(user_key, old_user_val, doc_key, old_doc_val, doc_id);

        self.index.delete(tx, doc_id).await?;

        Ok(())
    }
}
