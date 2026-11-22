pub mod mutations;
pub mod traversal;

use std::sync::atomic::Ordering;

use contextra_ports::{BoxFuture, GraphIndex, GraphIndexStats};
use contextra_types::{ContextraError, Entity, EntityId, PprConfig, Result, TxId};

use super::graph_write::CsrGraph;
use super::inner::GraphInner;
use super::types::MAX_TRAVERSAL_HOPS;
use super::visibility::is_suspicious_tx_id;

pub(crate) fn collect_entity_incident_edges(
    inner: &GraphInner,
    idx: usize,
) -> (Vec<EntityId>, Vec<EntityId>) {
    let mut outgoing = Vec::new();
    let mut incoming = Vec::new();

    if idx < inner.offsets.len() - 1 {
        for j in inner.offsets[idx]..inner.offsets[idx + 1] {
            let t_idx = inner.targets[j];
            if !inner.tombstoned_edges.contains(&(idx, t_idx)) {
                if let Some(&t_id) = inner.reverse_map.get(t_idx) {
                    outgoing.push(t_id);
                }
            }
        }
    }
    if let Some(pending) = inner.pending_edges.get(&idx) {
        for edge in pending {
            if !inner.tombstoned_edges.contains(&(idx, edge.target)) {
                if let Some(&t_id) = inner.reverse_map.get(edge.target) {
                    outgoing.push(t_id);
                }
            }
        }
    }

    let num_nodes = inner.reverse_map.len();
    for node_idx in 0..num_nodes {
        let source_id = match inner.reverse_map.get(node_idx) {
            Some(&id) => id,
            None => continue,
        };

        if node_idx < inner.offsets.len() - 1 {
            for j in inner.offsets[node_idx]..inner.offsets[node_idx + 1] {
                if inner.targets[j] == idx && !inner.tombstoned_edges.contains(&(node_idx, idx)) {
                    incoming.push(source_id);
                }
            }
        }
        if let Some(pending) = inner.pending_edges.get(&node_idx) {
            for edge in pending {
                if edge.target == idx && !inner.tombstoned_edges.contains(&(node_idx, idx)) {
                    incoming.push(source_id);
                }
            }
        }
    }

    (outgoing, incoming)
}

pub(crate) fn validate_traversal_params(max_hops: usize) -> Result<()> {
    if max_hops > 100 {
        return Err(ContextraError::InvalidInput(format!(
            "max_hops {max_hops} exceeds upper safety limit of 100"
        )));
    }
    if max_hops > MAX_TRAVERSAL_HOPS as usize {
        tracing::warn!(
            requested_max_hops = max_hops,
            effective_max_hops = MAX_TRAVERSAL_HOPS,
            "traversal requested max_hops ({max_hops}) exceeds internal cap MAX_TRAVERSAL_HOPS ({MAX_TRAVERSAL_HOPS}); capping depth"
        );
    }
    Ok(())
}

impl Default for CsrGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphIndex for CsrGraph {
    fn add_entity<'a>(&'a self, tx: TxId, entity: Entity) -> BoxFuture<'a, Result<()>> {
        mutations::add_entity_impl(self, tx, entity)
    }

    fn add_edge<'a>(&'a self, tx: TxId, edge: contextra_types::Edge) -> BoxFuture<'a, Result<()>> {
        mutations::add_edge_impl(self, tx, edge)
    }

    fn personalized_page_rank<'a>(
        &'a self,
        seed_nodes: &'a [EntityId],
        config: &'a PprConfig,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let deleted_view = self.deleted_view().await;
            if let Err(err) = self.compact_async().await {
                tracing::warn!(
                    error = %err,
                    "compact_async failed during personalized_page_rank compaction"
                );
            }
            let inner = self.inner_read();
            Ok(crate::ppr::compute_ppr(
                &inner,
                seed_nodes,
                config,
                &deleted_view,
            ))
        })
    }

    fn personalized_page_rank_at<'a>(
        &'a self,
        seed_nodes: &'a [EntityId],
        config: &'a PprConfig,
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let as_of_tx = TxId::new(seq_no);
            if is_suspicious_tx_id(as_of_tx) {
                tracing::warn!(
                    tx_id = seq_no,
                    hint = if as_of_tx == TxId::INVALID { "Sentinel TxId(0)" } else { "Wall-Clock-ns-Bereich" },
                    "AGT-GRAPH-001: Verdächtiger oder unallozierter seq_no in personalized_page_rank_at"
                );
            }
            let deleted_view = self.deleted_view().await;
            if let Err(err) = self.compact_async().await {
                tracing::warn!(
                    error = %err,
                    "compact_async failed during personalized_page_rank_at compaction"
                );
            }
            let inner = self.inner_read();
            Ok(crate::ppr::snapshot::compute_ppr_at(
                &inner,
                seed_nodes,
                config,
                &deleted_view,
                as_of_tx,
            ))
        })
    }

    fn path_rag_at<'a>(
        &'a self,
        start_nodes: &'a [EntityId],
        max_hops: usize,
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        self.path_rag_at(start_nodes, max_hops, seq_no)
    }

    fn traverse_at<'a>(
        &'a self,
        start_node: EntityId,
        max_hops: usize,
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            self.traverse_at_time(start_node, max_hops, TxId::new(seq_no))
                .await
        })
    }

    fn traverse_at_time<'a>(
        &'a self,
        start: EntityId,
        max_hops: usize,
        as_of: TxId,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            self.traverse_at_bitemporal(start, max_hops, as_of, None)
                .await
        })
    }

    fn traverse_at_bitemporal<'a>(
        &'a self,
        start: EntityId,
        max_hops: usize,
        as_of_tx: TxId,
        as_of_business: Option<i64>,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        traversal::traverse_at_bitemporal_impl(self, start, max_hops, as_of_tx, as_of_business)
    }

    fn traverse<'a>(
        &'a self,
        start: EntityId,
        max_hops: usize,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        traversal::traverse_impl(self, start, max_hops)
    }

    fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>> {
        mutations::commit_impl(self, tx)
    }

    fn remove_edge<'a>(
        &'a self,
        tx: TxId,
        from: EntityId,
        to: EntityId,
    ) -> BoxFuture<'a, Result<()>> {
        mutations::remove_edge_impl(self, tx, from, to)
    }

    fn add_bidirectional<'a>(
        &'a self,
        tx: TxId,
        from: EntityId,
        to: EntityId,
        label: &'a str,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.add_edge(tx, contextra_types::Edge::new(from, to, label))
                .await?;
            self.add_edge(tx, contextra_types::Edge::new(to, from, label))
                .await?;
            Ok(())
        })
    }

    fn neighbors<'a>(&'a self, start: EntityId) -> BoxFuture<'a, Result<Vec<EntityId>>> {
        Box::pin(async move { self.neighbors(start).await })
    }

    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut inner = self.inner_write();
            inner.staged_entities.retain(|&(t, _), _| t != tx);
            inner.staged_edges.retain(|&(t, _), _| t != tx);
            inner.staged_removals.remove(&tx);
            Ok(())
        })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(self.last_tx_id.load(Ordering::SeqCst))) })
    }

    fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
        Box::pin(async move { self.entity_count() })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>> {
        Box::pin(async move {
            let inner = self.inner_read();
            let num_entities = inner
                .entities
                .iter()
                .filter(|e| e.id != EntityId::new(0))
                .count();
            let num_edges = inner.targets.len()
                + inner.pending_edge_count
                + inner.staged_edges.values().map(|v| v.len()).sum::<usize>();

            let mem = (inner.reverse_map.len() * std::mem::size_of::<EntityId>())
                + (inner.entities.len() * std::mem::size_of::<Entity>())
                + (inner.offsets.len() * std::mem::size_of::<usize>())
                + (inner.targets.len() * std::mem::size_of::<usize>())
                + (inner.weights.len() * std::mem::size_of::<f32>());

            Ok(GraphIndexStats {
                num_entities,
                num_edges,
                memory_usage_bytes: mem,
            })
        })
    }
}
