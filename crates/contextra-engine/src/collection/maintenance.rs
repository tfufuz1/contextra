// FILE-CONTEXT
// ZWECK: Wartungs-, Reparatur- und Bereinigungsoperationen (Index repair, Expiry cleanup).
// INVARIANTEN: repair() stellt LSM<->Index Synchronität nach Crash sicher; Reaping stützt sich auf Snapshot-Sequenzen.
// NICHT-OFFENSICHTLICH: Pending TxIntents werden beim Start via Forward-Commit repariert und markiert.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

use super::{extract_text, Collection, StoredDocument, StoredDocumentMeta};
use crate::decay_controller::{AdaptiveDecayController, DecaySignalInputs};
use crate::metrics_names::{record_histogram_safe, REBUILD_DURATION};
use crate::CommunityDetectionConfig;
use contextra_graph::community::{detect_communities, CommunityAssignment};
use contextra_ports::{StorageEngine, TextIndex, VectorIndex};
use contextra_types::{ContextraError, DocId, EntityId, Result, TxId, EXPIRY_METADATA_KEY};
use std::sync::atomic::Ordering;
use std::sync::Arc;

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Returns a reference to the inverted text index.
    pub fn text_index_ref(&self) -> &contextra_text::inverted::InvertedIndex<S> {
        &self.text_index
    }

    /// Returns a reference to the graph index.
    pub fn graph_index(&self) -> &contextra_graph::CsrGraph {
        &self.graph_index
    }

    /// Returns a shared Arc reference to the graph index.
    pub fn graph_index_arc(&self) -> Arc<contextra_graph::CsrGraph> {
        self.graph_index.clone()
    }

    /// Führt einen Importance-Score-Sweep via Adaptive Decay Controller durch.
    /// Evictet Chunks deren effective_score unter eviction_threshold liegt.
    /// Gibt Anzahl der evictierten Dokumente zurück.
    #[tracing::instrument(level = "trace", skip(self, decay_controller))]
    pub async fn evict_decayed_chunks(
        &self,
        decay_controller: &AdaptiveDecayController,
        max_per_tick: usize,
    ) -> Result<usize> {
        let tombstone_ratio = self
            .index
            .stats()
            .await
            .map(|s| s.deleted_ratio as f32)
            .unwrap_or(0.0);

        let inputs = DecaySignalInputs {
            tombstone_ratio,
            ..DecaySignalInputs::default()
        };

        let current_tx = self.next_tx.load(Ordering::SeqCst);
        let user_prefix = self.namespaced_key(b"", 0);
        let mut evicted_ids = Vec::new();
        let mut cursor: Option<Vec<u8>> = None;
        const BATCH_SIZE: usize = 1000;

        'outer: loop {
            let (batch, next_cursor) = self
                .storage
                .scan_prefix_bounded(&user_prefix, BATCH_SIZE, cursor.as_deref())
                .await?;

            if batch.is_empty() {
                break;
            }

            for (k, v) in batch {
                let key_str = String::from_utf8_lossy(&k).to_string();
                let user_key = if self.name == "default" {
                    key_str
                } else {
                    let prefix_len = self.prefix.len() + 1;
                    if key_str.len() >= prefix_len {
                        key_str[prefix_len..].to_string()
                    } else {
                        key_str
                    }
                };

                if self.name == "default" && user_key.starts_with("__") {
                    continue;
                }

                let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) else {
                    continue;
                };

                let meta_obj = val
                    .get("metadata")
                    .and_then(|m| m.as_object())
                    .or_else(|| val.as_object());

                if let Some(obj) = meta_obj {
                    if let Some(imp_val) = obj.get("importance") {
                        let (base_score, created_tx) = if let Ok(imp) =
                            serde_json::from_value::<contextra_types::MemoryImportance>(
                                imp_val.clone(),
                            ) {
                            (imp.base_score.value(), imp.created_at_tx.inner())
                        } else if let Some(raw_f64) = imp_val.as_f64() {
                            let created = obj
                                .get("created_at_tx")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            (raw_f64 as f32, created)
                        } else {
                            continue;
                        };

                        let elapsed_tx = current_tx.saturating_sub(created_tx);

                        if decay_controller.should_evict(base_score, elapsed_tx, &inputs) {
                            evicted_ids.push(user_key);
                            if evicted_ids.len() >= max_per_tick {
                                break 'outer;
                            }
                        }
                    }
                }
            }

            if let Some(next) = next_cursor {
                cursor = Some(next);
                tokio::task::yield_now().await;
            } else {
                break;
            }
        }

        let count = evicted_ids.len();
        for id in &evicted_ids {
            tracing::info!(
                collection = %self.name,
                id = %id,
                "Decay controller evicting document"
            );
            if let Err(e) = self.delete(id).await {
                tracing::error!(
                    collection = %self.name,
                    id = %id,
                    error = %e,
                    "Decay controller failed to delete document"
                );
            }
        }

        Ok(count)
    }

    /// Repairs the index by re-syncing with the storage.
    ///
    /// Scans the storage for any documents that are missing from the index
    /// and reconciles them. This is critical for crash recovery.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn repair(&self) -> Result<()> {
        let start_nanos = self.clock().monotonic_nanos();
        let res = self.repair_internal().await;
        let end_nanos = self.clock().monotonic_nanos();
        let elapsed_secs = (end_nanos.saturating_sub(start_nanos)) as f64 / 1_000_000_000.0;
        let sink = self.metrics_sink();

        let status_label = if res.is_ok() { "success" } else { "error" };
        record_histogram_safe(
            sink.as_ref(),
            REBUILD_DURATION,
            elapsed_secs,
            &[("status", status_label)],
        );

        res
    }

    async fn repair_internal(&self) -> Result<()> {
        let _guard = self.consolidation_guard.lock().await;
        let mut repair_count = 0;
        let docs = self.storage.scan_prefix(&self.prefix).await?;
        let indexed_ids: std::collections::HashSet<DocId> =
            self.index.all_doc_ids().await?.into_iter().collect();

        tracing::info!("Starting integrity repair for collection '{}'", self.name);
        let start_time = std::time::Instant::now();

        // 1. Scan for pending transaction intents (2-Phase Commit Recovery)
        let intent_prefix = self.namespaced_key(&[], 3);
        let intents = self.storage.scan_prefix(&intent_prefix).await?;
        let recovery_tx = self.allocate_tx()?;
        let mut recovered_any = false;
        let mut recovered_text = false;

        for (intent_key, intent_val) in intents {
            use crate::transaction::CommitIntent;
            if let Ok(intent_variant) = serde_json::from_slice::<CommitIntent>(&intent_val) {
                let (doc_ids, has_text, _has_graph) = match intent_variant {
                    CommitIntent::Pending {
                        doc_ids,
                        has_text,
                        has_graph,
                        ..
                    } => (doc_ids, has_text, has_graph),
                    CommitIntent::Consolidation {
                        source_docs: _,
                        target_id,
                        base_tx: _,
                    } => {
                        let target_doc_key =
                            self.namespaced_key(&target_id.inner().to_le_bytes(), 1);
                        if self.storage.get(&target_doc_key).await?.is_some() {
                            (Arc::new(vec![target_id]), true, false)
                        } else {
                            if let Err(e) = self.storage.delete(recovery_tx, &intent_key).await {
                                tracing::warn!(key = ?intent_key, "Failed to delete interrupted consolidation intent: {e}");
                            }
                            continue;
                        }
                    }
                    _ => continue,
                };

                tracing::info!(
                    "Found pending transaction intent, recovering {} documents (has_text={})",
                    doc_ids.len(),
                    has_text
                );

                let mut any_doc_in_storage = false;
                for &doc_id in doc_ids.iter() {
                    let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
                    if self.storage.get(&doc_key).await?.is_some() {
                        any_doc_in_storage = true;
                        break;
                    }
                }

                if any_doc_in_storage {
                    for &doc_id in doc_ids.iter() {
                        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
                        if let Some(val) = self.storage.get(&doc_key).await? {
                            let meta_id = serde_json::from_slice::<StoredDocumentMeta>(&val)
                                .map(|m| m.id)
                                .ok();

                            let mut stored_doc = None;
                            if let Some(ref id_str) = meta_id {
                                let user_key = self.namespaced_key(id_str.as_bytes(), 0);
                                if let Some(user_val) = self.storage.get(&user_key).await? {
                                    if let Ok(stored) =
                                        StoredDocument::from_slice(&user_val)
                                    {
                                        stored_doc = Some(stored);
                                    }
                                }
                            }

                            if stored_doc.is_none() {
                                if let Ok(full) = StoredDocument::from_slice(&val) {
                                    stored_doc = Some(full);
                                }
                            }

                            if let Some(stored) = stored_doc {
                                if !indexed_ids.contains(&doc_id) {
                                    self.index
                                        .insert(recovery_tx, doc_id, &stored.embedding)
                                        .await?;
                                    repair_count += 1;
                                    recovered_any = true;
                                }

                                if has_text {
                                    if let Some(text) = extract_text(&stored.metadata) {
                                        let text_exists = match self
                                            .text_index
                                            .search_bm25(&text, 1, None)
                                            .await
                                        {
                                            Ok(hits) => hits.iter().any(|(id, _)| *id == doc_id),
                                            Err(_) => false,
                                        };
                                        if !text_exists {
                                            self.text_index
                                                .upsert_document(recovery_tx, doc_id, &text)
                                                .await?;
                                            recovered_text = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    for &doc_id in doc_ids.iter() {
                        if let Err(e) = self.index.delete(recovery_tx, doc_id).await {
                            tracing::warn!(?doc_id, error = %e, "Konnte verwaisten Vektor-Index-Eintrag bei Recovery nicht löschen");
                        } else {
                            recovered_any = true;
                        }

                        if has_text {
                            if let Err(e) =
                                self.text_index.delete_document(recovery_tx, doc_id).await
                            {
                                tracing::warn!(?doc_id, error = %e, "Konnte verwaisten Text-Index-Eintrag bei Recovery nicht löschen");
                            } else {
                                recovered_text = true;
                            }
                        }
                    }
                }

                if let Err(e) = self.storage.delete(recovery_tx, &intent_key).await {
                    tracing::warn!(key = ?intent_key, "Konnte wiederhergestellte TxIntent nicht löschen: {e}");
                }
            }
        }

        if recovered_any {
            self.index.commit(recovery_tx).await?;
        }
        if recovered_text {
            self.text_index.commit(recovery_tx).await?;
        }
        if recovered_any || recovered_text {
            self.storage.commit(recovery_tx).await?;
        }

        // 2. Fallback: Full scan for documents missing from index
        let fallback_tx = self.allocate_tx()?;
        let fallback_any = false;
        let mut fallback_text = false;

        for (namespaced_key, value) in docs {
            if self.name != "default" {
                if namespaced_key.get(self.prefix.len()) != Some(&0) {
                    continue;
                }
            } else if namespaced_key.starts_with(b"__") {
                continue;
            }

            let stored: StoredDocument = match StoredDocument::from_slice(&value) {
                Ok(d) => d,
                Err(e) => {
                    tracing::debug!(
                        key = ?namespaced_key,
                        error = %e,
                        "Überspringe nicht-deserialisierbare Einträge bei repair"
                    );
                    continue;
                }
            };

            let doc_id = DocId::from_key(&stored.id)?;

            if let Some(text) = extract_text(&stored.metadata) {
                if let Ok(bm25_res) = self.text_index.search_bm25(&text, 1, None).await {
                    if !bm25_res.iter().any(|(id, _)| *id == doc_id) {
                        self.text_index
                            .upsert_document(fallback_tx, doc_id, &text)
                            .await?;
                        fallback_text = true;
                    }
                }
            }
        }

        if fallback_any {
            self.index.commit(fallback_tx).await?;
        }
        if fallback_text {
            self.text_index.commit(fallback_tx).await?;
        }

        if repair_count > 0 {
            tracing::info!(
                "Repaired {} missing documents in collection '{}' in {:?}",
                repair_count,
                self.name,
                start_time.elapsed()
            );
        } else {
            tracing::debug!(
                "Integrity check passed for collection '{}' in {:?}",
                self.name,
                start_time.elapsed()
            );
        }

        Ok(())
    }

    /// Returns statistics for the collection's vector index.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn stats(&self) -> Result<contextra_ports::VectorIndexStats> {
        self.index.stats().await
    }

    /// Rebuilds the HNSW index from storage.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn reap_expired_documents(&self, max_expired: usize) -> Result<usize> {
        let current_seq = self.snapshot_seq().await?;
        let user_prefix = self.namespaced_key(b"", 0);
        let mut expired_ids = Vec::new();
        let mut cursor: Option<Vec<u8>> = None;
        const BATCH_SIZE: usize = 1000;

        'outer: loop {
            let (batch, next_cursor) = self
                .storage
                .scan_prefix_bounded(&user_prefix, BATCH_SIZE, cursor.as_deref())
                .await?;

            if batch.is_empty() {
                break;
            }

            for (k, v) in batch {
                let key_str = String::from_utf8_lossy(&k).to_string();
                let user_key = if self.name == "default" {
                    key_str
                } else {
                    let prefix_len = self.prefix.len() + 1;
                    if key_str.len() >= prefix_len {
                        key_str[prefix_len..].to_string()
                    } else {
                        key_str
                    }
                };

                if self.name == "default" && user_key.starts_with("__") {
                    continue;
                }

                let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) else {
                    continue;
                };

                let meta_obj = val
                    .get("metadata")
                    .and_then(|m| m.as_object())
                    .or_else(|| val.as_object());

                if let Some(obj) = meta_obj {
                    if let Some(expiry_seq) = obj.get(EXPIRY_METADATA_KEY).and_then(|v| v.as_u64())
                    {
                        if current_seq >= expiry_seq {
                            expired_ids.push(user_key);
                            if expired_ids.len() >= max_expired {
                                break 'outer;
                            }
                        }
                    }
                }
            }

            if let Some(next) = next_cursor {
                cursor = Some(next);
                tokio::task::yield_now().await;
            } else {
                break;
            }
        }

        let count = expired_ids.len();
        for id in &expired_ids {
            tracing::info!(collection = %self.name, id = %id, "Reaping expired document");
            if let Err(e) = self.delete(id).await {
                tracing::error!(
                    collection = %self.name,
                    id = %id,
                    error = %e,
                    "Expiry cleanup failed to delete document"
                );
            }
        }

        if count > 0 && self.index.is_rebuild_required() {
            tracing::info!(
                collection = %self.name,
                "HNSW tombstone threshold reached after expiry reaping; triggering async rebuild"
            );
            self.index.trigger_rebuild_async();
        }

        Ok(count)
    }

    /// Scans the collection for documents with expired TTLs or decayed importance scores and deletes them.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn trigger_expiry_cleanup(&self) -> Result<usize> {
        let now_ms = self.clock.read().now_unix_nanos() / 1_000_000;

        let now_tx = self.next_tx.load(Ordering::SeqCst);
        let user_prefix = self.namespaced_key(b"", 0);
        let mut expired_ids = Vec::new();
        let mut cursor: Option<Vec<u8>> = None;
        const BATCH_SIZE: usize = 1000;

        loop {
            let (batch, next_cursor) = self
                .storage
                .scan_prefix_bounded(&user_prefix, BATCH_SIZE, cursor.as_deref())
                .await?;

            if batch.is_empty() {
                break;
            }

            for (k, v) in batch {
                let key_str = String::from_utf8_lossy(&k).to_string();
                let user_key = if self.name == "default" {
                    key_str
                } else {
                    let prefix_len = self.prefix.len() + 1;
                    if key_str.len() >= prefix_len {
                        key_str[prefix_len..].to_string()
                    } else {
                        key_str
                    }
                };

                if self.name == "default" && user_key.starts_with("__") {
                    continue;
                }

                let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) else {
                    continue;
                };

                let meta_obj = val
                    .get("metadata")
                    .and_then(|m| m.as_object())
                    .or_else(|| val.as_object());

                let mut marked_for_deletion = false;

                if let Some(obj) = meta_obj {
                    if let Some(ttl_val) = obj.get("ttl_ms").and_then(|v| v.as_u64()) {
                        if ttl_val > 0 {
                            if let Some(created_at) = obj
                                .get("created_at_ms")
                                .or_else(|| obj.get("timestamp_ms"))
                                .and_then(|v| v.as_u64())
                            {
                                if let Some(expire_at) = created_at.checked_add(ttl_val) {
                                    if now_ms >= expire_at {
                                        expired_ids.push(user_key.clone());
                                        marked_for_deletion = true;
                                    }
                                }
                            }
                        }
                    }

                    if !marked_for_deletion {
                        if let Some(imp_val) = obj.get("importance") {
                            if let Ok(imp) = serde_json::from_value::<
                                contextra_types::MemoryImportance,
                            >(imp_val.clone())
                            {
                                if imp.decay != contextra_types::DecayFunction::None {
                                    let effective = imp.effective_score(TxId::new(now_tx));
                                    if effective < Self::DECAY_DELETION_THRESHOLD {
                                        expired_ids.push(user_key);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(next) = next_cursor {
                cursor = Some(next);
                tokio::task::yield_now().await;
            } else {
                break;
            }
        }

        let count = expired_ids.len();
        for id in expired_ids {
            match self.delete(&id).await {
                Ok(_) => {
                    tracing::debug!(
                        collection = %self.name,
                        doc_id = %id,
                        "Reaped expired TTL / decayed document"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        collection = %self.name,
                        id = %id,
                        error = %e,
                        "Expiry cleanup failed to delete expired document"
                    );
                }
            }
        }

        Ok(count)
    }

    /// Updates and persists the importance score and model provenance for a document.
    ///
    /// This method is the canonical public persistence endpoint on [`Collection`] for
    /// recording post-hoc importance assessments (e.g., from Ollama model inference or
    /// cognition pipeline consolidation). Note that [`record_importance_outcome`] in
    /// `crates/contextra-infer-ollama/src/importance.rs:45` feeds outcome feedback into an
    /// in-memory `IsotonicCalibrator`, whereas this method persists the score and `model_id`
    /// into LSM storage.
    pub async fn update_document_importance(
        &self,
        doc_id: &str,
        importance_score: f32,
        model_id: &str,
    ) -> Result<()> {
        let tx = self.allocate_tx()?;
        let user_key = self.namespaced_key(doc_id.as_bytes(), 0);
        let Some(data) = self.storage.get_tracked(tx, &user_key).await? else {
            return Err(contextra_types::ContextraError::NotFound(format!(
                "Document not found: {doc_id}"
            )));
        };
        let mut stored: StoredDocument = StoredDocument::from_slice(&data)?;
        let doc_id_typed = DocId::from_key(doc_id)?;
        let doc_key = self.namespaced_key(&doc_id_typed.inner().to_le_bytes(), 1);

        let meta_obj = match stored.metadata {
            Some(serde_json::Value::Object(ref mut map)) => map,
            _ => {
                stored.metadata = Some(serde_json::json!({}));
                match stored.metadata {
                    Some(serde_json::Value::Object(ref mut map)) => map,
                    _ => {
                        return Err(ContextraError::Serialization(
                            "Failed to initialize document metadata map".to_string(),
                        ))
                    }
                }
            }
        };

        let imp = if let Some(imp_val) = meta_obj.get("importance") {
            if let Ok(mut existing_imp) =
                serde_json::from_value::<contextra_types::MemoryImportance>(imp_val.clone())
            {
                existing_imp.base_score = contextra_types::ImportanceScore::new(importance_score);
                existing_imp
            } else {
                contextra_types::MemoryImportance::new(
                    contextra_types::ImportanceScore::new(importance_score),
                    contextra_types::DecayFunction::None,
                    tx,
                )
            }
        } else {
            contextra_types::MemoryImportance::new(
                contextra_types::ImportanceScore::new(importance_score),
                contextra_types::DecayFunction::None,
                tx,
            )
        };

        if let Ok(val) = serde_json::to_value(imp) {
            meta_obj.insert("importance".to_string(), val);
        }

        meta_obj.insert("model_id".to_string(), serde_json::json!(model_id));

        let meta_only = StoredDocumentMeta::from(&stored);
        let user_bytes = stored.encode_v2()?;
        let doc_bytes = serde_json::to_vec(&meta_only)?;

        let _guard = self.kv_locks.lock_for(doc_id).await;
        drop(_guard);
        self.storage.put(tx, &user_key, &user_bytes).await?;
        self.storage.put(tx, &doc_key, &doc_bytes).await?;
        self.storage.commit(tx).await?;

        Ok(())
    }

    /// Runs Leiden Community Detection on the collection's graph index
    /// and persists the resulting assignments in storage using TxId allocation.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn run_community_detection(&self) -> Result<Vec<CommunityAssignment>> {
        self.run_community_detection_with_config(&CommunityDetectionConfig::default())
            .await
    }

    /// Runs Leiden Community Detection with custom configuration
    /// and persists the resulting assignments in storage using TxId allocation.
    #[tracing::instrument(level = "trace", skip(self, _config))]
    pub async fn run_community_detection_with_config(
        &self,
        _config: &CommunityDetectionConfig,
    ) -> Result<Vec<CommunityAssignment>> {
        let graph_config = contextra_graph::CommunityDetectionConfig::default();
        let assignments = detect_communities(&self.graph_index, &graph_config).await?;
        if assignments.is_empty() {
            return Ok(assignments);
        }

        let tx = self.allocate_tx()?;

        for assignment in &assignments {
            let key = self.namespaced_key(
                format!("__graph:community:{}", assignment.entity_id.inner()).as_bytes(),
                4,
            );
            let val = serde_json::to_vec(&assignment.community_id)?;
            self.storage.put(tx, &key, &val).await?;
        }

        self.storage.commit(tx).await?;
        self.graph_index.set_communities_batch(&assignments);
        Ok(assignments)
    }

    /// Retrieves the persisted community ID for a given entity.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_community(&self, entity_id: EntityId) -> Result<Option<u64>> {
        let key = self.namespaced_key(
            format!("__graph:community:{}", entity_id.inner()).as_bytes(),
            4,
        );
        if let Some(bytes) = self.storage.get(&key).await? {
            let comm_id: u64 = serde_json::from_slice(&bytes).map_err(|e| {
                contextra_types::ContextraError::Internal(format!("community deserialize: {e}"))
            })?;
            Ok(Some(comm_id))
        } else {
            Ok(None)
        }
    }

    /// Retrieves community assignments for a batch of entity IDs in a single operation.
    #[tracing::instrument(level = "trace", skip(self, entity_ids))]
    pub async fn get_communities_batch(
        &self,
        entity_ids: &[EntityId],
    ) -> Result<std::collections::HashMap<EntityId, u64>> {
        if entity_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let graph_map = self.graph_index.get_communities_batch(entity_ids).await?;
        if !graph_map.is_empty() {
            return Ok(graph_map);
        }

        let prefix = self.namespaced_key(b"__graph:community:", 4);
        let entries = self.storage.scan_prefix(&prefix).await?;
        let target_set: std::collections::HashSet<&EntityId> = entity_ids.iter().collect();
        let mut map = std::collections::HashMap::new();

        for (k, v) in entries {
            let id_bytes = match k.strip_prefix(prefix.as_slice()) {
                Some(b) => b,
                None => continue,
            };
            if let Ok(id_str) = std::str::from_utf8(id_bytes) {
                let eid = EntityId::from(id_str);
                if target_set.contains(&eid) {
                    if let Ok(comm_id) = serde_json::from_slice::<u64>(&v) {
                        map.insert(eid, comm_id);
                    }
                }
            }
        }
        Ok(map)
    }

    /// Removes all data belonging to this collection from storage and resets RAM state.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn drop_collection(&self) -> Result<()> {
        let _guard = self.consolidation_guard.lock().await;
        if self.name == "default" {
            return Err(contextra_types::ContextraError::invalid_input(
                "Cannot drop default collection",
            ));
        }

        self.consolidation_in_progress.store(true, Ordering::SeqCst);

        let res = self.drop_collection_internal().await;

        self.consolidation_in_progress
            .store(false, Ordering::SeqCst);
        res
    }

    async fn drop_collection_internal(&self) -> Result<()> {
        let tx = self.allocate_tx()?;

        self.storage.delete_prefix(tx, &self.prefix).await?;

        let txt_prefix = format!("__txt:{}:", self.name).into_bytes();
        self.storage.delete_prefix(tx, &txt_prefix).await?;

        self.text_index.resident_index().clear();

        if let Ok(active_doc_ids) = self.index.all_doc_ids().await {
            for doc_id in active_doc_ids {
                let _ = self.index.delete(tx, doc_id).await;
            }
            let _ = self.index.commit(tx).await;
        }

        self.storage.commit(tx).await?;
        Ok(())
    }
}
