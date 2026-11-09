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
        let docs = self.storage.scan_prefix(&self.prefix).await?;
        let indexed_ids: std::collections::HashSet<DocId> =
            self.index.all_doc_ids().await?.into_iter().collect();

        tracing::info!("Starting integrity repair for collection '{}'", self.name);
        let start_nanos = self.clock().monotonic_nanos();

        let repair_count = self
            .recover_pending_transaction_intents(&indexed_ids)
            .await?;

        self.fallback_scan_missing_text_index_documents(docs)
            .await?;

        let elapsed = std::time::Duration::from_nanos(
            self.clock().monotonic_nanos().saturating_sub(start_nanos),
        );
        if repair_count > 0 {
            tracing::info!(
                "Repaired {} missing documents in collection '{}' in {:?}",
                repair_count,
                self.name,
                elapsed
            );
        } else {
            tracing::debug!(
                "Integrity check passed for collection '{}' in {:?}",
                self.name,
                elapsed
            );
        }

        Ok(())
    }

    /// Scans and resolves pending 2-phase commit transaction intents during crash recovery.
    async fn recover_pending_transaction_intents(
        &self,
        indexed_ids: &std::collections::HashSet<DocId>,
    ) -> Result<usize> {
        let intent_prefix = self.namespaced_key(&[], 3);
        let intents = self.storage.scan_prefix(&intent_prefix).await?;
        let recovery_tx = self.allocate_tx()?;
        let mut repair_count = 0;
        let mut recovered_any = false;
        let mut recovered_text = false;
        let mut recovered_graph = false;

        for (intent_key, intent_val) in intents {
            use crate::transaction::CommitIntent;
            if let Ok(intent_variant) = serde_json::from_slice::<CommitIntent>(&intent_val) {
                let (doc_ids, has_text, has_graph) = match intent_variant {
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
                    "Found pending transaction intent, recovering {} documents (has_text={}, has_graph={})",
                    doc_ids.len(),
                    has_text,
                    has_graph
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
                                        serde_json::from_slice::<StoredDocument>(&user_val)
                                    {
                                        stored_doc = Some(stored);
                                    }
                                }
                            }

                            if stored_doc.is_none() {
                                if let Ok(full) = serde_json::from_slice::<StoredDocument>(&val) {
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

                                if has_graph {
                                    if let Ok(eid) = EntityId::from_key(&stored.id) {
                                        let entity = contextra_types::Entity::new(
                                            eid, &stored.id, "Document",
                                        );
                                        if let Err(e) =
                                            self.graph_index.add_entity(recovery_tx, entity).await
                                        {
                                            tracing::warn!(
                                                doc_id = %stored.id,
                                                error = %e,
                                                "Konnte Entity bei Graph-Integritäts-Wiederherstellung nicht hinzufügen"
                                            );
                                        } else {
                                            recovered_graph = true;
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

                        if has_graph {
                            let eid = EntityId::from_doc_id(doc_id);
                            if let Err(e) = self.graph_index.remove_entity(recovery_tx, eid).await {
                                tracing::warn!(?eid, error = %e, "Konnte verwaisten Graph-Index-Eintrag bei Recovery nicht löschen");
                            } else {
                                recovered_graph = true;
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
        if recovered_graph {
            self.graph_index.commit(recovery_tx).await?;
        }
        if recovered_any || recovered_text || recovered_graph {
            self.storage.commit(recovery_tx).await?;
        }

        Ok(repair_count)
    }

    /// Performs fallback scan across collection storage prefix for missing text index entries.
    async fn fallback_scan_missing_text_index_documents(
        &self,
        docs: Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Result<()> {
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

            let stored: StoredDocument = match serde_json::from_slice(&value) {
                Ok(d) => d,
                Err(e) => {
                    tracing::debug!(
                        key = ?namespaced_key,
                        error = %e,
                        "Überspringe nicht-deserialisierbare Einträge bei repair (erwartet für Metadaten-Keys)"
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

        Ok(())
    }

    /// Returns statistics for the collection's vector index.
}
