use super::super::graph_write::CsrGraph;
use contextra_types::EntityId;
use std::collections::{HashMap, HashSet};

impl CsrGraph {
    pub async fn pagerank(
        &self,
        damping_factor: f32,
        max_iterations: usize,
        tolerance: f32,
    ) -> HashMap<EntityId, f32> {
        if let Err(err) = self.compact_async().await {
            tracing::warn!(error = %err, "compact_async failed during pagerank compaction");
        }
        let inner = self.inner_read();
        let n = inner.reverse_map.len();
        if n == 0 {
            return HashMap::new();
        }

        let mut ranks = vec![1.0 / (n as f32); n];
        let d = damping_factor;

        let mut out_degree = vec![0usize; n];
        for (i, deg) in out_degree.iter_mut().enumerate().take(n) {
            if i < inner.offsets.len() - 1 {
                let s = inner.offsets.get(i).copied().unwrap_or(0);
                let e = inner.offsets.get(i + 1).copied().unwrap_or(0);
                *deg = e.saturating_sub(s);
            }
        }

        for _iter in 0..max_iterations {
            let mut next_ranks = vec![(1.0 - d) / (n as f32); n];

            let dangling_sum: f32 = (0..n)
                .filter(|&i| out_degree.get(i).copied() == Some(0))
                .filter_map(|i| ranks.get(i).copied())
                .sum();
            let dangling_contrib = d * dangling_sum / (n as f32);
            for r in &mut next_ranks {
                *r += dangling_contrib;
            }

            for i in 0..n {
                let deg = out_degree.get(i).copied().unwrap_or(0);
                if deg > 0 {
                    let r_i = ranks.get(i).copied().unwrap_or(0.0);
                    let share = d * r_i / (deg as f32);
                    let start = inner.offsets.get(i).copied().unwrap_or(0);
                    let end = inner.offsets.get(i + 1).copied().unwrap_or(0);
                    for edge_idx in start..end {
                        if let Some(&target) = inner.targets.get(edge_idx) {
                            if let Some(slot) = next_ranks.get_mut(target) {
                                *slot += share;
                            }
                        }
                    }
                }
            }

            let diff: f32 = ranks
                .iter()
                .zip(next_ranks.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();

            ranks = next_ranks;

            if diff < tolerance {
                break;
            }
        }

        let mut result = HashMap::new();
        for (idx, &rank) in ranks.iter().enumerate() {
            if inner.entity_at(idx).is_some() {
                if let Some(&id) = inner.reverse_map.get(idx) {
                    result.insert(id, rank);
                }
            }
        }
        result
    }

    pub async fn get_deleted_node_indices(&self) -> HashSet<usize> {
        if self.storage.is_none() {
            return HashSet::new();
        }
        let entity_ids: Vec<(usize, EntityId)> = {
            let inner = self.inner_read();
            inner.reverse_map.iter().copied().enumerate().collect()
        };
        let mut deleted_indices = HashSet::new();
        for (idx, entity_id) in entity_ids {
            if self.is_entity_deleted(entity_id).await {
                deleted_indices.insert(idx);
            }
        }
        deleted_indices
    }

    pub async fn deleted_view(&self) -> crate::DeletedView {
        crate::DeletedView::from_nodes(self.get_deleted_node_indices().await)
    }

    pub async fn personalized_page_rank_with_context_async(
        &self,
        seed_nodes: &[EntityId],
        config: &contextra_types::PprConfig,
        ctx: &mut crate::PprContext,
    ) -> Vec<(EntityId, f32)> {
        let deleted_view = self.deleted_view().await;
        if let Err(err) = self.compact_async().await {
            tracing::warn!(
                error = %err,
                "compact_async failed during personalized_page_rank_with_context_async compaction"
            );
        }
        let inner = self.inner_read();
        crate::ppr::compute_ppr_with_context(&inner, seed_nodes, config, &deleted_view, ctx)
    }
}
