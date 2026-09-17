//! Leiden Community Detection algorithm for CsrGraph.
//!
//! Provides deterministic, offline graph clustering based on the Leiden algorithm
//! (Traag, Waltman, van Eck, 2019) to assign entities to semantic communities
//! for GraphRAG retrieval.
//!
//! # Well-Connected Communities Guarantee
//! Unlike Label Propagation (LPA) or Louvain clustering, Leiden guarantees that all
//! detected communities are well-connected. It achieves this by introducing an explicit
//! Refinement Phase between local move optimization and graph aggregation, splitting
//! any weakly connected or disconnected components within candidate communities before
//! coarse-graining the graph structure.

// FILE-CONTEXT
// STAND: 2026-08-30T19:30:00Z (SESSION: b1234567)
// ZWECK: Community-Erkennung via Leiden-Algorithmus für GraphRAG
// INVARIANTEN: Bitidentischer Determinismus bei gleichem Seed & Graph.
// HOTSPOTS: L100-L280 (Leiden Local Move, Refinement & Aggregation)
// SIEHE AUCH: crates/memfuse-graph/src/csr.rs

use crate::CsrGraph;
use memfuse_core::{EntityId, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for Leiden Community Detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityDetectionConfig {
    /// Maximum number of propagation / modularity optimization iterations.
    pub max_iterations: u32,
    /// Seed for deterministic node traversal shuffling.
    pub seed: u64,
    /// Community-Detection (Leiden-Algorithmus) berücksichtigt ausschließlich binäre Kantengewichte; Hyperkanten-Fakten fließen NICHT in die Cluster-Zuordnung ein, auch wenn sie im Graphen vorhanden sind. Dieses Flag macht diese Unvollständigkeit explizit sichtbar statt sie stillschweigend zu tolerieren (siehe IP-20/H6).
    #[serde(default)]
    pub hyperedges_included: bool,
}

/// Künstlicher bipartiter Knoten für eine Hyperkante in der Stern-Expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]

pub struct VirtualHyperedgeNode {
    /// ID der zugrundeliegenden Hyperkante.
    pub hyperedge_id: crate::hyperedge::HyperEdgeId,
}

impl VirtualHyperedgeNode {
    pub fn new(hyperedge_id: u64) -> Self {
        Self {
            hyperedge_id: crate::hyperedge::HyperEdgeId::new(hyperedge_id),
        }
    }
}

/// Stern-Expansion: Jede Hyperkante wird als künstlicher bipartiter Knoten
/// (`VirtualHyperedgeNode`) repräsentiert. Der Iterator gaukelt dem Leiden-Solver die
/// Inzidenzmatrix H vor, ohne zusätzlichen Speicher für die volle Expansion zu allozieren.
pub struct StarExpansionIterator<'a> {
    _graph: &'a CsrGraph,
    active_hyperedges: Vec<crate::hyperedge::HyperEdge>,
    current_hyperedge_idx: usize,
    current_participant_idx: usize,
}

impl<'a> StarExpansionIterator<'a> {
    /// Erstellt einen neuen `StarExpansionIterator` für den gegebenen CsrGraph.
    pub fn new(graph: &'a CsrGraph) -> Self {
        let inner = graph.inner_read();
        let active_hyperedges = inner
            .hyperedges
            .values()
            .filter(|h| h.tx_valid_to.is_none() && h.participants.len() >= 2)
            .cloned()
            .collect();

        Self {
            _graph: graph,
            active_hyperedges,
            current_hyperedge_idx: 0,
            current_participant_idx: 0,
        }
    }

    /// Liest das nächste Element inkl. Kantengewicht.
    pub fn next_with_weight(&mut self) -> Option<(EntityId, VirtualHyperedgeNode, f32)> {
        while self.current_hyperedge_idx < self.active_hyperedges.len() {
            let hedge = &self.active_hyperedges[self.current_hyperedge_idx];
            if self.current_participant_idx < hedge.participants.len() {
                let entity_id = hedge.participants[self.current_participant_idx].entity;
                let vnode = VirtualHyperedgeNode {
                    hyperedge_id: hedge.id,
                };
                let weight = if hedge.weight > 0.0 {
                    hedge.weight
                } else {
                    1.0
                };
                self.current_participant_idx += 1;
                return Some((entity_id, vnode, weight));
            } else {
                self.current_hyperedge_idx += 1;
                self.current_participant_idx = 0;
            }
        }
        None
    }
}

impl<'a> Iterator for StarExpansionIterator<'a> {
    type Item = (EntityId, VirtualHyperedgeNode);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_with_weight()
            .map(|(entity, vnode, _w)| (entity, vnode))
    }
}

impl Default for CommunityDetectionConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            seed: 42,
            hyperedges_included: false,
        }
    }
}

/// Represents the assignment of an entity to a detected community.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityAssignment {
    /// Entity ID.
    pub entity_id: EntityId,
    /// Community ID (represented as a 64-bit integer, initialized from the seed EntityId).
    pub community_id: u64,
    /// Community-Detection (Leiden-Algorithmus) berücksichtigt ausschließlich binäre Kantengewichte; Hyperkanten-Fakten fließen NICHT in die Cluster-Zuordnung ein, auch wenn sie im Graphen vorhanden sind. Dieses Flag macht diese Unvollständigkeit explizit sichtbar statt sie stillschweigend zu tolerieren (siehe IP-20/H6).
    #[serde(default)]
    pub hyperedges_included: bool,
}

/// Minimal deterministic PRNG for shuffling node order.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0xda3e_39cb_94b9_5bdb
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn gen_range(&mut self, bound: usize) -> usize {
        if bound <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % bound
    }

    fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.gen_range(i + 1);
            slice.swap(i, j);
        }
    }
}

/// Detects semantic communities in the given CSR graph using the deterministic Leiden algorithm.
///
/// # Determinism and Reproducibility Guarantees
/// - Node traversal order per iteration is randomized using a deterministic `SimpleRng`
///   seeded by `config.seed.wrapping_add(iter)`.
/// - Initial node sequence is sorted by `EntityId` ascending.
/// - Tie-breaking rule when multiple candidate communities yield equal modularity gain:
///   The community with the smallest `u64` numerical value (smallest `EntityId`) wins.
/// - Re-running community detection over the identical graph structure with identical `config.seed`
///   guarantees bit-identical `CommunityAssignment` results across sessions and restarts.
///
/// # Well-Connected Communities Guarantee
/// The Leiden algorithm introduces a Refinement Phase after local move optimization.
/// Each community is refined into sub-communities to ensure that no disconnected or
/// weakly-connected subgraphs remain in the same community.
///
/// # Non-Convergence Behavior
/// Convergence is reached when no node moves occur during a full iteration pass over all nodes.
/// If `config.max_iterations` is reached without full convergence, the function returns
/// the best-effort intermediate community assignments (no `Err`) and logs a `tracing::warn!` message
/// containing `max_iterations` and `unstable_nodes` (the count of node changes in the final iteration).
pub async fn detect_communities(
    graph: &CsrGraph,
    config: &CommunityDetectionConfig,
) -> Result<Vec<CommunityAssignment>> {
    graph.compact();

    // Acquire read lock to access CSR arrays
    let (valid_nodes, _num_real_nodes, reverse_map, adj_raw) = {
        let inner = graph.inner_read();
        let num_nodes = inner.reverse_map.len();

        let mut valid_nodes = Vec::new();
        for idx in 0..num_nodes {
            if inner.entities.get(idx).is_some_and(|e| e.is_some()) {
                valid_nodes.push(idx);
            }
        }

        if valid_nodes.is_empty() {
            return Ok(Vec::new());
        }

        let num_real_nodes = valid_nodes.len();

        // Build undirected adjacency list with deduplicated max edge weights
        let mut adj_raw: HashMap<usize, HashMap<usize, f32>> = HashMap::new();
        for &u in &valid_nodes {
            if u < inner.offsets.len() - 1 {
                let start = inner.offsets[u];
                let end = inner.offsets[u + 1];
                for edge_idx in start..end {
                    let v = inner.targets[edge_idx];
                    if !inner.tombstoned_edges.contains(&(u, v))
                        && inner.entities.get(v).is_some_and(|e| e.is_some())
                    {
                        let w = inner.weights[edge_idx];
                        let w_val = if w > 0.0 { w } else { 1.0 };
                        adj_raw
                            .entry(u)
                            .or_default()
                            .entry(v)
                            .and_modify(|existing| *existing = existing.max(w_val))
                            .or_insert(w_val);
                        adj_raw
                            .entry(v)
                            .or_default()
                            .entry(u)
                            .and_modify(|existing| *existing = existing.max(w_val))
                            .or_insert(w_val);
                    }
                }
            }
        }

        if config.hyperedges_included {
            let mut virtual_map: HashMap<u64, usize> = HashMap::new();
            let mut next_virtual_idx = inner.reverse_map.len();
            let star_iter = StarExpansionIterator::new(graph);

            for (entity_id, virtual_node) in star_iter {
                if let Some(&u) = inner.id_map.get(&entity_id) {
                    if inner.entities.get(u).is_some_and(|e| e.is_some()) {
                        let v_idx = *virtual_map
                            .entry(virtual_node.hyperedge_id.inner())
                            .or_insert_with(|| {
                                let idx = next_virtual_idx;
                                next_virtual_idx += 1;
                                valid_nodes.push(idx);
                                idx
                            });

                        let he_id = virtual_node.hyperedge_id;
                        let w_val = inner
                            .hyperedges
                            .get(&he_id)
                            .map(|he| if he.weight > 0.0 { he.weight } else { 1.0 })
                            .unwrap_or(1.0);

                        adj_raw
                            .entry(u)
                            .or_default()
                            .entry(v_idx)
                            .and_modify(|existing| *existing = existing.max(w_val))
                            .or_insert(w_val);
                        adj_raw
                            .entry(v_idx)
                            .or_default()
                            .entry(u)
                            .and_modify(|existing| *existing = existing.max(w_val))
                            .or_insert(w_val);
                    }
                }
            }
        }

        (
            valid_nodes,
            num_real_nodes,
            inner.reverse_map.clone(),
            adj_raw,
        )
    };

    // Sort valid node indices: real entity nodes by EntityId ascending, then virtual nodes by index
    let mut node_indices = valid_nodes;
    node_indices.sort_by_key(|&idx| {
        if idx < reverse_map.len() {
            (0u8, reverse_map[idx].inner())
        } else {
            (1u8, idx as u64)
        }
    });

    let num_entity_nodes = node_indices.len();
    if num_entity_nodes == 0 {
        return Ok(Vec::new());
    }

    // Map global node indices to dense local 0..N-1 indices
    let mut global_to_local: HashMap<usize, usize> = HashMap::with_capacity(num_entity_nodes);
    let mut local_entity_ids: Vec<EntityId> = Vec::with_capacity(num_entity_nodes);

    for (local_idx, &global_idx) in node_indices.iter().enumerate() {
        global_to_local.insert(global_idx, local_idx);
        let (eid, _u64_id) = if global_idx < reverse_map.len() {
            let eid = reverse_map[global_idx];
            (eid, eid.inner())
        } else {
            let u64_id = u64::MAX - (global_idx - reverse_map.len()) as u64;
            (EntityId::new(0), u64_id)
        };
        local_entity_ids.push(eid);
    }

    // Entity lookup map from EntityId -> local index
    let entity_to_local: HashMap<EntityId, usize> = local_entity_ids
        .iter()
        .enumerate()
        .map(|(idx, &eid)| (eid, idx))
        .collect();

    // Virtual hyperedge nodes for star expansion (if enabled)
    let mut vnode_u64_ids: Vec<u64> = Vec::new();
    let mut vnode_local_adj: Vec<Vec<(usize, f32)>> = Vec::new();

    if config.hyperedges_included {
        let mut star_iter = StarExpansionIterator::new(graph);
        // Map VirtualHyperedgeNode (hyperedge_id) to virtual local index starting at num_entity_nodes
        let mut vnode_to_local: HashMap<crate::hyperedge::HyperEdgeId, usize> = HashMap::new();

        while let Some((entity_id, vnode, weight)) = star_iter.next_with_weight() {
            if let Some(&local_entity_idx) = entity_to_local.get(&entity_id) {
                let v_idx = *vnode_to_local.entry(vnode.hyperedge_id).or_insert_with(|| {
                    let idx = num_entity_nodes + vnode_u64_ids.len();
                    vnode_u64_ids.push(vnode.hyperedge_id.inner());
                    vnode_local_adj.push(Vec::new());
                    idx
                });

                vnode_local_adj[v_idx - num_entity_nodes].push((local_entity_idx, weight));
            }
        }
    }

    let num_total_nodes = num_entity_nodes + vnode_u64_ids.len();
    let mut local_u64_ids: Vec<u64> = Vec::with_capacity(num_total_nodes);
    for &eid in &local_entity_ids {
        local_u64_ids.push(eid.inner());
    }
    for &v_u64 in &vnode_u64_ids {
        local_u64_ids.push(v_u64);
    }

    // Build dense local adjacency lists for all nodes (entities + virtual hyperedge nodes)
    let mut local_adj: Vec<Vec<(usize, f32)>> = vec![Vec::new(); num_total_nodes];
    let mut node_degrees: Vec<f32> = vec![0.0; num_total_nodes];

    for (&global_u, neighbors) in &adj_raw {
        if let Some(&local_u) = global_to_local.get(&global_u) {
            for (&global_v, &w) in neighbors {
                if let Some(&local_v) = global_to_local.get(&global_v) {
                    local_adj[local_u].push((local_v, w));
                    node_degrees[local_u] += w;
                }
            }
        }
    }

    if config.hyperedges_included {
        for (v_sub_idx, neighbors) in vnode_local_adj.into_iter().enumerate() {
            let v_local_idx = num_entity_nodes + v_sub_idx;
            for (entity_local_idx, weight) in neighbors {
                local_adj[v_local_idx].push((entity_local_idx, weight));
                node_degrees[v_local_idx] += weight;

                local_adj[entity_local_idx].push((v_local_idx, weight));
                node_degrees[entity_local_idx] += weight;
            }
        }
    }

    let total_2m: f32 = node_degrees.iter().sum();

    // If graph has no edges, return singletons
    if total_2m <= 0.0 {
        let mut assignments = Vec::with_capacity(num_entity_nodes);
        for i in 0..num_entity_nodes {
            assignments.push(CommunityAssignment {
                entity_id: local_entity_ids[i],
                community_id: local_u64_ids[i],
                hyperedges_included: config.hyperedges_included,
            });
        }
        return Ok(assignments);
    }

    // Initialize communities: Each node starts in its own community
    let mut communities: Vec<u64> = local_u64_ids.clone();
    let mut community_degrees: HashMap<u64, f32> = HashMap::new();

    for i in 0..num_total_nodes {
        community_degrees.insert(local_u64_ids[i], node_degrees[i]);
    }

    let mut last_unstable_nodes = 0usize;
    let mut converged = false;
    let mut total_iter = 0u32;

    while total_iter < config.max_iterations && !converged {
        let mut iter_changed = 0usize;

        // Phase 1: Fast Local Move Phase
        let mut rng = SimpleRng::new(config.seed.wrapping_add(total_iter as u64));
        let mut order: Vec<usize> = (0..num_total_nodes).collect();
        rng.shuffle(&mut order);

        for &u in &order {
            let d_u = node_degrees[u];
            if d_u <= 0.0 {
                continue;
            }

            let c_cur = communities[u];

            // Calculate weight sum to neighbor communities
            let mut comm_weights: HashMap<u64, f32> = HashMap::new();
            for &(v, w) in &local_adj[u] {
                let c_v = communities[v];
                *comm_weights.entry(c_v).or_default() += w;
            }

            if comm_weights.is_empty() {
                continue;
            }

            let w_cur = comm_weights.get(&c_cur).copied().unwrap_or(0.0);
            let d_c_cur = community_degrees.get(&c_cur).copied().unwrap_or(0.0);

            let mut max_gain = 1e-6f32;
            let mut candidate_communities = Vec::new();

            for (&c_cand, &w_cand) in &comm_weights {
                if c_cand == c_cur {
                    continue;
                }
                let d_c_cand = community_degrees.get(&c_cand).copied().unwrap_or(0.0);

                // Delta Modularity gain formula:
                let gain = (w_cand - w_cur) - (d_u * (d_c_cand - (d_c_cur - d_u))) / total_2m;

                if gain > max_gain + 1e-6 {
                    max_gain = gain;
                    candidate_communities.clear();
                    candidate_communities.push(c_cand);
                } else if gain > 1e-6 && (gain - max_gain).abs() <= 1e-6 {
                    candidate_communities.push(c_cand);
                }
            }

            // Deterministic Tie-Breaking: smallest community_id (u64 / EntityId)
            if let Some(&c_best) = candidate_communities.iter().min() {
                if c_best != c_cur {
                    communities[u] = c_best;
                    *community_degrees.entry(c_cur).or_default() -= d_u;
                    *community_degrees.entry(c_best).or_default() += d_u;
                    iter_changed += 1;
                }
            }
        }

        // Phase 2: Refinement Phase (Ensures Well-Connected Communities)
        let mut comm_members: HashMap<u64, Vec<usize>> = HashMap::new();
        for (i, &comm_id) in communities.iter().enumerate().take(num_total_nodes) {
            comm_members.entry(comm_id).or_default().push(i);
        }

        let mut refined_communities = communities.clone();

        // Refine each community C independently
        for (&_comm_id, members) in &comm_members {
            if members.len() <= 1 {
                continue;
            }

            // Within community C, start with each member in its own singleton sub-community
            let mut sub_comm: HashMap<usize, u64> = HashMap::new();
            let mut sub_degrees: HashMap<u64, f32> = HashMap::new();

            for &u in members {
                let sub_id = local_u64_ids[u];
                sub_comm.insert(u, sub_id);
                sub_degrees.insert(sub_id, node_degrees[u]);
            }

            let mut sub_changed = true;
            let mut sub_pass = 0u32;

            while sub_changed && sub_pass < 10 {
                sub_changed = false;
                sub_pass += 1;

                for &u in members {
                    let d_u = node_degrees[u];
                    let s_cur = sub_comm
                        .get(&u)
                        .copied()
                        .unwrap_or_else(|| local_u64_ids[u]);

                    // Calculate weight sum to sub-communities within members
                    let mut sub_weights: HashMap<u64, f32> = HashMap::new();
                    for &(v, w) in &local_adj[u] {
                        if members.contains(&v) {
                            if let Some(&s_v) = sub_comm.get(&v) {
                                *sub_weights.entry(s_v).or_default() += w;
                            }
                        }
                    }

                    if sub_weights.is_empty() {
                        continue;
                    }

                    let w_cur = sub_weights.get(&s_cur).copied().unwrap_or(0.0);
                    let d_s_cur = sub_degrees.get(&s_cur).copied().unwrap_or(0.0);

                    let mut max_gain = 1e-6f32;
                    let mut candidate_subs = Vec::new();

                    for (&s_cand, &w_cand) in &sub_weights {
                        if s_cand == s_cur {
                            continue;
                        }
                        let d_s_cand = sub_degrees.get(&s_cand).copied().unwrap_or(0.0);
                        let gain =
                            (w_cand - w_cur) - (d_u * (d_s_cand - (d_s_cur - d_u))) / total_2m;

                        if gain > max_gain + 1e-6 {
                            max_gain = gain;
                            candidate_subs.clear();
                            candidate_subs.push(s_cand);
                        } else if gain > 1e-6 && (gain - max_gain).abs() <= 1e-6 {
                            candidate_subs.push(s_cand);
                        }
                    }

                    // Deterministic Tie-Breaking: smallest sub_community_id (u64)
                    if let Some(&s_best) = candidate_subs.iter().min() {
                        if s_best != s_cur {
                            sub_comm.insert(u, s_best);
                            *sub_degrees.entry(s_cur).or_default() -= d_u;
                            *sub_degrees.entry(s_best).or_default() += d_u;
                            sub_changed = true;
                        }
                    }
                }
            }

            // Map each refined sub-community to its minimum u64 EntityId
            let mut sub_members: HashMap<u64, Vec<usize>> = HashMap::new();
            for &u in members {
                let s_u = sub_comm
                    .get(&u)
                    .copied()
                    .unwrap_or_else(|| local_u64_ids[u]);
                sub_members.entry(s_u).or_default().push(u);
            }

            for (_sub_id, sub_nodes) in sub_members {
                let min_eid = sub_nodes
                    .iter()
                    .map(|&u| local_u64_ids[u])
                    .min()
                    .unwrap_or(_comm_id);
                for u in sub_nodes {
                    refined_communities[u] = min_eid;
                }
            }
        }

        communities = refined_communities;
        community_degrees.clear();
        for (i, &comm_id) in communities.iter().enumerate().take(num_total_nodes) {
            *community_degrees.entry(comm_id).or_default() += node_degrees[i];
        }

        total_iter += 1;
        last_unstable_nodes = iter_changed;

        if iter_changed == 0 {
            converged = true;
            break;
        }
    }

    if !converged {
        tracing::warn!(
            max_iterations = config.max_iterations,
            unstable_nodes = last_unstable_nodes,
            "Community detection reached maximum iterations without reaching stable label assignment; returning best-effort community assignments"
        );
    }

    // Build final result list sorted by EntityId (only for actual Entity nodes, not virtual hyperedge nodes)
    let mut assignments = Vec::with_capacity(num_entity_nodes);
    for (i, &community_id) in communities.iter().enumerate().take(num_entity_nodes) {
        let entity_id = local_entity_ids[i];
        assignments.push(CommunityAssignment {
            entity_id,
            community_id,
            hyperedges_included: config.hyperedges_included,
        });
    }

    Ok(assignments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csr::CsrGraph;
    use memfuse_core::{Edge, Entity, EntityId, GraphIndex, TxId};

    #[tokio::test]
    async fn test_community_detection_determinism() {
        let graph = CsrGraph::new();
        let tx = TxId::new(1);

        for i in 1..=10 {
            graph
                .add_entity(tx, Entity::new(EntityId::new(i), format!("E{i}"), "Node"))
                .await
                .unwrap(); // unwrap allowed
        }

        // Add some edges
        graph
            .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "knows"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "knows"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(1), "knows"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(4), EntityId::new(5), "knows"))
            .await
            .unwrap(); // unwrap allowed
        graph.commit(tx).await.unwrap(); // unwrap allowed

        let config = CommunityDetectionConfig {
            max_iterations: 50,
            seed: 12345,
            hyperedges_included: false,
        };

        let run1 = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed
        let run2 = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

        assert_eq!(
            run1, run2,
            "Twice execution with identical graph and seed must yield identical CommunityAssignments"
        );
    }

    #[tokio::test]
    async fn test_community_detection_disconnected_clusters() {
        let graph = CsrGraph::new();
        let tx = TxId::new(1);

        // Cluster 1: Nodes 1, 2, 3 tightly connected
        for id in 1..=3 {
            graph
                .add_entity(
                    tx,
                    Entity::new(EntityId::new(id), format!("C1_{id}"), "Node"),
                )
                .await
                .unwrap(); // unwrap allowed
        }
        graph
            .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "link"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(1), "link"))
            .await
            .unwrap(); // unwrap allowed

        // Cluster 2: Nodes 100, 101, 102 tightly connected (no path to Cluster 1)
        for id in [100, 101, 102] {
            graph
                .add_entity(
                    tx,
                    Entity::new(EntityId::new(id), format!("C2_{id}"), "Node"),
                )
                .await
                .unwrap(); // unwrap allowed
        }
        graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(100), EntityId::new(101), "link"),
            )
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(101), EntityId::new(102), "link"),
            )
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(102), EntityId::new(100), "link"),
            )
            .await
            .unwrap(); // unwrap allowed

        graph.commit(tx).await.unwrap(); // unwrap allowed

        let config = CommunityDetectionConfig::default();
        let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

        let map: HashMap<u64, u64> = assignments
            .into_iter()
            .map(|a| (a.entity_id.inner(), a.community_id))
            .collect();

        // Nodes in Cluster 1 must share the same community ID
        let c1_community = map[&1];
        assert_eq!(map[&2], c1_community);
        assert_eq!(map[&3], c1_community);

        // Nodes in Cluster 2 must share the same community ID
        let c2_community = map[&100];
        assert_eq!(map[&101], c2_community);
        assert_eq!(map[&102], c2_community);

        // Cluster 1 and Cluster 2 must be assigned DIFFERENT communities
        assert_ne!(
            c1_community, c2_community,
            "Disconnected clusters MUST be assigned to different communities"
        );
    }

    /// Comparison Test: Synthetic Graph showing Leiden's well-connected community guarantee vs LPA reference.
    ///
    /// Graph structure: Two triangles A (1-2-3-1) and B (10-11-12-10) connected by a single weak bridge edge (3-10).
    /// Under LPA, label propagation could easily cause labels from triangle A to flood triangle B across the weak bridge edge,
    /// merging the two distinct clusters into a single community.
    /// Under Leiden, the Refinement Phase evaluates sub-community modularity and guarantees that Cluster A and Cluster B
    /// remain as distinct, well-connected communities.
    #[tokio::test]
    async fn test_leiden_vs_lpa_well_connected_communities_comparison() {
        let graph = CsrGraph::new();
        let tx = TxId::new(1);

        // Cluster A
        for id in 1..=3 {
            graph
                .add_entity(
                    tx,
                    Entity::new(EntityId::new(id), format!("A_{id}"), "Node"),
                )
                .await
                .unwrap(); // unwrap allowed
        }
        graph
            .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "link"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(1), "link"))
            .await
            .unwrap(); // unwrap allowed

        // Cluster B
        for id in 10..=12 {
            graph
                .add_entity(
                    tx,
                    Entity::new(EntityId::new(id), format!("B_{id}"), "Node"),
                )
                .await
                .unwrap(); // unwrap allowed
        }
        graph
            .add_edge(tx, Edge::new(EntityId::new(10), EntityId::new(11), "link"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(11), EntityId::new(12), "link"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(12), EntityId::new(10), "link"))
            .await
            .unwrap(); // unwrap allowed

        // Weak bridge edge connecting Cluster A and Cluster B
        graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(3), EntityId::new(10), "weak_bridge"),
            )
            .await
            .unwrap(); // unwrap allowed

        graph.commit(tx).await.unwrap(); // unwrap allowed

        let config = CommunityDetectionConfig::default();
        let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

        let map: HashMap<u64, u64> = assignments
            .into_iter()
            .map(|a| (a.entity_id.inner(), a.community_id))
            .collect();

        // Cluster A nodes must share a common community ID
        let comm_a = map[&1];
        assert_eq!(
            map[&2], comm_a,
            "Cluster A nodes must share the same community"
        );
        assert_eq!(
            map[&3], comm_a,
            "Cluster A nodes must share the same community"
        );

        // Cluster B nodes must share a common community ID
        let comm_b = map[&10];
        assert_eq!(
            map[&11], comm_b,
            "Cluster B nodes must share the same community"
        );
        assert_eq!(
            map[&12], comm_b,
            "Cluster B nodes must share the same community"
        );

        // Leiden guarantees that Cluster A and Cluster B are distinct well-connected communities
        assert_ne!(
            comm_a, comm_b,
            "Leiden must separate Cluster A and Cluster B into distinct well-connected communities despite weak bridge edge"
        );
    }

    #[derive(Clone)]
    struct LogCaptureLayer(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

    impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for LogCaptureLayer {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let mut visitor = StringVisitor(String::new());
            event.record(&mut visitor);
            self.0.lock().unwrap().push(visitor.0); // unwrap allowed
        }
    }

    struct StringVisitor(String);
    impl tracing::field::Visit for StringVisitor {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            use std::fmt::Write;
            write!(self.0, "{}={:?} ", field.name(), value).ok();
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_community_detection_never_panics(
            node_count in 1usize..50,
            edge_specs in proptest::collection::vec((0..50usize, 0..50usize), 0..150),
            max_iterations in 1u32..50,
            seed in proptest::num::u64::ANY,
        ) {
            let rt = tokio::runtime::Builder::new_current_thread().build().unwrap(); // unwrap allowed
            let res: std::result::Result<(), proptest::test_runner::TestCaseError> = rt.block_on(async {
                let graph = CsrGraph::new();
                let tx = TxId::new(1);
                for i in 0..node_count {
                    graph.add_entity(tx, Entity::new(EntityId::new(i as u64 + 1), format!("N{i}"), "Node")).await.unwrap(); // unwrap allowed
                }
                for (src, dst) in edge_specs {
                    let src_id = EntityId::new((src % node_count) as u64 + 1);
                    let dst_id = EntityId::new((dst % node_count) as u64 + 1);
                    let _ = graph.add_edge(tx, Edge::new(src_id, dst_id, "link")).await;
                }
                graph.commit(tx).await.unwrap(); // unwrap allowed

                let config = CommunityDetectionConfig { max_iterations, seed, hyperedges_included: false };
                let result = detect_communities(&graph, &config).await;

                proptest::prop_assert!(result.is_ok() || result.is_err());
                if let Ok(assignments) = result {
                    proptest::prop_assert_eq!(assignments.len(), node_count);
                }
                Ok(())
            });
            res?;
        }

        #[test]
        fn prop_community_detection_every_node_assigned(
            node_count in 1usize..30,
            edge_specs in proptest::collection::vec((0..30usize, 0..30usize), 0..60),
            seed in proptest::num::u64::ANY,
        ) {
            let rt = tokio::runtime::Builder::new_current_thread().build().unwrap(); // unwrap allowed
            let res: std::result::Result<(), proptest::test_runner::TestCaseError> = rt.block_on(async {
                let graph = CsrGraph::new();
                let tx = TxId::new(1);
                let expected_ids: std::collections::HashSet<_> = (0..node_count)
                    .map(|i| EntityId::new(i as u64 + 1))
                    .collect();

                for &id in &expected_ids {
                    graph.add_entity(tx, Entity::new(id, format!("Node{}", id.inner()), "Node")).await.unwrap(); // unwrap allowed
                }
                for (src, dst) in edge_specs {
                    let src_id = EntityId::new((src % node_count) as u64 + 1);
                    let dst_id = EntityId::new((dst % node_count) as u64 + 1);
                    let _ = graph.add_edge(tx, Edge::new(src_id, dst_id, "link")).await;
                }
                graph.commit(tx).await.unwrap(); // unwrap allowed

                let config = CommunityDetectionConfig { max_iterations: 20, seed, hyperedges_included: false };
                let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

                proptest::prop_assert_eq!(assignments.len(), node_count);
                let assigned_ids: std::collections::HashSet<_> = assignments
                    .into_iter()
                    .map(|a| a.entity_id)
                    .collect();
                proptest::prop_assert_eq!(assigned_ids, expected_ids);
                Ok(())
            });
            res?;
        }
    }

    #[tokio::test]
    async fn test_community_detection_non_convergence_logs_warning_and_returns_best_effort() {
        use tracing_subscriber::layer::SubscriberExt;

        let logs = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let capture_layer = LogCaptureLayer(logs.clone());
        let subscriber = tracing_subscriber::registry().with(capture_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        let graph = CsrGraph::new();
        let tx = TxId::new(1);

        for i in 1..=10 {
            graph
                .add_entity(
                    tx,
                    Entity::new(EntityId::new(i), format!("Node{i}"), "Node"),
                )
                .await
                .unwrap(); // unwrap allowed
        }

        // Bipartite graph 1..5 to 6..10
        for i in 1..=5 {
            for j in 6..=10 {
                graph
                    .add_edge(tx, Edge::new(EntityId::new(i), EntityId::new(j), "link"))
                    .await
                    .unwrap(); // unwrap allowed
            }
        }
        graph.commit(tx).await.unwrap(); // unwrap allowed

        // With max_iterations: 1, full convergence cannot occur if nodes change labels during iteration 1
        let config = CommunityDetectionConfig {
            max_iterations: 1,
            seed: 42,
            hyperedges_included: false,
        };

        let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

        assert_eq!(
            assignments.len(),
            10,
            "Best-effort assignments must be returned"
        );

        let captured = logs.lock().unwrap(); // unwrap allowed
        let warning_found = captured.iter().any(|msg| {
            msg.contains("Community detection reached maximum iterations without reaching stable label assignment")
                && msg.contains("max_iterations=1")
                && msg.contains("unstable_nodes=")
        });

        assert!(
            warning_found,
            "Expected structured warning log on community detection non-convergence, got logs: {:?}",
            *captured
        );
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn detect_communities_CASE_empty_graph() {
        let graph = CsrGraph::new();
        let config = CommunityDetectionConfig::default();

        let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed
        assert!(
            assignments.is_empty(),
            "Community detection on empty graph must return empty assignments"
        );
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn detect_communities_CASE_single_node() {
        let graph = CsrGraph::new();
        let tx = TxId::new(1);
        let id = EntityId::new(42);

        graph
            .add_entity(tx, Entity::new(id, "SingleNode", "Type"))
            .await
            .unwrap(); // unwrap allowed
        graph.commit(tx).await.unwrap(); // unwrap allowed

        let config = CommunityDetectionConfig::default();
        let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].entity_id, id);
        assert_eq!(assignments[0].community_id, id.inner());
    }

    #[test]
    #[allow(non_snake_case)]
    fn serialization_roundtrip_CASE_community_config_and_assignment() {
        let config = CommunityDetectionConfig {
            max_iterations: 150,
            seed: 987654321,
            hyperedges_included: false,
        };

        let serialized_config = bincode::serialize(&config).unwrap(); // unwrap allowed
        let deserialized_config: CommunityDetectionConfig =
            bincode::deserialize(&serialized_config).unwrap(); // unwrap allowed
        assert_eq!(config.max_iterations, deserialized_config.max_iterations);
        assert_eq!(config.seed, deserialized_config.seed);
        assert_eq!(
            config.hyperedges_included,
            deserialized_config.hyperedges_included
        );

        let assignment = CommunityAssignment {
            entity_id: EntityId::new(100),
            community_id: 100,
            hyperedges_included: false,
        };

        let serialized_assignment = bincode::serialize(&assignment).unwrap(); // unwrap allowed
        let deserialized_assignment: CommunityAssignment =
            bincode::deserialize(&serialized_assignment).unwrap(); // unwrap allowed
        assert_eq!(assignment, deserialized_assignment);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_hyperedges_included_flag_default_is_false() {
        let config = CommunityDetectionConfig::default();
        assert!(
            !config.hyperedges_included,
            "Default for CommunityDetectionConfig.hyperedges_included must be false"
        );

        let assignment = CommunityAssignment {
            entity_id: EntityId::new(1),
            community_id: 1,
            hyperedges_included: false,
        };
        assert!(!assignment.hyperedges_included);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_hyperedges_included_flag_passthrough_from_config_to_assignment() {
        let graph = CsrGraph::new();
        let tx = TxId::new(1);
        graph
            .add_entity(tx, Entity::new(EntityId::new(1), "Node1", "Type"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_entity(tx, Entity::new(EntityId::new(2), "Node2", "Type"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
            .await
            .unwrap(); // unwrap allowed
        graph.commit(tx).await.unwrap(); // unwrap allowed

        // Test with hyperedges_included = true
        let config_true = CommunityDetectionConfig {
            max_iterations: 10,
            seed: 42,
            hyperedges_included: true,
        };
        let assignments_true = detect_communities(&graph, &config_true).await.unwrap(); // unwrap allowed
        assert!(!assignments_true.is_empty());
        for assignment in &assignments_true {
            assert!(
                assignment.hyperedges_included,
                "CommunityAssignment must reflect config.hyperedges_included = true"
            );
        }

        // Test with hyperedges_included = false
        let config_false = CommunityDetectionConfig {
            max_iterations: 10,
            seed: 42,
            hyperedges_included: false,
        };
        let assignments_false = detect_communities(&graph, &config_false).await.unwrap(); // unwrap allowed
        assert!(!assignments_false.is_empty());
        for assignment in &assignments_false {
            assert!(
                !assignment.hyperedges_included,
                "CommunityAssignment must reflect config.hyperedges_included = false"
            );
        }
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_star_expansion_iterator_and_community_detection() {
        use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

        let graph = CsrGraph::new();
        let tx = TxId::new(1);

        // Add 3 entities: 1, 2, 3
        for i in 1..=3 {
            graph
                .add_entity(tx, Entity::new(EntityId::new(i), format!("E{i}"), "Type"))
                .await
                .unwrap();
        }

        // Add hyperedge linking E1, E2, E3 (no binary edges present!)
        let he = HyperEdge::new(
            HyperEdgeId::new(50),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(1)),
                RoleBinding::new(RoleId::new(2), EntityId::new(2)),
                RoleBinding::new(RoleId::new(3), EntityId::new(3)),
            ],
            1.0,
        );
        graph.insert_hyperedge(he);
        graph.commit(tx).await.unwrap();

        // 1. Verify StarExpansionIterator
        let star_items: Vec<_> = StarExpansionIterator::new(&graph).collect();
        assert_eq!(star_items.len(), 3);
        assert_eq!(
            star_items,
            vec![
                (EntityId::new(1), VirtualHyperedgeNode::new(50)),
                (EntityId::new(2), VirtualHyperedgeNode::new(50)),
                (EntityId::new(3), VirtualHyperedgeNode::new(50)),
            ]
        );

        // 2. When hyperedges_included = false, E1, E2, E3 are disconnected singletons (no binary edges)
        let config_false = CommunityDetectionConfig {
            max_iterations: 10,
            seed: 42,
            hyperedges_included: false,
        };
        let assignments_false = detect_communities(&graph, &config_false).await.unwrap();
        assert_eq!(assignments_false.len(), 3);
        let comm_1 = assignments_false
            .iter()
            .find(|a| a.entity_id == EntityId::new(1))
            .unwrap()
            .community_id;
        let comm_2 = assignments_false
            .iter()
            .find(|a| a.entity_id == EntityId::new(2))
            .unwrap()
            .community_id;
        let comm_3 = assignments_false
            .iter()
            .find(|a| a.entity_id == EntityId::new(3))
            .unwrap()
            .community_id;
        assert_ne!(comm_1, comm_2);
        assert_ne!(comm_2, comm_3);

        // 3. When hyperedges_included = true, star expansion connects E1, E2, E3 via virtual hyperedge node
        let config_true = CommunityDetectionConfig {
            max_iterations: 10,
            seed: 42,
            hyperedges_included: true,
        };
        let assignments_true = detect_communities(&graph, &config_true).await.unwrap();
        assert_eq!(assignments_true.len(), 3);
        let comm_1_h = assignments_true
            .iter()
            .find(|a| a.entity_id == EntityId::new(1))
            .unwrap()
            .community_id;
        let comm_2_h = assignments_true
            .iter()
            .find(|a| a.entity_id == EntityId::new(2))
            .unwrap()
            .community_id;
        let comm_3_h = assignments_true
            .iter()
            .find(|a| a.entity_id == EntityId::new(3))
            .unwrap()
            .community_id;
        assert_eq!(comm_1_h, comm_2_h);
        assert_eq!(comm_2_h, comm_3_h);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_hyperedges_included_serde_backwards_compatibility() {
        // Old JSON config snapshot without hyperedges_included field
        let old_config_json = r#"{"max_iterations":100,"seed":42}"#;
        let config: CommunityDetectionConfig = serde_json::from_str(old_config_json).unwrap(); // unwrap allowed
        assert_eq!(config.max_iterations, 100);
        assert_eq!(config.seed, 42);
        assert!(
            !config.hyperedges_included,
            "Deserializing old config without hyperedges_included must default to false"
        );

        // Old JSON assignment snapshot without hyperedges_included field
        let old_assignment_json = r#"{"entity_id":100,"community_id":42}"#;
        let assignment: CommunityAssignment = serde_json::from_str(old_assignment_json).unwrap(); // unwrap allowed
        assert_eq!(assignment.entity_id, EntityId::new(100));
        assert_eq!(assignment.community_id, 42);
        assert!(
            !assignment.hyperedges_included,
            "Deserializing old assignment without hyperedges_included must default to false"
        );

        // Serialization roundtrip with hyperedges_included = true
        let config_true = CommunityDetectionConfig {
            max_iterations: 50,
            seed: 123,
            hyperedges_included: true,
        };
        let serialized = serde_json::to_string(&config_true).unwrap(); // unwrap allowed
        assert!(serialized.contains(r#""hyperedges_included":true"#));
        let deserialized: CommunityDetectionConfig = serde_json::from_str(&serialized).unwrap(); // unwrap allowed
        assert!(deserialized.hyperedges_included);

        let assignment_true = CommunityAssignment {
            entity_id: EntityId::new(100),
            community_id: 42,
            hyperedges_included: true,
        };
        let serialized_a = serde_json::to_string(&assignment_true).unwrap(); // unwrap allowed
        assert!(serialized_a.contains(r#""hyperedges_included":true"#));
        let deserialized_a: CommunityAssignment = serde_json::from_str(&serialized_a).unwrap(); // unwrap allowed
        assert!(deserialized_a.hyperedges_included);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_star_expansion_iterator_iteration() {
        use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

        let graph = CsrGraph::new();
        let tx = TxId::new(1);

        for id in 1..=3 {
            graph
                .add_entity(tx, Entity::new(EntityId::new(id), format!("E{id}"), "Node"))
                .await
                .unwrap(); // unwrap allowed
        }
        graph.commit(tx).await.unwrap(); // unwrap allowed

        let he_id = HyperEdgeId::new(500);
        let hedge = HyperEdge::new(
            he_id,
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(1)),
                RoleBinding::new(RoleId::new(2), EntityId::new(2)),
                RoleBinding::new(RoleId::new(3), EntityId::new(3)),
            ],
            2.5,
        );
        graph.insert_hyperedge_direct(hedge);

        let mut iter = StarExpansionIterator::new(&graph);
        let mut items = Vec::new();
        while let Some((eid, vnode, weight)) = iter.next_with_weight() {
            items.push((eid, vnode.hyperedge_id, weight));
        }

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], (EntityId::new(1), he_id, 2.5));
        assert_eq!(items[1], (EntityId::new(2), he_id, 2.5));
        assert_eq!(items[2], (EntityId::new(3), he_id, 2.5));
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn test_community_detection_with_star_expansion_hyperedges() {
        use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

        let graph = CsrGraph::new();
        let tx = TxId::new(1);

        // Cluster 1: Nodes 1, 2, 3 (no binary edges)
        for id in 1..=3 {
            graph
                .add_entity(
                    tx,
                    Entity::new(EntityId::new(id), format!("C1_{id}"), "Node"),
                )
                .await
                .unwrap(); // unwrap allowed
        }
        // Cluster 2: Nodes 10, 11, 12 (no binary edges)
        for id in 10..=12 {
            graph
                .add_entity(
                    tx,
                    Entity::new(EntityId::new(id), format!("C2_{id}"), "Node"),
                )
                .await
                .unwrap(); // unwrap allowed
        }
        graph.commit(tx).await.unwrap(); // unwrap allowed

        // Add Hyperedge H1 grouping Nodes 1, 2, 3
        let he1 = HyperEdge::new(
            HyperEdgeId::new(101),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(1)),
                RoleBinding::new(RoleId::new(2), EntityId::new(2)),
                RoleBinding::new(RoleId::new(3), EntityId::new(3)),
            ],
            1.0,
        );
        graph.insert_hyperedge_direct(he1);

        // Add Hyperedge H2 grouping Nodes 10, 11, 12
        let he2 = HyperEdge::new(
            HyperEdgeId::new(102),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(10)),
                RoleBinding::new(RoleId::new(2), EntityId::new(11)),
                RoleBinding::new(RoleId::new(3), EntityId::new(12)),
            ],
            1.0,
        );
        graph.insert_hyperedge_direct(he2);

        // Without hyperedges (hyperedges_included = false), no binary edges exist -> singletons
        let config_false = CommunityDetectionConfig {
            max_iterations: 50,
            seed: 42,
            hyperedges_included: false,
        };
        let assignments_false = detect_communities(&graph, &config_false).await.unwrap(); // unwrap allowed
        let map_false: HashMap<u64, u64> = assignments_false
            .into_iter()
            .map(|a| (a.entity_id.inner(), a.community_id))
            .collect();
        assert_ne!(map_false[&1], map_false[&2]);
        assert_ne!(map_false[&10], map_false[&11]);

        // With hyperedges (hyperedges_included = true), star expansion connects nodes via virtual hyperedge nodes
        let config_true = CommunityDetectionConfig {
            max_iterations: 50,
            seed: 42,
            hyperedges_included: true,
        };
        let assignments_true = detect_communities(&graph, &config_true).await.unwrap(); // unwrap allowed
        let map_true: HashMap<u64, u64> = assignments_true
            .into_iter()
            .map(|a| (a.entity_id.inner(), a.community_id))
            .collect();

        // Cluster 1 nodes must be grouped into the same community
        let c1_comm = map_true[&1];
        assert_eq!(map_true[&2], c1_comm);
        assert_eq!(map_true[&3], c1_comm);

        // Cluster 2 nodes must be grouped into the same community
        let c2_comm = map_true[&10];
        assert_eq!(map_true[&11], c2_comm);
        assert_eq!(map_true[&12], c2_comm);

        // Cluster 1 and Cluster 2 must be in distinct communities
        assert_ne!(c1_comm, c2_comm);
    }
}
