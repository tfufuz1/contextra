//! Snapshot-isolated Personalized PageRank (PPR) power iteration support.

// FILE-CONTEXT
// STAND: 2026-09-24
// ZWECK: Multi-Version Concurrency Control (MVCC) snapshot-isolated PPR computation.
// INVARIANTEN: Bit-identical output for identical snapshot state; panic-free execution.
// PERFORMANCE: O(E) snapshot copy overhead per query (ADR-024).

use crate::csr::types::{EdgePayload, InternalIndex};
use crate::csr::visibility::is_edge_visible_bitemporal;
use crate::csr::GraphInner;
use crate::ppr::{compute_ppr_with_context, DeletedView, PprContext};
use contextra_core::{EntityId, PprConfig, TxId};
use std::collections::HashMap;

/// Calculates Personalized PageRank (PPR) over the graph state visible at a specific MVCC transaction sequence number (`as_of_tx`).
///
/// # Performance Considerations
/// Constructs a snapshot-filtered, compacted `GraphInner` instance in $O(|V| + |E|)$ time by applying
/// [`is_edge_visible_bitemporal`] filtering over both compacted CSR edges and pending delta buffer edges,
/// as well as active hyperedges. This snapshot isolation trade-off is documented in BENCHMARK_PROTOCOL §6.
///
/// # Invariants
/// - Deterministic, panic-free execution.
/// - Does NOT mutate the underlying `inner` graph state.
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
        let mut visible_edges = Vec::new();

        // 1. Compacted CSR edges
        if i < inner.offsets.len() - 1 {
            let start = inner.offsets[i];
            let end = inner.offsets[i + 1];
            for edge_idx in start..end {
                let target = inner.targets[edge_idx];
                if inner.tombstoned_edges.contains(&(i, target)) {
                    continue;
                }
                let tx_vf = inner.tx_valid_from_at(edge_idx);
                let tx_vt = inner.tx_valid_to_at(edge_idx);
                let biz_vf = inner.business_valid_from_at(edge_idx);
                let biz_vt = inner.business_valid_to_at(edge_idx);

                if is_edge_visible_bitemporal(tx_vf, tx_vt, as_of_tx, biz_vf, biz_vt, None) {
                    visible_edges.push(EdgePayload {
                        target,
                        weight: inner.weights[edge_idx],
                        tx_valid_from: tx_vf,
                        tx_valid_to: tx_vt,
                        business_valid_from: biz_vf,
                        business_valid_to: biz_vt,
                        source_doc_id: inner.source_doc_id_at(edge_idx),
                    });
                }
            }
        }

        // 2. Uncompacted delta buffer edges
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

        if !visible_edges.is_empty() {
            pending_count += visible_edges.len();
            snapshot_pending.insert(i, visible_edges);
        }
    }

    snapshot.pending_edges = snapshot_pending;
    snapshot.pending_edge_count = pending_count;

    // 3. Filter Hyperedges visible as of as_of_tx
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

    // Rebuild CSR structures & compute out_weight_sums
    snapshot.compact();

    let mut ctx = PprContext::new();
    compute_ppr_with_context(&snapshot, seed_nodes, config, deleted_nodes, &mut ctx)
}
