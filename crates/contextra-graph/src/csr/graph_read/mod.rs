//! Modul `mod` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

pub mod communities;
pub mod pagerank;
pub mod path_rag;

use crate::GraphIndexExt;
use contextra_types::{EntityId, Result, TxId};

use super::graph_write::CsrGraph;

impl CsrGraph {
    // ANCHOR[contextra-graph:COMPACT-0102]
    /// Funktion `compact` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn compact(&self) {
        let snapshot = self.inner.load();
        let num_nodes = snapshot.reverse_map.len();
        if !snapshot.is_dirty
            && snapshot.pending_edges.is_empty()
            && snapshot.tombstoned_edges.is_empty()
            && snapshot.offsets.len() == num_nodes + 1
        {
            return;
        }
        drop(snapshot);

        let mut inner = self.inner_write();
        let num_nodes = inner.reverse_map.len();
        if inner.is_dirty
            || !inner.pending_edges.is_empty()
            || !inner.tombstoned_edges.is_empty()
            || inner.offsets.len() != num_nodes + 1
        {
            inner.compact();
        }
    }

    // ANCHOR[contextra-graph:COMPACT_ASYN-0103]
    /// Funktion `compact_async` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub async fn compact_async(&self) -> Result<()> {
        let snapshot = self.inner.load();
        let num_nodes = snapshot.reverse_map.len();
        let is_needed = snapshot.is_dirty
            || !snapshot.pending_edges.is_empty()
            || !snapshot.tombstoned_edges.is_empty()
            || snapshot.offsets.len() != num_nodes + 1;

        if !is_needed {
            return Ok(());
        }

        if let Some(ref tracker) = self.config.resource_tracker {
            if !tracker.has_memory_capacity() {
                tracing::warn!(
                    "compact_async deferred due to global ResourceTracker memory budget exhaustion"
                );
                return Ok(());
            }
        }

        if let Some(max_mb) = self.config.max_compaction_peak_memory_mb {
            let estimated_bytes = snapshot.estimate_memory_bytes();
            let estimated_peak_bytes = estimated_bytes * 2;
            let max_bytes = max_mb * 1024 * 1024;
            if estimated_peak_bytes > max_bytes {
                tracing::warn!(
                    estimated_peak_mb = estimated_peak_bytes / (1024 * 1024),
                    max_compaction_peak_memory_mb = max_mb,
                    "compact_async deferred due to compaction memory budget constraint"
                );
                return Ok(());
            }
        }

        self.compact();
        Ok(())
    }

    // ANCHOR[contextra-graph:NEIGHBORS-0104]
    /// Funktion `neighbors` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub async fn neighbors(&self, start: EntityId) -> Result<Vec<EntityId>> {
        let inner = self.inner_read();
        let start_idx = match inner.id_map.get(&start) {
            Some(&idx) => idx,
            None => return Ok(Vec::new()),
        };
        if inner.entity_at(start_idx).is_none() {
            return Ok(Vec::new());
        }

        let mut seen: std::collections::HashSet<EntityId> = std::collections::HashSet::new();
        let mut neighbors: Vec<EntityId> = Vec::new();

        let mut push_if_new = |id: EntityId| {
            if seen.insert(id) {
                neighbors.push(id);
            }
        };

        if start_idx < inner.offsets.len() - 1 {
            let start_e = inner.offsets.get(start_idx).copied().unwrap_or(0);
            let end_e = inner.offsets.get(start_idx + 1).copied().unwrap_or(0);
            for edge_idx in start_e..end_e {
                if let Some(&neighbor_idx) = inner.targets.get(edge_idx) {
                    if !inner.tombstoned_edges.contains(&(start_idx, neighbor_idx))
                        && inner.entity_at(neighbor_idx).is_some()
                    {
                        if let Some(&id) = inner.reverse_map.get(neighbor_idx) {
                            push_if_new(id);
                        }
                    }
                }
            }
        }
        if let Some(pending) = inner.pending_edges.get(&start_idx) {
            for edge in pending {
                let neighbor_idx = edge.target;
                if !inner.tombstoned_edges.contains(&(start_idx, neighbor_idx))
                    && inner.entity_at(neighbor_idx).is_some()
                {
                    if let Some(&id) = inner.reverse_map.get(neighbor_idx) {
                        push_if_new(id);
                    }
                }
            }
        }
        Ok(neighbors)
    }

    // ANCHOR[contextra-graph:ENTITY_COUNT-0105]
    /// Funktion `entity_count` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn entity_count(&self) -> usize {
        self.inner_read()
            .entities
            .iter()
            .filter(|e| e.id != EntityId::new(0))
            .count()
    }

    // ANCHOR[contextra-graph:ENTITY_EXIST-0106]
    /// Funktion `entity_exists` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn entity_exists(&self, id: EntityId) -> bool {
        let inner = self.inner_read();
        if let Some(&idx) = inner.id_map.get(&id) {
            inner.entity_at(idx).is_some()
        } else {
            false
        }
    }

    // ANCHOR[contextra-graph:IS_ENTITY_DE-0107]
    /// Funktion `is_entity_deleted` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub async fn is_entity_deleted(&self, entity: EntityId) -> bool {
        if let Some(storage) = &self.storage {
            let key = format!("graph:entity:deleted:{}", entity.0);
            return storage.get(key.as_bytes()).await.ok().flatten().is_some();
        }
        false
    }

    // ANCHOR[contextra-graph:EDGE_COUNT-0108]
    /// Funktion `edge_count` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn edge_count(&self) -> usize {
        let inner = self.inner_read();
        inner.targets.len()
            + inner.pending_edge_count
            + inner.staged_edges.values().map(|v| v.len()).sum::<usize>()
    }

    // ANCHOR[contextra-graph:REMOVE_ENTIT-0109]
    /// Funktion `remove_entity` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub async fn remove_entity(&self, tx: TxId, entity: EntityId) -> Result<()> {
        GraphIndexExt::remove_entity(self, tx, entity).await
    }
}
