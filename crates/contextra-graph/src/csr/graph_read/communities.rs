//! Modul `communities` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

use super::super::graph_write::CsrGraph;
use super::super::types::GRAPH_COMMUNITY_PREFIX;
use contextra_types::{EntityId, Result};
use std::collections::HashMap;

impl CsrGraph {
    // ANCHOR[contextra-graph:SET_COMMUNIT-0100]
    /// Funktion `set_communities_batch` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn set_communities_batch(&self, assignments: &[crate::CommunityAssignment]) {
        let mut inner = self.inner_write();
        for a in assignments {
            inner.communities.insert(a.entity_id, a.community_id);
        }
        inner.communities_loaded = true;
    }

    // ANCHOR[contextra-graph:GET_COMMUNIT-0101]
    /// Funktion `get_communities_batch` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub async fn get_communities_batch(
        &self,
        entity_ids: &[EntityId],
    ) -> Result<HashMap<EntityId, u64>> {
        let (map, done) = {
            let inner = self.inner_read();
            let mut map = HashMap::with_capacity(entity_ids.len());
            for &eid in entity_ids {
                if let Some(&comm_id) = inner.communities.get(&eid) {
                    map.insert(eid, comm_id);
                }
            }
            let done = inner.communities_loaded || self.storage.is_none() || entity_ids.is_empty();
            (map, done)
        };

        if done {
            return Ok(map);
        }

        if let Some(ref storage) = self.storage {
            let entries = storage.scan_prefix(GRAPH_COMMUNITY_PREFIX).await?;
            let mut inner = self.inner_write();
            inner.communities_loaded = true;
            for (raw_key, raw_val) in entries {
                if let Some(key_payload) = raw_key.get(GRAPH_COMMUNITY_PREFIX.len()..) {
                    if let Ok(key_str) = std::str::from_utf8(key_payload) {
                        let eid = EntityId::from(key_str);
                        if let Ok(comm_id) = serde_json::from_slice::<u64>(&raw_val) {
                            inner.communities.insert(eid, comm_id);
                        }
                    }
                }
            }
            let mut map = HashMap::with_capacity(entity_ids.len());
            for &eid in entity_ids {
                if let Some(&comm_id) = inner.communities.get(&eid) {
                    map.insert(eid, comm_id);
                }
            }
            Ok(map)
        } else {
            Ok(HashMap::new())
        }
    }
}
