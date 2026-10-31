//! Modul `snapshot` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

use crate::csr::types::{EdgePayload, InternalIndex};
use crate::csr::visibility::is_edge_visible_bitemporal;
use crate::csr::GraphInner;
use crate::ppr::{compute_ppr_with_context, DeletedView, PprContext};
use contextra_types::{EntityId, PprConfig, TxId};
use std::collections::HashMap;

fn filter_node_visible_edges(inner: &GraphInner, i: usize, as_of_tx: TxId) -> Vec<EdgePayload> {
    let mut visible_edges = Vec::new();

    if i < inner.offsets.len() - 1 {
        let start = inner.offsets.get(i).copied().unwrap_or(0);
        let end = inner.offsets.get(i + 1).copied().unwrap_or(0);
        for edge_idx in start..end {
            if let Some(&target) = inner.targets.get(edge_idx) {
                if inner.tombstoned_edges.contains(&(i, target)) {
                    continue;
                }
                let tx_vf = inner.tx_valid_from_at(edge_idx);
                let tx_vt = inner.tx_valid_to_at(edge_idx);
                let biz_vf = inner.business_valid_from_at(edge_idx);
                let biz_vt = inner.business_valid_to_at(edge_idx);

                if is_edge_visible_bitemporal(tx_vf, tx_vt, as_of_tx, biz_vf, biz_vt, None) {
                    if let Some(&weight) = inner.weights.get(edge_idx) {
                        visible_edges.push(EdgePayload {
                            target,
                            weight,
                            tx_valid_from: tx_vf,
                            tx_valid_to: tx_vt,
                            business_valid_from: biz_vf,
                            business_valid_to: biz_vt,
                            source_doc_id: inner.source_doc_id_at(edge_idx),
                        });
                    }
                }
            }
        }
    }

    if let Some(pending) = inner.pending_edges.get(&i) {
        for edge in pending {
            let target = edge.target;
            if inner.tombstoned_edges.contains(&(i, target)) {
                continue;
            }
            if is_edge_visible_bitemporal(
                edge.tx_valid_from,
                edge.tx_valid_to,
                as_of_tx,
                edge.business_valid_from,
                edge.business_valid_to,
                None,
            ) {
                visible_edges.push(edge.clone());
            }
        }
    }

    visible_edges
}

fn populate_snapshot_hyperedges(inner: &GraphInner, snapshot: &mut GraphInner, as_of_tx: TxId) {
    for (&hedge_id, hedge) in &inner.hyperedges {
        if is_edge_visible_bitemporal(
            hedge.tx_valid_from,
            hedge.tx_valid_to,
            as_of_tx,
            hedge.business_valid_from,
            hedge.business_valid_to,
            None,
        ) {
            snapshot.hyperedges.insert(hedge_id, hedge.clone());
            for p in hedge.participants.iter() {
                snapshot
                    .hyperedge_index
                    .entry(p.entity)
                    .or_default()
                    .insert(hedge_id);
            }
            for &child_id in hedge.child_edge_ids.iter() {
                snapshot
                    .child_to_parents
                    .entry(child_id)
                    .or_default()
                    .insert(hedge_id);
            }
        }
    }
}

// ANCHOR[contextra-graph:COMPUTE_PPR_-0304]
/// Funktion `compute_ppr_at` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub(crate) fn compute_ppr_at(
    inner: &GraphInner,
    seed_nodes: &[EntityId],
    config: &PprConfig,
    deleted_nodes: &DeletedView,
    as_of_tx: TxId,
) -> Vec<(EntityId, f32)> {
    let num_nodes = inner.reverse_map.len();
    if num_nodes == 0 || seed_nodes.is_empty() {
        return Vec::new();
    }

    let mut snapshot = GraphInner::new();
    snapshot.id_map = inner.id_map.clone();
    snapshot.reverse_map = inner.reverse_map.clone();
    snapshot.entities = inner.entities.clone();
    snapshot.communities = inner.communities.clone();
    snapshot.communities_loaded = inner.communities_loaded;

    let mut snapshot_pending: HashMap<InternalIndex, Vec<EdgePayload>> = HashMap::new();
    let mut pending_count = 0;

    for i in 0..num_nodes {
        let visible_edges = filter_node_visible_edges(inner, i, as_of_tx);
        if !visible_edges.is_empty() {
            pending_count += visible_edges.len();
            snapshot_pending.insert(i, visible_edges);
        }
    }

    snapshot.pending_edges = snapshot_pending;
    snapshot.pending_edge_count = pending_count;

    populate_snapshot_hyperedges(inner, &mut snapshot, as_of_tx);
    snapshot.compact();

    let mut ctx = PprContext::new();
    compute_ppr_with_context(&snapshot, seed_nodes, config, deleted_nodes, &mut ctx)
}
