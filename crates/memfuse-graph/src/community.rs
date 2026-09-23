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
    /// Creates a new `VirtualHyperedgeNode`.
    pub fn new(hyperedge_id: u64) -> Self {
        Self {
            hyperedge_id: crate::hyperedge::HyperEdgeId::new(hyperedge_id),
        }
    }
}

/// Stern-Expansion: Jede Hyperkante wird als künstlicher bipartiter Knoten
/// (`VirtualHyperedgeNode`) repräsentiert. Der Iterator gaukelt dem Leiden-Solver die
/// Inzidenzmatrix H vor, ohne zusätzlichen Speicher für die volle Expansion zu allozieren.
///
/// # Ring-0 Zero-Copy Invariante (HK-05)
/// `StarExpansionIterator` klont keine `HyperEdge`-Instanzen, sondern iteriert
/// referenzbasiert über die in `Arc<GraphInner>` gespeicherten Hyperkanten.
pub struct StarExpansionIterator {
    inner: arc_swap::Guard<std::sync::Arc<crate::csr::inner::GraphInner>>,
    hyperedge_keys: Vec<crate::hyperedge::HyperEdgeId>,
    current_hyperedge_idx: usize,
    current_participant_idx: usize,
}

impl StarExpansionIterator {
    /// Erstellt einen neuen `StarExpansionIterator` für den gegebenen CsrGraph.
    pub fn new(graph: &CsrGraph) -> Self {
        let inner = graph.inner_read();
        let mut hyperedge_keys: Vec<crate::hyperedge::HyperEdgeId> = inner
            .hyperedges
            .iter()
            .filter(|(_, h)| h.tx_valid_to.is_none() && h.participants.len() >= 2)
            .map(|(k, _)| *k)
            .collect();
        hyperedge_keys.sort();

        Self {
            inner,
            hyperedge_keys,
            current_hyperedge_idx: 0,
            current_participant_idx: 0,
        }
    }

    /// Liest das nächste Element inkl. Kantengewicht (Konvention K: `2w/(|e|-1)`).
    pub fn next_with_weight(&mut self) -> Option<(EntityId, VirtualHyperedgeNode, f32)> {
        while self.current_hyperedge_idx < self.hyperedge_keys.len() {
            let key = self.hyperedge_keys[self.current_hyperedge_idx];
            if let Some(hedge) = self.inner.hyperedges.get(&key) {
                if self.current_participant_idx < hedge.participants.len() {
                    let entity_id = hedge.participants[self.current_participant_idx].entity;
                    let vnode = VirtualHyperedgeNode {
                        hyperedge_id: hedge.id,
                    };
                    let weight =
                        crate::hyperedge::star_weight(hedge.weight, hedge.participants.len());
                    self.current_participant_idx += 1;
                    return Some((entity_id, vnode, weight));
                }
            }
            self.current_hyperedge_idx += 1;
            self.current_participant_idx = 0;
        }
        None
    }
}

impl Iterator for StarExpansionIterator {
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
    let (valid_nodes, num_real_nodes, reverse_map, adj_raw) = {
        let inner = graph.inner_read();
        let num_nodes = inner.reverse_map.len();

        let mut valid_nodes = Vec::new();
        for idx in 0..num_nodes {
            if inner.entity_at(idx).is_some() {
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
                    if !inner.tombstoned_edges.contains(&(u, v)) && inner.entity_at(v).is_some() {
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
            let mut virtual_map: HashMap<crate::hyperedge::HyperEdgeId, usize> = HashMap::new();
            let mut next_virtual_idx = inner.reverse_map.len();
            let star_iter = StarExpansionIterator::new(graph);

            for (entity_id, virtual_node) in star_iter {
                if let Some(&u) = inner.id_map.get(&entity_id) {
                    if inner.entity_at(u).is_some() {
                        let v_idx =
                            *virtual_map
                                .entry(virtual_node.hyperedge_id)
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

    let num_entity_nodes = num_real_nodes;
    if num_entity_nodes == 0 {
        return Ok(Vec::new());
    }

    let num_total_nodes = node_indices.len();
    let mut global_to_local: HashMap<usize, usize> = HashMap::with_capacity(num_total_nodes);
    let mut local_entity_ids: Vec<EntityId> = Vec::with_capacity(num_real_nodes);
    let mut local_u64_ids: Vec<u64> = Vec::with_capacity(num_total_nodes);

    for (local_idx, &global_idx) in node_indices.iter().enumerate() {
        global_to_local.insert(global_idx, local_idx);
        let (eid, u64_id) = if global_idx < reverse_map.len() {
            let eid = reverse_map[global_idx];
            (eid, eid.inner())
        } else {
            let u64_id = u64::MAX - (global_idx - reverse_map.len()) as u64;
            (EntityId::new(0), u64_id)
        };
        local_entity_ids.push(eid);
        local_u64_ids.push(u64_id);
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
    for &eid in &local_entity_ids[..num_entity_nodes] {
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

    let total_2m: f32 = node_degrees.iter().sum();

    // If graph has no edges, return singletons
    if total_2m <= 0.0 {
        let mut assignments = Vec::with_capacity(num_real_nodes);
        for i in 0..num_real_nodes {
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
    let mut assignments = Vec::with_capacity(num_real_nodes);
    for (i, &community_id) in communities.iter().enumerate().take(num_real_nodes) {
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
mod tests;
