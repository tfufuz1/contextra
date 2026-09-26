use super::internal::{validate_doc_id, validate_embedding};
use crate::collection::{
    ensure_importance_metadata, extract_text, Collection, StoredDocument, StoredDocumentMeta,
};
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{DocId, EntityId, Result};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Updates an existing document in the collection.
    #[tracing::instrument(level = "trace", skip(self, embedding, metadata))]
    pub async fn update(
        &self,
        id: &str,
        embedding: &[f32],
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        if embedding.len() != self.dimension {
            return Err(contextra_types::ContextraError::invalid_input(format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            )));
        }

        let _guard = self.kv_locks.lock_for(id).await;
        drop(_guard);
        let db_tx = self.begin_transaction()?;

        match self.update_op(&db_tx, id, embedding, metadata).await {
            Ok(_) => {
                db_tx.commit().await?;
                self.check_and_trigger_community_detection(1);
                Ok(())
            }
            Err(e) => {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!("[INV-DB-3] Failed to rollback update: {}", rollback_err);
                }
                Err(e)
            }
        }
    }

    pub async fn update_op(
        &self,
        db_tx: &crate::transaction::DbTransaction<S, V>,
        id: &str,
        embedding: &[f32],
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        validate_doc_id(id)?;
        validate_embedding(embedding)?;
        let tx = db_tx.tx_id;
        let doc_id = DocId::from_key(id)?;

        self.check_doc_id_collision(doc_id, id).await?;

        let user_key = self.namespaced_key(id.as_bytes(), 0);
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);

        let old_user_val = self.storage.get_at_seq(&user_key, u64::MAX).await?;
        let old_doc_val = self.storage.get_at_seq(&doc_key, u64::MAX).await?;
        let is_update = old_user_val.is_some() || old_doc_val.is_some();

        // Stage removal from old text index
        db_tx.stage_text_delete(doc_id);

        let mut metadata = metadata;
        let text_opt = extract_text(&metadata);
        ensure_importance_metadata(&mut metadata, tx, text_opt.as_deref());
        if let Some(serde_json::Value::Object(ref mut map)) = metadata {
            map.insert("updated_at_tx".to_string(), serde_json::json!(tx.inner()));
        }

        let stored = StoredDocument {
            id: id.to_string(),
            embedding: embedding.to_vec(),
            metadata: metadata.clone(),
        };
        let meta_only = StoredDocumentMeta::from(&stored);
        let data = serde_json::to_vec(&stored)?;
        let meta_data = serde_json::to_vec(&meta_only)?;

        self.storage.put(tx, &user_key, &data).await?;
        self.storage.put(tx, &doc_key, &meta_data).await?;

        db_tx.record_keys_with_old_values(user_key, old_user_val, doc_key, old_doc_val, doc_id);

        // Stage re-insertion into text index if new text present
        if let Some(new_text) = extract_text(&metadata) {
            db_tx.stage_text_insert(doc_id, new_text);
        }

        // Stage graph entity update
        if let Ok(eid) = EntityId::from_key(id) {
            let entity = contextra_types::Entity::new(eid, id, "Document");
            db_tx.stage_graph_entity(entity);
        }

        // Re-insert into HNSW
        // Recovery-Pfad ist HNSW-Rebuild (>20% deleted nodes) der mit LSM re-synct.
        if is_update {
            if let Err(e) = self.index.delete(tx, doc_id).await {
                tracing::warn!(
                    doc_id = ?doc_id,
                    "HNSW soft-delete fehlgeschlagen: {e}. Doc wird nach HNSW-Rebuild nicht mehr in Vektorsuchen erscheinen."
                );
            }
        }
        self.index.insert(tx, doc_id, embedding).await?;

        Ok(())
    }
}
