use contextra_ports::{LlmTextGenerator, StorageEngine, VectorIndex};
use contextra_types::{
    ContextChunk, ContextraError, DocId, Result, TenantId, TokenBudget, TxId,
};
use contextra_engine::collection::{Collection, StoredDocumentMeta};
use contextra_engine::transaction::CommitIntent;

use super::compactor::ContextCompactor;
use super::types::CompactionStrategy;

/// Optimistic Concurrency Control (OCC) Consolidation Session for Sleep-Cycle Memory Compaction.
///
/// Prevents lost updates / phantom erasures by verifying that no source documents were modified
/// while asynchronous LLM summarization was in progress. Also journals a `CommitIntent::Consolidation`
/// entry into storage for crash resilience (INV-CONSOLIDATE-1, INV-CONSOLIDATE-2).
pub struct ConsolidationSession<'a, S: StorageEngine, V: VectorIndex = contextra_vector::HnswIndex> {
    /// Reference to the active collection.
    pub collection: &'a Collection<S, V>,
    /// Source document IDs and their transaction IDs captured at read snapshot time.
    pub source_docs: Vec<(DocId, TxId)>,
    /// Storage key for the consolidation intent in WAL/LSM.
    pub intent_key: Vec<u8>,
    /// Target document identifier for the synthesized memory.
    pub target_id: DocId,
    /// Base transaction ID allocated when session started.
    pub base_tx: TxId,
}

impl<'a, S: StorageEngine, V: VectorIndex> ConsolidationSession<'a, S, V> {
    /// Starts a consolidation session, snapshotting the version of each source document
    /// and writing a `CommitIntent::Consolidation` intent to storage.
    pub async fn start(
        collection: &'a Collection<S, V>,
        source_doc_ids: &[DocId],
        target_id: DocId,
    ) -> Result<Self> {
        let base_tx = collection.allocate_tx()?;
        let mut source_docs = Vec::with_capacity(source_doc_ids.len());
        for &doc_id in source_doc_ids {
            let tx = collection.get_doc_tx(doc_id).await?.unwrap_or(base_tx);
            source_docs.push((doc_id, tx));
        }

        let intent_key = collection.namespaced_key(&target_id.inner().to_le_bytes(), 3);
        let intent = CommitIntent::Consolidation {
            source_docs: source_docs.clone(),
            target_id,
            base_tx,
        };
        let intent_bytes = serde_json::to_vec(&intent)?;
        collection
            .storage()
            .put(base_tx, &intent_key, &intent_bytes)
            .await?;
        collection.storage().commit(base_tx).await?;

        Ok(Self {
            collection,
            source_docs,
            intent_key,
            target_id,
            base_tx,
        })
    }

    /// Validates optimistic concurrency control: checks that every source document
    /// has NOT been mutated since the consolidation session started.
    pub async fn validate_occ(&self) -> Result<()> {
        for &(doc_id, expected_tx) in &self.source_docs {
            match self.collection.get_doc_tx(doc_id).await? {
                Some(current_tx) => {
                    if current_tx.inner() > expected_tx.inner() {
                        return Err(ContextraError::StaleRead(format!(
                            "OCC conflict: Document {:?} was mutated during consolidation (snapshot tx={}, current tx={})",
                            doc_id, expected_tx, current_tx
                        )));
                    }
                }
                None => {
                    return Err(ContextraError::StaleRead(format!(
                        "OCC conflict: Document {:?} was deleted or missing during consolidation",
                        doc_id
                    )));
                }
            }
        }
        Ok(())
    }

    /// Refreshes the consolidation session by reading the current transaction IDs of all source documents.
    /// This is used to retry consolidation after an OCC conflict, allowing the caller to re-summarize
    /// only the documents that have actually changed.
    /// Returns a list of document IDs that were mutated or deleted since the session started.
    pub async fn refresh(&mut self) -> Result<Vec<DocId>> {
        let mut changed_docs = Vec::new();
        let mut new_source_docs = Vec::with_capacity(self.source_docs.len());

        let new_base_tx = self.collection.allocate_tx()?;

        for &(doc_id, expected_tx) in &self.source_docs {
            match self.collection.get_doc_tx(doc_id).await? {
                Some(current_tx) => {
                    if current_tx.inner() > expected_tx.inner() {
                        changed_docs.push(doc_id);
                    }
                    new_source_docs.push((doc_id, current_tx));
                }
                None => {
                    changed_docs.push(doc_id);
                    // If deleted, we just track the current new_base_tx
                    new_source_docs.push((doc_id, new_base_tx));
                }
            }
        }

        self.source_docs = new_source_docs;

        // Update intent in storage
        let intent = CommitIntent::Consolidation {
            source_docs: self.source_docs.clone(),
            target_id: self.target_id,
            base_tx: new_base_tx,
        };
        let intent_bytes = serde_json::to_vec(&intent)?;
        self.collection
            .storage()
            .put(new_base_tx, &self.intent_key, &intent_bytes)
            .await?;
        self.collection.storage().commit(new_base_tx).await?;
        self.base_tx = new_base_tx;

        Ok(changed_docs)
    }

    /// Cancels / aborts the consolidation session, removing the intent key.
    pub async fn abort(self) -> Result<()> {
        let abort_tx = self.collection.allocate_tx()?;
        self.collection
            .storage()
            .delete(abort_tx, &self.intent_key)
            .await?;
        self.collection.storage().commit(abort_tx).await?;
        Ok(())
    }

    /// Executes the consolidation process using the provided LLM text generator:
    /// validates OCC, reads source document contents, generates summary, and commits.
    pub async fn execute<G>(&self, generator: &G) -> Result<()>
    where
        G: LlmTextGenerator + ?Sized,
    {
        // 1. Strict OCC validation
        self.validate_occ().await?;

        // 2. Fetch source documents
        let mut chunks = Vec::with_capacity(self.source_docs.len());
        for &(doc_id, _) in &self.source_docs {
            let doc_key = self
                .collection
                .namespaced_key(&doc_id.inner().to_le_bytes(), 1);
            if let Some(val) = self.collection.storage().get(&doc_key).await? {
                if let Ok(meta) = serde_json::from_slice::<StoredDocumentMeta>(&val) {
                    if let Some(doc) = self.collection.get(&meta.id).await? {
                        let content = doc
                            .metadata
                            .as_ref()
                            .and_then(|m| m.get("text").or_else(|| m.get("content")))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let token_count = crate::context::ContextManager::estimate_tokens(&content);
                        chunks.push(ContextChunk {
                            doc_id,
                            content,
                            relevance: 1.0,
                            token_count,
                            metadata: doc.metadata,
                            contextual_prefix: None,
                            links: Vec::new(),
                        });
                    }
                }
            }
        }

        if chunks.is_empty() {
            return Err(ContextraError::NotFound(
                "No source documents found for consolidation".to_string(),
            ));
        }

        // 3. Generate summary via LLM
        let compactor =
            ContextCompactor::new(TokenBudget::new(8192, 0), CompactionStrategy::Summarize);
        let default_tenant = TenantId::try_new(1).unwrap_or(TenantId::SYSTEM);
        let compacted = compactor
            .consolidate_via_llm(default_tenant, &chunks, generator, "default")
            .await?;

        let summary_text = compacted
            .retained_chunks
            .first()
            .map(|c| c.content.as_str())
            .unwrap_or_default();

        let target_str_id = format!("doc_{}", self.target_id.inner());
        let dim_f32 = (self.collection.dimension().max(1)) as f32;
        let norm_val = 1.0f32 / dim_f32.sqrt();
        let valid_embedding = vec![norm_val; self.collection.dimension()];

        // 4. Commit
        self.commit_ref(&target_str_id, &valid_embedding, summary_text, None)
            .await
    }

    /// Commits the consolidated document and removes the source documents.
    pub async fn commit(
        self,
        target_string_id: &str,
        embedding: &[f32],
        summary_content: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        self.commit_ref(target_string_id, embedding, summary_content, metadata)
            .await
    }

    /// Commits the consolidated document and removes the source documents (takes `&self`).
    pub async fn commit_ref(
        &self,
        target_string_id: &str,
        embedding: &[f32],
        summary_content: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let _guard = self.collection.consolidation_guard.lock().await;

        // 1. Strict OCC validation under lock
        self.validate_occ().await?;

        // 2. Prepare transaction
        let mut db_tx = self.collection.begin_transaction()?;

        // 3. Insert target consolidated document
        let mut final_metadata = metadata.unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = final_metadata.as_object_mut() {
            obj.insert("consolidated".to_string(), serde_json::json!(true));
            obj.insert(
                "source_doc_ids".to_string(),
                serde_json::json!(self
                    .source_docs
                    .iter()
                    .map(|(d, _)| d.inner())
                    .collect::<Vec<_>>()),
            );
            obj.insert("summary".to_string(), serde_json::json!(summary_content));
        }

        self.collection
            .insert_op(&db_tx, target_string_id, embedding, Some(final_metadata))
            .await?;

        // 4. Delete source docs — Fehler MÜSSEN die Konsolidierung abbrechen
        for &(src_id, _) in &self.source_docs {
            let doc_key = self
                .collection
                .namespaced_key(&src_id.inner().to_le_bytes(), 1);
            match self.collection.storage().get(&doc_key).await? {
                Some(val) => match serde_json::from_slice::<StoredDocumentMeta>(&val) {
                    Ok(meta) => {
                        self.collection
                            .delete_op(&mut db_tx, &meta.id)
                            .await
                            .map_err(|e| {
                                ContextraError::Internal(format!(
                                    "Consolidation commit: failed to delete source doc {:?}: {}",
                                    src_id, e
                                ))
                            })?;
                    }
                    Err(e) => {
                        return Err(ContextraError::Serialization(format!(
                            "Consolidation commit: cannot deserialize meta for source doc {:?}: {}",
                            src_id, e
                        )));
                    }
                },
                None => {
                    tracing::warn!(src_id = ?src_id, "Consolidation: source doc not found, already deleted — skipping");
                }
            }
        }

        // 5. Delete intent key
        let commit_tx = db_tx.tx_id;
        self.collection
            .storage()
            .delete(commit_tx, &self.intent_key)
            .await?;

        // 6. Commit transaction
        db_tx.commit().await?;

        Ok(())
    }
}
