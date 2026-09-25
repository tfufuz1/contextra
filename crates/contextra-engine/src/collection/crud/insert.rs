use super::internal::{validate_doc_id, validate_embedding};
use crate::collection::{
    ensure_importance_metadata, extract_text, Collection, StoredDocument, StoredDocumentMeta,
};
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{DocId, EntityId, Result, EXPIRY_METADATA_KEY};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Inserts a text document, automatically generating its embedding.
    #[tracing::instrument(level = "trace", skip(self, text, metadata))]
    pub async fn insert_text_only(
        &self,
        id: &str,
        text: &str,
        mut metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let embedding = {
            let embedder = {
                let guard = self.embedder.read();
                guard
                    .as_ref()
                    .ok_or_else(|| {
                        contextra_types::ContextraError::Internal(
                            "No embedder configured for this collection".into(),
                        )
                    })?
                    .clone()
            };
            embedder.embed(text).await?
        };

        let meta = metadata.get_or_insert(serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            if !obj.contains_key("text") {
                obj.insert(
                    "text".to_string(),
                    serde_json::Value::String(text.to_string()),
                );
            }
        }

        self.insert(id, &embedding, metadata).await
    }

    /// Upserts a text document, automatically generating its embedding.
    #[tracing::instrument(level = "trace", skip(self, text, metadata))]
    pub async fn upsert_text_only(
        &self,
        id: &str,
        text: &str,
        mut metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let embedding = {
            let embedder = {
                let guard = self.embedder.read();
                guard
                    .as_ref()
                    .ok_or_else(|| {
                        contextra_types::ContextraError::Internal(
                            "No embedder configured for this collection".into(),
                        )
                    })?
                    .clone()
            };
            embedder.embed(text).await?
        };

        let meta = metadata.get_or_insert(serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            if !obj.contains_key("text") {
                obj.insert(
                    "text".to_string(),
                    serde_json::Value::String(text.to_string()),
                );
            }
        }

        self.upsert(id, &embedding, metadata).await
    }

    /// Inserts a document with a Sequence-based Time-To-Live (TTL in committed ops).
    #[tracing::instrument(level = "trace", skip(self, embedding, metadata))]
    pub async fn insert_with_ttl(
        &self,
        id: &str,
        embedding: &[f32],
        metadata: Option<serde_json::Value>,
        ttl_committed_ops: u64,
    ) -> Result<()> {
        if embedding.len() != self.dimension {
            return Err(contextra_types::ContextraError::invalid_input(format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            )));
        }
        let current_seq = self.snapshot_seq().await?;
        let expiry_seq = current_seq.saturating_add(ttl_committed_ops);

        let mut meta = metadata.unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            obj.insert(
                EXPIRY_METADATA_KEY.to_string(),
                serde_json::json!(expiry_seq),
            );
        } else {
            meta = serde_json::json!({ EXPIRY_METADATA_KEY: expiry_seq });
        }

        self.insert(id, embedding, Some(meta)).await
    }

    /// Speichert ein Dokument mit expliziter kognitiver Gedächtnisklassifikation.
    pub async fn insert_typed(
        &self,
        id: &str,
        embedding: &[f32],
        memory_type: contextra_types::MemoryType,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        if embedding.len() != self.dimension {
            return Err(contextra_types::ContextraError::invalid_input(format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            )));
        }
        let mut meta = metadata.unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            obj.insert(
                "memory_type".to_string(),
                serde_json::to_value(memory_type)
                    .map_err(|e| contextra_types::ContextraError::Serialization(e.to_string()))?,
            );
            if !obj.contains_key("decay_function") {
                if let Ok(decay_val) = serde_json::to_value(memory_type.default_decay()) {
                    obj.insert("decay_function".to_string(), decay_val);
                }
            }
            if !obj.contains_key("ttl_tx") {
                if let Some(ttl) = memory_type.default_ttl_tx() {
                    obj.insert("ttl_tx".to_string(), serde_json::json!(ttl));
                }
            }
        }
        self.insert(id, embedding, Some(meta)).await
    }

    /// Inserts a document with an embedding and optional metadata.
    #[tracing::instrument(level = "trace", skip(self, embedding, metadata))]
    pub async fn insert(
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
        self.apply_insert_backpressure().await;
        let _guard = self.kv_locks.lock_for(id).await;
        self.insert_inner_unlocked(id, embedding, metadata).await
    }

    /// Internal single document insert method without lock acquisition.
    pub(crate) async fn insert_inner_unlocked(
        &self,
        id: &str,
        embedding: &[f32],
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        if id.is_empty() {
            return Err(contextra_types::ContextraError::invalid_input(
                "Document ID cannot be empty",
            ));
        }
        if id.len() > 1024 {
            return Err(contextra_types::ContextraError::invalid_input(
                "Document ID length exceeds maximum allowed limit of 1024 bytes",
            ));
        }
        if embedding.len() != self.dimension {
            return Err(contextra_types::ContextraError::invalid_input(format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            )));
        }
        validate_embedding(embedding)?;

        let db_tx = self.begin_transaction()?;

        match self.insert_op(&db_tx, id, embedding, metadata).await {
            Ok(_) => {
                db_tx.commit().await?;
                self.check_and_trigger_community_detection(1);
                Ok(())
            }
            Err(e) => {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!("[INV-DB-3] Failed to rollback insert: {}", rollback_err);
                }
                Err(e)
            }
        }
    }

    /// Checks if a `doc_id` collision exists for a different user key string.
    pub(crate) async fn check_doc_id_collision(&self, doc_id: DocId, id: &str) -> Result<()> {
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        if let Some(val) = self.storage.get(&doc_key).await? {
            let existing_id = if let Ok(meta) = serde_json::from_slice::<StoredDocumentMeta>(&val) {
                Some(meta.id)
            } else if let Ok(full) = serde_json::from_slice::<StoredDocument>(&val) {
                Some(full.id)
            } else {
                None
            };

            if let Some(existing) = existing_id {
                if existing != id {
                    return Err(contextra_types::ContextraError::Internal(format!(
                        "DocId-Kollision erkannt für Schlüssel '{id}' — bitte Support kontaktieren"
                    )));
                }
            }
        }
        Ok(())
    }

    pub async fn insert_op(
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

        let user_key = self.namespaced_key(id.as_bytes(), 0);
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);

        let old_user_val = self.storage.get_at_seq(&user_key, u64::MAX).await?;
        let old_doc_val = self.storage.get_at_seq(&doc_key, u64::MAX).await?;

        let data = serde_json::to_vec(&stored)?;
        self.storage.put(tx, &user_key, &data).await?;

        let meta_data = serde_json::to_vec(&meta_only)?;
        let written = self.storage.put_if_absent(tx, &doc_key, &meta_data).await?;
        if !written {
            self.check_doc_id_collision(doc_id, id).await?;
        }

        db_tx.record_keys_with_old_values(user_key, old_user_val, doc_key, old_doc_val, doc_id);

        self.index.insert(tx, doc_id, embedding).await?;

        if let Some(text) = extract_text(&metadata) {
            db_tx.stage_text_insert(doc_id, text);
        }

        if let Ok(eid) = EntityId::from_key(id) {
            let entity = contextra_types::Entity::new(eid, id, "Document");
            db_tx.stage_graph_entity(entity);
        }

        Ok(())
    }

    /// Inserts multiple documents in a single atomic transaction under key-granular locks.
    #[tracing::instrument(level = "trace", skip(self, docs))]
    pub async fn insert_many(
        &self,
        docs: &[(String, Vec<f32>, Option<serde_json::Value>)],
    ) -> Result<()> {
        if docs.is_empty() {
            return Err(contextra_types::ContextraError::invalid_input(
                "insert_many requires at least one document",
            ));
        }
        if docs.len() > 10_000 {
            return Err(contextra_types::ContextraError::invalid_input(format!(
                "Batch size {} exceeds maximum allowed limit 10000",
                docs.len()
            )));
        }

        for (_id, embedding, _) in docs {
            if embedding.len() != self.dimension {
                return Err(contextra_types::ContextraError::invalid_input(format!(
                    "Dimension mismatch: expected {}, got {}",
                    self.dimension,
                    embedding.len()
                )));
            }
        }

        self.apply_insert_backpressure().await;
        let _guards = self
            .lock_keys_sorted(docs.iter().map(|(id, _, _)| id.as_str()))
            .await;
        let db_tx = self.begin_transaction()?;

        for (id, embedding, metadata) in docs {
            if embedding.len() != self.dimension {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!(
                        "[INV-DB-3] Failed to rollback insert_many on dimension mismatch: {}",
                        rollback_err
                    );
                }
                return Err(contextra_types::ContextraError::invalid_input(format!(
                    "Dimension mismatch: expected {}, got {}",
                    self.dimension,
                    embedding.len()
                )));
            }
            if let Err(e) = validate_embedding(embedding) {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!(
                        "[INV-DB-3] Failed to rollback insert_many on invalid embedding: {}",
                        rollback_err
                    );
                }
                return Err(e);
            }

            if let Err(e) = self
                .insert_op(&db_tx, id, embedding, metadata.clone())
                .await
            {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!(
                        "[INV-DB-3] Failed to rollback insert_many: {}",
                        rollback_err
                    );
                }
                return Err(e);
            }
        }
        db_tx.commit().await?;
        self.check_and_trigger_community_detection(docs.len());
        Ok(())
    }

    /// Upserts a document (inserts if missing, updates if exists) atomically.
    #[tracing::instrument(level = "trace", skip(self, embedding, metadata))]
    pub async fn upsert(
        &self,
        id: &str,
        embedding: &[f32],
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        if id.is_empty() {
            return Err(contextra_types::ContextraError::invalid_input(
                "Document ID cannot be empty",
            ));
        }
        if id.len() > 1024 {
            return Err(contextra_types::ContextraError::invalid_input(
                "Document ID length exceeds maximum allowed limit of 1024 bytes",
            ));
        }
        if embedding.len() != self.dimension {
            return Err(contextra_types::ContextraError::invalid_input(format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            )));
        }

        self.apply_insert_backpressure().await;
        let _guard = self.kv_locks.lock_for(id).await;
        let db_tx = self.begin_transaction()?;
        let result = self.update_op(&db_tx, id, embedding, metadata).await;

        match result {
            Ok(_) => {
                db_tx.commit().await?;
                self.check_and_trigger_community_detection(1);
                Ok(())
            }
            Err(e) => {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!("[INV-DB-3] Failed to rollback upsert: {}", rollback_err);
                }
                Err(e)
            }
        }
    }

    /// Upserts multiple documents in a single transaction.
    #[tracing::instrument(level = "trace", skip(self, docs))]
    pub async fn upsert_many(
        &self,
        docs: &[(String, Vec<f32>, Option<serde_json::Value>)],
    ) -> Result<()> {
        if docs.is_empty() {
            return Err(contextra_types::ContextraError::invalid_input(
                "upsert_many requires at least one document",
            ));
        }
        if docs.len() > 10_000 {
            return Err(contextra_types::ContextraError::invalid_input(format!(
                "Batch size {} exceeds maximum allowed limit 10000",
                docs.len()
            )));
        }

        for (_id, embedding, _) in docs {
            if embedding.len() != self.dimension {
                return Err(contextra_types::ContextraError::invalid_input(format!(
                    "Dimension mismatch: expected {}, got {}",
                    self.dimension,
                    embedding.len()
                )));
            }
        }

        self.apply_insert_backpressure().await;
        let _guards = self
            .lock_keys_sorted(docs.iter().map(|(id, _, _)| id.as_str()))
            .await;
        let db_tx = self.begin_transaction()?;
        for (id, embedding, metadata) in docs {
            if embedding.len() != self.dimension {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!(
                        "[INV-DB-3] Failed to rollback upsert_many on dimension mismatch: {}",
                        rollback_err
                    );
                }
                return Err(contextra_types::ContextraError::invalid_input(format!(
                    "Dimension mismatch: expected {}, got {}",
                    self.dimension,
                    embedding.len()
                )));
            }
            if let Err(e) = validate_embedding(embedding) {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!(
                        "[INV-DB-3] Failed to rollback upsert_many on invalid embedding: {}",
                        rollback_err
                    );
                }
                return Err(e);
            }
            let result = self
                .update_op(&db_tx, id, embedding, metadata.clone())
                .await;
            if let Err(e) = result {
                if let Err(rollback_err) = db_tx.rollback().await {
                    tracing::error!(
                        "[INV-DB-3] Failed to rollback upsert_many: {}",
                        rollback_err
                    );
                }
                return Err(e);
            }
        }
        db_tx.commit().await?;
        self.check_and_trigger_community_detection(docs.len());
        Ok(())
    }
}
