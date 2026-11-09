//! # Modul: maintenance.rs
//!
//! **Zweck:** Wartungsoperationen für einzelne Collections (Kompaktierung, Index-Optimierung).
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

// FILE-CONTEXT
// ZWECK: Wartungs-, Reparatur- und Bereinigungsoperationen (Index repair, Expiry cleanup, Community detection).
// INVARIANTEN: repair() stellt LSM<->Index Synchronität nach Crash sicher; Reaping stützt sich auf Snapshot-Sequenzen.
// NICHT-OFFENSICHTLICH: Pending TxIntents werden beim Start via Forward-Commit repariert und markiert.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

use super::{extract_text, Collection, StoredDocument, StoredDocumentMeta};
use crate::decay_controller::{AdaptiveDecayController, DecaySignalInputs};
use crate::metrics_names::{
    record_counter_safe, record_histogram_safe, REAPER_DOCS_REAPED, REBUILD_DURATION,
};
use contextra_graph::{detect_communities, CommunityAssignment, CommunityDetectionConfig};
use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};
use contextra_types::{ContextraError, DocId, EntityId, Result, TxId, EXPIRY_METADATA_KEY};
use std::sync::atomic::Ordering;
use std::sync::Arc;


impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    pub async fn stats(&self) -> Result<contextra_ports::VectorIndexStats> {
        self.index.stats().await
    }

    /// Rebuilds the HNSW index from storage.
    /// Scans the collection for documents with expired sequence-based TTLs and deletes them in batches.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn reap_expired_documents(&self, max_expired: usize) -> Result<usize> {
        let current_seq = self.snapshot_seq().await?;
        // AI-TAG[SMELL][MAJOR] RESOLVED: AGT-DB-f18d79a2 — reap_expired_documents uses cursor-based batch pagination to avoid 10k silent truncation limit (TS: 2026-09-12T18:43:13Z)
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

        if count > 0 {
            record_counter_safe(
                self.metrics_sink().as_ref(),
                REAPER_DOCS_REAPED,
                count as u64,
                &[],
            );
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
    ///
    /// Reads `created_at_ms` (or `timestamp_ms`) and `ttl_ms` from document metadata for wall-clock TTL,
    /// and `importance` metadata for TxId-based decay sweep (`effective_score < DECAY_DELETION_THRESHOLD`).
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
                    // 1. Working-Memory wall-clock TTL check ZUERST
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

                    // 2. TxId-basierter Decay-Sweep (nur wenn decay != None)
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

        if count > 0 {
            record_counter_safe(
                self.metrics_sink().as_ref(),
                REAPER_DOCS_REAPED,
                count as u64,
                &[],
            );
        }

        Ok(count)
    }

    /// Aktualisiert den Importance-Score sowie die Provenance (Modell-ID) in den Metadaten.
    ///
    /// Diese Funktion entkoppelt die LLM-Bewertung (die z.B. über Ollama geschieht)
    /// von der reinen Datenbank-Operation.
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
        let mut stored: StoredDocument = serde_json::from_slice(&data)?;
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
        let user_bytes = serde_json::to_vec(&stored)?;
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
    #[tracing::instrument(level = "trace", skip(self, config))]
    pub async fn run_community_detection_with_config(
        &self,
        config: &CommunityDetectionConfig,
    ) -> Result<Vec<CommunityAssignment>> {
        let assignments = detect_communities(&self.graph_index, config).await?;
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
        // SSI: nicht tx-gebunden — reiner Lesezugriff für Community-Abfrage
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

        // 1. First try graph_index in-memory batch lookup
        let graph_map = self.graph_index.get_communities_batch(entity_ids).await?;
        if !graph_map.is_empty() {
            return Ok(graph_map);
        }

        // 2. Fallback: single batch scan over collection's community prefix in storage
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

    /// Removes all data belonging to this collection from storage.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn drop_collection(&self) -> Result<()> {
        let _guard = self.consolidation_guard.lock().await;
        let prefix = if self.name == "default" {
            return Err(contextra_types::ContextraError::invalid_input(
                "Cannot drop default collection",
            ));
        } else {
            self.prefix.clone()
        };

        let tx = self.allocate_tx()?;

        // 1. Clean collection data (user keys, docs, rels, intents)
        self.storage.delete_prefix(tx, &prefix).await?;

        // 2. Clean text index namespace (FIND-DB-002)
        let txt_prefix = format!("__txt:{}:", self.name).into_bytes();
        self.storage.delete_prefix(tx, &txt_prefix).await?;

        self.storage.commit(tx).await?;
        Ok(())
    }
}

// ANCHOR[contextra-engine:COL-PER-870]
/// Dokumentation für `PercolationResult` im Rahmen der Collection- und Suchabstraktion.
#[cfg(feature = "graph-connectivity-health")]
pub use contextra_graph::percolation::PercolationConfig;

#[cfg(feature = "graph-connectivity-health")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PercolationResult {
    pub health: Option<f32>,
    pub rebonding_triggered: bool,
    pub new_edges_added: usize,
}

#[cfg(feature = "graph-connectivity-health")]
impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Überprüft die Perkolations-Gesundheit des Wissensgraphen und löst bei Unterschreitung
    /// des Schwellenwerts automatisch einen Re-Bonding-Pass aus.
    #[tracing::instrument(level = "trace", skip(self, config))]
    pub async fn run_percolation_check(
        &self,
        config: &contextra_graph::percolation::PercolationConfig,
    ) -> Result<PercolationResult> {
        let stats = self.graph_index.stats().await?;
        let health = contextra_graph::percolation::compute_percolation_health(
            stats.num_entities,
            stats.num_edges,
        );

        let rebonding_triggered = if let Some(phi) = health {
            contextra_graph::percolation::should_trigger_rebonding(phi, config)
        } else {
            false
        };

        let mut new_edges_added = 0;

        if rebonding_triggered {
            let user_prefix = self.namespaced_key(b"", 0);
            let mut embeddings = std::collections::HashMap::new();
            let mut id_map = std::collections::HashMap::new();
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

                for (_k, v) in batch {
                    if let Ok(stored) = serde_json::from_slice::<StoredDocument>(&v) {
                        if let Ok(eid) = EntityId::from_key(&stored.id) {
                            id_map.insert(eid, stored.id.clone());
                            embeddings.insert(eid, stored.embedding);
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

            let candidate_pairs = contextra_graph::percolation::find_rebonding_candidates(
                self.graph_index.as_ref(),
                &embeddings,
                config,
            )
            .await;

            for (from_id, to_id, _sim) in candidate_pairs {
                if let (Some(from_str), Some(to_str)) = (id_map.get(&from_id), id_map.get(&to_id)) {
                    if self
                        .relate_bidirectional(from_str, to_str, "rebonded")
                        .await
                        .is_ok()
                    {
                        new_edges_added += 2;
                    }
                }
            }
        }

        Ok(PercolationResult {
            health,
            rebonding_triggered,
            new_edges_added,
        })
    }
}
