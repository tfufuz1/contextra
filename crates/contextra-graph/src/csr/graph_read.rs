use std::collections::{HashMap, HashSet};

use crate::GraphIndexExt;
use contextra_types::{EntityId, Result, TxId};

use super::graph_write::CsrGraph;
use super::types::GRAPH_COMMUNITY_PREFIX;

impl CsrGraph {
    /// Force compacts the graph delta buffer into the main CSR arrays to optimize traversal layout.
    pub fn compact(&self) {
        // Double-checked locking to avoid unnecessary write locks (FIND-GRA-002)
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

    /// Asynchronously compacts the graph delta buffer.
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
                // AI-TAG[RESOLVED][IP-08-BUDGET-COUPLING](TS:2026-09-18T12:00:00Z)(SESSION:e095d708): Connected to global ResourceTracker when cross-crate tracker handle is configured.
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
                // NOTE(IP-20): Hyperedge memory contributions are included in estimate_memory_bytes() for accurate local budget checks.
                return Ok(());
            }
        }

        self.compact();
        Ok(())
    }

    /// Sets community assignments in batch for in-memory graph index lookups.
    pub fn set_communities_batch(&self, assignments: &[crate::CommunityAssignment]) {
        let mut inner = self.inner_write();
        for a in assignments {
            inner.communities.insert(a.entity_id, a.community_id);
        }
        inner.communities_loaded = true;
    }

    /// Retrieves community assignments for a batch of entity IDs in a single operation.
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

    /// Returns direct 1-hop outgoing neighbors of `start`.
    pub async fn neighbors(&self, start: EntityId) -> Result<Vec<EntityId>> {
        let inner = self.inner_read();
        let start_idx = match inner.id_map.get(&start) {
            Some(&idx) => idx,
            None => return Ok(Vec::new()),
        };
        if inner.entity_at(start_idx).is_none() {
            return Ok(Vec::new());
        }

        // HashSet für O(1)-Dedup statt O(k) Vec::contains
        let mut seen: std::collections::HashSet<EntityId> = std::collections::HashSet::new();
        let mut neighbors: Vec<EntityId> = Vec::new();

        // Helper-Closure für deduped Insert:
        let mut push_if_new = |id: EntityId| {
            if seen.insert(id) {
                neighbors.push(id);
            }
        };

        // 1. CSR targets
        if start_idx < inner.offsets.len() - 1 {
            for edge_idx in inner.offsets[start_idx]..inner.offsets[start_idx + 1] {
                let neighbor_idx = inner.targets[edge_idx];
                if !inner.tombstoned_edges.contains(&(start_idx, neighbor_idx))
                    && inner.entity_at(neighbor_idx).is_some()
                {
                    if let Some(&id) = inner.reverse_map.get(neighbor_idx) {
                        push_if_new(id);
                    }
                }
            }
        }
        // 2. Pending edges
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

    /// Calculates PageRank for all entities in the graph using the CSR layout.
    pub async fn pagerank(
        &self,
        damping_factor: f32,
        max_iterations: usize,
        tolerance: f32,
    ) -> HashMap<EntityId, f32> {
        // Befund 2.1: compact_async() offloads CPU work if compaction is needed.
        // NOTE: This is an intermediate mitigation preventing Tokio runtime stalls during O(V+E) rebuilds;
        // exclusive write-lock scoping remains open for IP-08.
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

        // Out-degree per node
        let mut out_degree = vec![0usize; n];
        for (i, deg) in out_degree.iter_mut().enumerate().take(n) {
            if i < inner.offsets.len() - 1 {
                *deg = inner.offsets[i + 1] - inner.offsets[i];
            }
        }

        for _iter in 0..max_iterations {
            let mut next_ranks = vec![(1.0 - d) / (n as f32); n];

            // Account for dangling nodes (out_degree == 0)
            let dangling_sum: f32 = (0..n)
                .filter(|&i| out_degree[i] == 0)
                .map(|i| ranks[i])
                .sum();
            let dangling_contrib = d * dangling_sum / (n as f32);
            for r in &mut next_ranks {
                *r += dangling_contrib;
            }

            // Distribute rank across outgoing edges
            for i in 0..n {
                let deg = out_degree[i];
                if deg > 0 {
                    let share = d * ranks[i] / (deg as f32);
                    let start = inner.offsets[i];
                    let end = inner.offsets[i + 1];
                    for edge_idx in start..end {
                        let target = inner.targets[edge_idx];
                        next_ranks[target] += share;
                    }
                }
            }

            // Check convergence
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

    /// Collects internal node indices of entities that are marked as deleted in storage.
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

    /// Loads tombstone status of all graph nodes and constructs an authoritative [`crate::DeletedView`].
    pub async fn deleted_view(&self) -> crate::DeletedView {
        crate::DeletedView::from_nodes(self.get_deleted_node_indices().await)
    }

    /// Calculates Personalized PageRank (PPR) using a reusable [`crate::PprContext`] buffer to avoid allocations.
    pub async fn personalized_page_rank_with_context_async(
        &self,
        seed_nodes: &[EntityId],
        config: &contextra_types::PprConfig,
        ctx: &mut crate::PprContext,
    ) -> Vec<(EntityId, f32)> {
        let deleted_view = self.deleted_view().await;
        // Befund 2.1: compact_async() offloads CPU work if compaction is needed.
        // NOTE: This is an intermediate mitigation preventing Tokio runtime stalls during O(V+E) rebuilds;
        // exclusive write-lock scoping remains open for IP-08.
        if let Err(err) = self.compact_async().await {
            tracing::warn!(
                error = %err,
                "compact_async failed during personalized_page_rank_with_context_async compaction"
            );
        }
        let inner = self.inner_read();
        crate::ppr::compute_ppr_with_context(&inner, seed_nodes, config, &deleted_view, ctx)
    }

    /// Returns the number of committed entities in the graph.
    pub fn entity_count(&self) -> usize {
        self.inner_read()
            .entities
            .iter()
            .filter(|e| e.id != EntityId::new(0))
            .count()
    }

    /// Checks if a committed entity exists in the graph.
    pub fn entity_exists(&self, id: EntityId) -> bool {
        let inner = self.inner_read();
        if let Some(&idx) = inner.id_map.get(&id) {
            inner.entity_at(idx).is_some()
        } else {
            false
        }
    }

    /// Prüft ob eine Entity per Tombstone als gelöscht markiert wurde.
    /// Nutzt den Key "graph:entity:deleted:{entity_id}" im LSM-Storage (wenn vorhanden).
    pub async fn is_entity_deleted(&self, entity: EntityId) -> bool {
        if let Some(storage) = &self.storage {
            let key = format!("graph:entity:deleted:{}", entity.0);
            return storage.get(key.as_bytes()).await.ok().flatten().is_some();
        }
        false
    }

    /// Returns the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        let inner = self.inner_read();
        inner.targets.len()
            + inner.pending_edge_count
            + inner.staged_edges.values().map(|v| v.len()).sum::<usize>()
    }

    /// Removes an entity node and all its incident (outgoing and incoming) edges from the graph.
    pub async fn remove_entity(&self, tx: TxId, entity: EntityId) -> Result<()> {
        GraphIndexExt::remove_entity(self, tx, entity).await
    }
}
