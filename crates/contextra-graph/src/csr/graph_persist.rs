//! Persistenz-Hilfsmethoden für CsrGraph.
//!
//! # Ring-0 Reinheit (R-03)
//! `contextra-graph` besitzt keine eigene Storage-Anbindung oder Dateisystem-I/O.
//! Alle Persistenzinteraktionen erfolgen entkoppelt über den `StorageEngine`-Trait
//! (bzw. `StorageRead`/`StorageWrite`-Trait-Abstraktionen aus `contextra-ports`/`contextra-core`).

use serde::Deserialize;
use std::collections::HashSet;
use std::sync::atomic::Ordering;

use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, Entity, EntityId, Result, TxId};

use super::graph_write::CsrGraph;
use super::types::{
    PersistedEdgePayload, GRAPH_COMMUNITY_PREFIX, GRAPH_EDGE_PREFIX, GRAPH_ENTITY_DELETED_PREFIX,
    GRAPH_ENTITY_PREFIX,
};

impl CsrGraph {
    /// Persistiert eine einzelne Entity in den übergebenen Storage.
    pub async fn persist_entity<S: StorageEngine + ?Sized>(
        &self,
        storage: &S,
        tx: TxId,
        entity: &Entity,
    ) -> Result<()> {
        let key = [GRAPH_ENTITY_PREFIX, entity.id.as_bytes().as_slice()].concat();
        let value = bincode::serialize(entity)
            .map_err(|e| ContextraError::Internal(format!("graph entity serialize: {e}")))?;
        storage.put(tx, &key, &value).await
    }

    /// Persistiert eine einzelne Edge in den übergebenen Storage.
    pub async fn persist_edge<S: StorageEngine + ?Sized>(
        &self,
        storage: &S,
        tx: TxId,
        from: &EntityId,
        to: &EntityId,
        payload: &PersistedEdgePayload,
    ) -> Result<()> {
        let key = [
            GRAPH_EDGE_PREFIX,
            from.as_bytes().as_slice(),
            b":",
            to.as_bytes().as_slice(),
        ]
        .concat();
        let value = bincode::serialize(payload)
            .map_err(|e| ContextraError::Internal(format!("graph edge serialize: {e}")))?;
        storage.put(tx, &key, &value).await
    }

    /// Löscht eine einzelne Edge aus dem übergebenen Storage.
    pub async fn delete_edge_persistence<S: StorageEngine + ?Sized>(
        &self,
        storage: &S,
        tx: TxId,
        from: &EntityId,
        to: &EntityId,
    ) -> Result<()> {
        let key = [
            GRAPH_EDGE_PREFIX,
            from.as_bytes().as_slice(),
            b":",
            to.as_bytes().as_slice(),
        ]
        .concat();
        storage.delete(tx, &key).await
    }

    /// Lädt den kompletten Graph-Zustand aus dem Storage (beim Startup).
    pub async fn load_from_storage<S: StorageEngine + ?Sized>(storage: &S) -> Result<Self> {
        let graph = Self::new();

        // 0. Deleted Entities (Tombstones) laden
        let deleted_entries = storage.scan_prefix(GRAPH_ENTITY_DELETED_PREFIX).await?;
        let mut deleted_entity_ids = HashSet::new();
        for (raw_key, _) in deleted_entries {
            if let Some(key_payload) = raw_key.get(GRAPH_ENTITY_DELETED_PREFIX.len()..) {
                if let Ok(key_str) = std::str::from_utf8(key_payload) {
                    deleted_entity_ids.insert(EntityId::from(key_str));
                }
            }
        }

        // 1. Entities laden
        let entity_entries = storage.scan_prefix(GRAPH_ENTITY_PREFIX).await?;
        let mut entity_count = 0usize;
        for (_, raw_value) in entity_entries {
            let entity: Entity = bincode::deserialize(&raw_value)
                .map_err(|e| ContextraError::Internal(format!("graph entity deserialize: {e}")))?;
            if deleted_entity_ids.contains(&entity.id) {
                continue;
            }
            graph.load_entity_direct(entity)?;
            entity_count += 1;
        }

        // 2. Edges laden
        let edge_entries = storage.scan_prefix(GRAPH_EDGE_PREFIX).await?;
        let mut edge_count = 0usize;
        for (raw_key, raw_value) in edge_entries {
            let (
                weight,
                tx_valid_from,
                tx_valid_to,
                business_valid_from,
                business_valid_to,
                source_doc_id,
            ) = if let Ok(p) = bincode::deserialize::<PersistedEdgePayload>(&raw_value) {
                (
                    p.weight,
                    p.tx_valid_from,
                    p.tx_valid_to,
                    p.business_valid_from,
                    p.business_valid_to,
                    p.source_doc_id,
                )
            } else {
                // Backward compatibility fallback for legacy 5-field PersistedEdgePayload
                #[derive(Deserialize)]
                struct LegacyPersistedEdgePayloadV2 {
                    weight: f32,
                    valid_from: Option<TxId>,
                    valid_to: Option<TxId>,
                    business_valid_from: Option<i64>,
                    business_valid_to: Option<i64>,
                }

                if let Ok(legacy2) =
                    bincode::deserialize::<LegacyPersistedEdgePayloadV2>(&raw_value)
                {
                    (
                        legacy2.weight,
                        legacy2.valid_from,
                        legacy2.valid_to,
                        legacy2.business_valid_from,
                        legacy2.business_valid_to,
                        None,
                    )
                } else {
                    // Backward compatibility fallback for legacy 3-field PersistedEdgePayload
                    #[derive(Deserialize)]
                    struct LegacyPersistedEdgePayloadV1 {
                        weight: f32,
                        valid_from: Option<TxId>,
                        valid_to: Option<TxId>,
                    }

                    if let Ok(legacy) =
                        bincode::deserialize::<LegacyPersistedEdgePayloadV1>(&raw_value)
                    {
                        (
                            legacy.weight,
                            legacy.valid_from,
                            legacy.valid_to,
                            None,
                            None,
                            None,
                        )
                    } else {
                        // Backward compatibility fallback for legacy raw f32 weight values
                        let w: f32 = bincode::deserialize(&raw_value).map_err(|e| {
                            ContextraError::Internal(format!("graph edge deserialize: {e}"))
                        })?;
                        (w, None, None, None, None, None)
                    }
                }
            };

            // Key-Format: "__graph:edge:{from_id}:{to_id}"
            let key_payload = raw_key
                .get(GRAPH_EDGE_PREFIX.len()..)
                .ok_or_else(|| ContextraError::Internal("graph edge key zu kurz".into()))?;

            let key_str = std::str::from_utf8(key_payload)
                .map_err(|e| ContextraError::Internal(format!("graph edge key UTF-8: {e}")))?;

            if let Some((from_str, to_str)) = key_str.split_once(':') {
                let from_id = EntityId::from(from_str);
                let to_id = EntityId::from(to_str);
                graph.load_edge_direct(
                    from_id,
                    to_id,
                    weight,
                    tx_valid_from,
                    tx_valid_to,
                    business_valid_from,
                    business_valid_to,
                    source_doc_id,
                )?;
                edge_count += 1;
            } else {
                tracing::warn!(key = key_str, "Ungültiger graph edge key, übersprungen");
            }
        }

        // 3. CSR kompaktieren — MUSS nach allen Edges aufgerufen werden
        graph.compact();

        if let Ok(last_tx) = storage.last_tx_id().await {
            graph
                .last_tx_id
                .fetch_max(last_tx.inner(), Ordering::SeqCst);
        }

        // CSR rebuilt from LSM on startup; Supersedes relations are re-evaluated to re-apply edge tombstones.
        let doc_entries = storage.scan_prefix(b"").await?;
        let wal_seq = graph.last_tx_id.load(Ordering::SeqCst);
        for (_raw_key, raw_value) in doc_entries {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&raw_value) {
                let meta_obj = val
                    .get("metadata")
                    .and_then(|m| m.as_object())
                    .or_else(|| val.as_object());
                if let Some(obj) = meta_obj {
                    if let Some(links_val) = obj.get("links") {
                        if let Ok(links) = serde_json::from_value::<Vec<contextra_types::MemoryLink>>(
                            links_val.clone(),
                        ) {
                            for link in links {
                                if link.relation == contextra_types::LinkRelation::Supersedes {
                                    let superseded_doc = link.target;
                                    let edge_ids = graph.edges_for_doc(superseded_doc);
                                    if !edge_ids.is_empty() {
                                        let _ = graph
                                            .tombstone_edges_direct(&edge_ids, TxId::new(wal_seq));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Communities laden
        let community_entries = storage.scan_prefix(GRAPH_COMMUNITY_PREFIX).await?;
        let mut community_count = 0usize;
        {
            let mut inner = graph.inner_write();
            inner.communities_loaded = true;
            for (raw_key, raw_value) in community_entries {
                if let Some(key_payload) = raw_key.get(GRAPH_COMMUNITY_PREFIX.len()..) {
                    if let Ok(key_str) = std::str::from_utf8(key_payload) {
                        let eid = EntityId::from(key_str);
                        if let Ok(comm_id) = serde_json::from_slice::<u64>(&raw_value) {
                            inner.communities.insert(eid, comm_id);
                            community_count += 1;
                        }
                    }
                }
            }
        }

        tracing::info!(
            entities = entity_count,
            edges = edge_count,
            communities = community_count,
            last_tx = graph.last_tx_id.load(Ordering::SeqCst),
            "Graph aus Storage geladen und kompaktiert"
        );
        Ok(graph)
    }
}
