// FILE-CONTEXT
// ZWECK: Rein lesender Residue-Scan für CSR-Graphen und Verwandte (Evidenz ohne Zeugenausstellung).
// STAND: TS:2026-10-17T00:00:00Z

use crate::csr::CsrGraph;
#[cfg(feature = "edge-reinforcement-learning")]
use crate::edge_reinforcement_buffer::EdgeReinforcementBuffer;
use crate::hyperedge::HyperEdgeId;
use crate::provenance::DocEdgeIndex;
use contextra_types::EntityId;

/// Befund des lesenden Entity-Residue-Scans über alle Graph-Teilstrukturen.
///
/// Reine Evidenz — keine Zeugenausstellung und keine Mutationen!
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphResidueReport {
    /// Die gescannte Entität.
    pub entity_id: EntityId,
    /// Ausgehende CSR-Kanten (from_entity == entity_id, to_entity).
    pub outgoing_edges: Vec<(EntityId, EntityId)>,
    /// Eingehende CSR-Kanten (from_entity, to_entity == entity_id).
    pub incoming_edges: Vec<(EntityId, EntityId)>,
    /// Aktive Hyperkanten, in denen die Entität als Participant auftaucht.
    pub hyperedges: Vec<HyperEdgeId>,
    /// Community-Zuordnung, falls noch in `communities` registriert.
    pub community_assigned: Option<u64>,
    /// Anzahl gepufferter Signale im `EdgeReinforcementBuffer` mit Bezug zu `entity_id`.
    pub reinforcement_buffer_signals: usize,
    /// Anzahl Kanten-Einträge im `DocEdgeIndex`, in denen `entity_id` als Quell- oder Zielknoten auftaucht.
    pub doc_edge_index_references: usize,
}

impl GraphResidueReport {
    pub fn new(entity_id: EntityId) -> Self {
        Self {
            entity_id,
            outgoing_edges: Vec::new(),
            incoming_edges: Vec::new(),
            hyperedges: Vec::new(),
            community_assigned: None,
            reinforcement_buffer_signals: 0,
            doc_edge_index_references: 0,
        }
    }

    /// Liefert `true`, wenn keinerlei Rückstände zur Entität in den gescannten Strukturen verbleiben.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.outgoing_edges.is_empty()
            && self.incoming_edges.is_empty()
            && self.hyperedges.is_empty()
            && self.community_assigned.is_none()
            && self.reinforcement_buffer_signals == 0
            && self.doc_edge_index_references == 0
    }
}

/// Ein rein lesender Scan, der alle Graph-Teilstrukturen nach Bezügen zur angegebenen `entity` durchsucht.
///
/// Gescannte Teilstrukturen:
/// - CSR-Kanten (ausgehend und eingehend, inkl. unkompaktierter Delta-Buffer)
/// - Hyperkanten und Role-Bindings
/// - Community-Zuordnung (`communities` Map)
/// - `edge_store` (Adjazenz-Cache)
/// - `EdgeReinforcementBuffer` (Co-Occurrence- & Traversal-Signale)
/// - `DocEdgeIndex` (Provenienz-Verweise)
pub fn scan_entity_residue(
    graph: &CsrGraph,
    entity: EntityId,
    #[cfg(feature = "edge-reinforcement-learning")] buffer: Option<&EdgeReinforcementBuffer>,
    doc_index: Option<&DocEdgeIndex>,
) -> GraphResidueReport {
    let mut report = GraphResidueReport::new(entity);

    let inner = graph.inner_read();

    // 1. Find all internal indices associated with `entity` (either in id_map or reverse_map)
    let entity_indices: Vec<usize> = if let Some(&idx) = inner.id_map.get(&entity) {
        vec![idx]
    } else {
        inner
            .reverse_map
            .iter()
            .enumerate()
            .filter_map(|(idx, &e)| if e == entity { Some(idx) } else { None })
            .collect()
    };

    for &entity_idx in &entity_indices {
        // Outgoing edges from entity
        let offsets_len_minus_1 = inner.offsets.len().saturating_sub(1);
        if entity_idx < offsets_len_minus_1 {
            let start = *inner.offsets.get(entity_idx).unwrap_or(&0);
            let next_idx = entity_idx.saturating_add(1);
            let end = *inner.offsets.get(next_idx).unwrap_or(&0);
            for j in start..end {
                let Some(&target_idx) = inner.targets.get(j) else { continue };
                if !inner.tombstoned_edges.contains(&(entity_idx, target_idx)) {
                    let target_id = inner
                        .reverse_map
                        .get(target_idx)
                        .copied()
                        .unwrap_or_else(|| EntityId::new(0));
                    if !report.outgoing_edges.contains(&(entity, target_id)) {
                        report.outgoing_edges.push((entity, target_id));
                    }
                }
            }
        }
        if let Some(pending) = inner.pending_edges.get(&entity_idx) {
            for edge in pending {
                if !inner.tombstoned_edges.contains(&(entity_idx, edge.target)) {
                    let target_id = inner
                        .reverse_map
                        .get(edge.target)
                        .copied()
                        .unwrap_or_else(|| EntityId::new(0));
                    if !report.outgoing_edges.contains(&(entity, target_id)) {
                        report.outgoing_edges.push((entity, target_id));
                    }
                }
            }
        }

        // Incoming edges to entity
        let num_nodes = inner.reverse_map.len();
        for src_idx in 0..num_nodes {
            if src_idx == entity_idx {
                continue;
            }
            let src_id = match inner.reverse_map.get(src_idx) {
                Some(&id) => id,
                None => continue,
            };

            let mut found_incoming = false;
            if src_idx < offsets_len_minus_1 {
                let start = *inner.offsets.get(src_idx).unwrap_or(&0);
                let next_idx = src_idx.saturating_add(1);
                let end = *inner.offsets.get(next_idx).unwrap_or(&0);
                for j in start..end {
                    if inner.targets.get(j) == Some(&entity_idx)
                        && !inner.tombstoned_edges.contains(&(src_idx, entity_idx))
                    {
                        found_incoming = true;
                        break;
                    }
                }
            }
            if !found_incoming {
                if let Some(pending) = inner.pending_edges.get(&src_idx) {
                    for edge in pending {
                        if edge.target == entity_idx
                            && !inner.tombstoned_edges.contains(&(src_idx, entity_idx))
                        {
                            found_incoming = true;
                            break;
                        }
                    }
                }
            }
            if found_incoming && !report.incoming_edges.contains(&(src_id, entity)) {
                report.incoming_edges.push((src_id, entity));
            }
        }
    }

    // 2. Check hyperedges and role bindings
    if let Some(hyperedge_ids) = inner.hyperedge_index.get(&entity) {
        for &hid in hyperedge_ids {
            if let Some(hedge) = inner.hyperedges.get(&hid) {
                if hedge.tx_valid_to.is_none()
                    && hedge.participants.iter().any(|p| p.entity == entity)
                {
                    report.hyperedges.push(hid);
                }
            }
        }
    } else {
        // Fallback scan across all hyperedges in case hyperedge_index is out of sync
        for (&hid, hedge) in inner.hyperedges.iter() {
            if hedge.tx_valid_to.is_none()
                && hedge.participants.iter().any(|p| p.entity == entity)
                && !report.hyperedges.contains(&hid)
            {
                report.hyperedges.push(hid);
            }
        }
    }

    // 3. Check community assignment
    if let Some(&comm_id) = inner.communities.get(&entity) {
        report.community_assigned = Some(comm_id);
    }

    // 4. Check edge_store cache
    #[cfg(feature = "edge-reinforcement-learning")]
    {
        if let Some(cached_edges) = inner.edge_store.get(&entity) {
            for edge in cached_edges {
                let target_id = edge.target;
                if !report.outgoing_edges.contains(&(entity, target_id)) {
                    report.outgoing_edges.push((entity, target_id));
                }
            }
        }
    }

    // 5. Check EdgeReinforcementBuffer
    #[cfg(feature = "edge-reinforcement-learning")]
    if let Some(buf) = buffer {
        report.reinforcement_buffer_signals = buf.count_signals_for_entity(entity);
    }

    // 6. Check DocEdgeIndex
    if let Some(idx) = doc_index {
        report.doc_edge_index_references = idx.count_references_for_entity(entity);
    } else {
        report.doc_edge_index_references = graph.doc_edge_index.count_references_for_entity(entity);
    }

    report
}
