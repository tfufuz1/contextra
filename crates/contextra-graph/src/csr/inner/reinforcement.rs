//! Modul `reinforcement` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

#[cfg(feature = "edge-reinforcement-learning")]
use contextra_types::EntityId;

#[cfg(feature = "edge-reinforcement-learning")]
use super::super::types::CsrEdge;
#[cfg(feature = "edge-reinforcement-learning")]
use super::GraphInner;

#[cfg(feature = "edge-reinforcement-learning")]
impl GraphInner {
    // ANCHOR[contextra-graph:FIND_EDGE_MU-0173]
    /// Funktion `find_edge_mut_by_entities` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn find_edge_mut_by_entities(
        &mut self,
        from: EntityId,
        to: EntityId,
    ) -> Option<&mut CsrEdge> {
        let from_idx = *self.id_map.get(&from)?;
        let to_idx = *self.id_map.get(&to)?;

        let exists_pending = self
            .pending_edges
            .get(&from_idx)
            .is_some_and(|edges| edges.iter().any(|e| e.target == to_idx));

        let exists_csr = if from_idx < self.offsets.len() - 1 {
            let start = self.offsets.get(from_idx).copied().unwrap_or(0);
            let end = self.offsets.get(from_idx + 1).copied().unwrap_or(0);
            self.targets
                .get(start..end)
                .map_or(false, |slice| slice.contains(&to_idx))
        } else {
            false
        };

        if !exists_pending && !exists_csr {
            return None;
        }

        let vec = self.edge_store.entry(from).or_default();
        if !vec.iter().any(|e| e.target == to) {
            let current_weight = if let Some(pending) = self.pending_edges.get(&from_idx) {
                pending
                    .iter()
                    .find(|e| e.target == to_idx)
                    .map(|e| e.weight)
            } else {
                None
            }
            .unwrap_or_else(|| {
                if from_idx < self.offsets.len() - 1 {
                    let start = self.offsets.get(from_idx).copied().unwrap_or(0);
                    let end = self.offsets.get(from_idx + 1).copied().unwrap_or(0);
                    for j in start..end {
                        if self.targets.get(j) == Some(&to_idx) {
                            return self.weights.get(j).copied().unwrap_or(1.0);
                        }
                    }
                }
                1.0
            });

            vec.push(CsrEdge::new(to, current_weight));
        }

        self.edge_store
            .get_mut(&from)?
            .iter_mut()
            .find(|e| e.target == to)
    }

    // ANCHOR[contextra-graph:ALL_ENTITY_I-0174]
    /// Funktion `all_entity_ids` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn all_entity_ids(&self) -> Vec<EntityId> {
        self.reverse_map.clone()
    }

    // ANCHOR[contextra-graph:OUTGOING_EDG-0175]
    /// Funktion `outgoing_edges_mut` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn outgoing_edges_mut(&mut self, entity_id: EntityId) -> &mut [CsrEdge] {
        if let Some(from_idx) = self.id_map.get(&entity_id).copied() {
            let mut target_ids = Vec::new();
            if let Some(pending) = self.pending_edges.get(&from_idx) {
                for p in pending {
                    if let Some(&t_id) = self.reverse_map.get(p.target) {
                        target_ids.push((t_id, p.weight));
                    }
                }
            }
            if from_idx < self.offsets.len() - 1 {
                let start = self.offsets.get(from_idx).copied().unwrap_or(0);
                let end = self.offsets.get(from_idx + 1).copied().unwrap_or(0);
                for j in start..end {
                    if let (Some(&t_idx), Some(&w)) = (self.targets.get(j), self.weights.get(j)) {
                        if let Some(&t_id) = self.reverse_map.get(t_idx) {
                            target_ids.push((t_id, w));
                        }
                    }
                }
            }

            let vec = self.edge_store.entry(entity_id).or_default();
            for (t_id, w) in target_ids {
                if !vec.iter().any(|e| e.target == t_id) {
                    vec.push(CsrEdge::new(t_id, w));
                }
            }
        }

        self.edge_store
            .get_mut(&entity_id)
            .map(|v| v.as_mut_slice())
            .unwrap_or(&mut [])
    }

    // ANCHOR[contextra-graph:SYNC_EDGE_RE-0176]
    /// Funktion `sync_edge_reinforcement_weights` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn sync_edge_reinforcement_weights(
        &mut self,
        config: &crate::edge_reinforcement::EdgeReinforcementConfig,
    ) {
        use crate::edge_reinforcement::compute_edge_weight;

        for (&from_id, edges) in &self.edge_store {
            let Some(&from_idx) = self.id_map.get(&from_id) else {
                continue;
            };

            for edge in edges {
                let new_weight = compute_edge_weight(
                    edge.cooccurrence_weight,
                    edge.traversal_weight,
                    config.alpha,
                );

                if let Some(&to_idx) = self.id_map.get(&edge.target) {
                    if let Some(pending) = self.pending_edges.get_mut(&from_idx) {
                        for p_edge in pending.iter_mut() {
                            if p_edge.target == to_idx {
                                p_edge.weight = new_weight;
                            }
                        }
                    }

                    if from_idx < self.offsets.len() - 1 {
                        let start = self.offsets.get(from_idx).copied().unwrap_or(0);
                        let end = self.offsets.get(from_idx + 1).copied().unwrap_or(0);
                        for j in start..end {
                            if self.targets.get(j) == Some(&to_idx) {
                                if let Some(slot) = self.weights.get_mut(j) {
                                    *slot = new_weight;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
