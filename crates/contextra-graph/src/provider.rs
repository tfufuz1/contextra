//! Generic `GraphProvider` trait and implementations for isolated graph algorithms.

use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding};
use crate::path_rag::PathGraph;
use contextra_types::{EntityId, Result};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

/// Generic Graph Provider abstraction.
pub trait GraphProvider: Send + Sync {
    /// Returns all active node IDs in the graph.
    fn node_ids(&self) -> Vec<EntityId>;
    /// Returns the number of active nodes.
    fn node_count(&self) -> usize { self.node_ids().len() }
    /// Checks if a node exists.
    fn contains_node(&self, node: EntityId) -> bool;
    /// Returns outgoing neighbors of `node` with their edge weights.
    fn outgoing_edges(&self, node: EntityId) -> Vec<(EntityId, f32)>;
    /// Returns incoming predecessors of `node` with their edge weights.
    fn incoming_edges(&self, node: EntityId) -> Vec<(EntityId, f32)>;
    /// Returns out-degree.
    fn out_degree(&self, node: EntityId) -> usize { self.outgoing_edges(node).len() }
    /// Returns in-degree.
    fn in_degree(&self, node: EntityId) -> usize { self.incoming_edges(node).len() }
    /// Returns hyperedge IDs for entity.
    fn hyperedges_for_entity(&self, _node: EntityId) -> Vec<HyperEdgeId> { Vec::new() }
    /// Resolves HyperEdgeId.
    fn get_hyperedge(&self, _id: HyperEdgeId) -> Option<Arc<HyperEdge>> { None }
    /// Returns role bindings.
    fn hyperedge_participants(&self, id: HyperEdgeId) -> Arc<[RoleBinding]> {
        self.get_hyperedge(id).map(|he| he.participants.clone()).unwrap_or_else(|| Arc::from([]))
    }
}

impl PathGraph for InMemoryGraphProvider {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> { self.outgoing_edges(node) }
    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> { self.incoming_edges(node) }
    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<HyperEdgeId> { GraphProvider::hyperedges_for_entity(self, node) }
    fn get_hyperedge(&self, id: HyperEdgeId) -> Option<Arc<HyperEdge>> { GraphProvider::get_hyperedge(self, id) }
    fn hyperedge_participants(&self, id: HyperEdgeId) -> Arc<[RoleBinding]> { GraphProvider::hyperedge_participants(self, id) }
}

/// Standalone in-memory reference implementation of `GraphProvider`.
#[derive(Debug, Clone, Default)]
pub struct InMemoryGraphProvider {
    nodes: BTreeSet<EntityId>,
    outgoing: BTreeMap<EntityId, Vec<(EntityId, f32)>>,
    incoming: BTreeMap<EntityId, Vec<(EntityId, f32)>>,
    hyperedges: BTreeMap<HyperEdgeId, Arc<HyperEdge>>,
    entity_hyperedges: BTreeMap<EntityId, BTreeSet<HyperEdgeId>>,
}

impl InMemoryGraphProvider {
    pub fn new() -> Self { Self::default() }
    pub fn add_node(&mut self, node: EntityId) {
        self.nodes.insert(node);
        self.outgoing.entry(node).or_default();
        self.incoming.entry(node).or_default();
    }
    pub fn add_edge(&mut self, source: EntityId, target: EntityId, weight: f32) {
        self.add_node(source);
        self.add_node(target);
        let out = self.outgoing.entry(source).or_default();
        if let Some(pos) = out.iter().position(|(e, _)| *e == target) { out[pos].1 = weight; } else { out.push((target, weight)); }
        let inc = self.incoming.entry(target).or_default();
        if let Some(pos) = inc.iter().position(|(e, _)| *e == source) { inc[pos].1 = weight; } else { inc.push((source, weight)); }
    }
    pub fn add_hyperedge(&mut self, hyperedge: HyperEdge) {
        let id = hyperedge.id;
        for participant in hyperedge.participants.iter() {
            self.add_node(participant.entity);
            self.entity_hyperedges.entry(participant.entity).or_default().insert(id);
        }
        self.hyperedges.insert(id, Arc::new(hyperedge));
    }
}

impl GraphProvider for InMemoryGraphProvider {
    fn node_ids(&self) -> Vec<EntityId> { self.nodes.iter().copied().collect() }
    fn node_count(&self) -> usize { self.nodes.len() }
    fn contains_node(&self, node: EntityId) -> bool { self.nodes.contains(&node) }
    fn outgoing_edges(&self, node: EntityId) -> Vec<(EntityId, f32)> { self.outgoing.get(&node).cloned().unwrap_or_default() }
    fn incoming_edges(&self, node: EntityId) -> Vec<(EntityId, f32)> { self.incoming.get(&node).cloned().unwrap_or_default() }
    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<HyperEdgeId> {
        self.entity_hyperedges.get(&node).map(|set| set.iter().copied().collect()).unwrap_or_default()
    }
    fn get_hyperedge(&self, id: HyperEdgeId) -> Option<Arc<HyperEdge>> { self.hyperedges.get(&id).cloned() }
}

/// Generic PageRank computation executing against any `GraphProvider`.
pub fn compute_pagerank<G: GraphProvider>(graph: &G, damping: f32, max_iter: usize, tol: f32) -> HashMap<EntityId, f32> {
    let nodes = graph.node_ids();
    let n = nodes.len();
    if n == 0 { return HashMap::new(); }
    let node_to_idx: HashMap<EntityId, usize> = nodes.iter().enumerate().map(|(idx, &id)| (id, idx)).collect();
    let mut ranks = vec![1.0 / (n as f32); n];
    let mut out_degrees = vec![0usize; n];
    let mut adj: Vec<Vec<(usize, f32)>> = vec![Vec::new(); n];
    for (i, &u) in nodes.iter().enumerate() {
        let edges = graph.outgoing_edges(u);
        out_degrees[i] = edges.len();
        for (v, w) in edges {
            if let Some(&target_idx) = node_to_idx.get(&v) { adj[i].push((target_idx, w)); }
        }
    }
    for _ in 0..max_iter {
        let mut next = vec![(1.0 - damping) / (n as f32); n];
        let d_sum: f32 = (0..n).filter(|&i| out_degrees[i] == 0).map(|i| ranks[i]).sum();
        let contrib = damping * d_sum / (n as f32);
        for r in &mut next { *r += contrib; }
        for i in 0..n {
            let deg = out_degrees[i];
            if deg > 0 {
                let share = damping * ranks[i] / (deg as f32);
                for &(tgt, _w) in &adj[i] { next[tgt] += share; }
            }
        }
        let diff: f32 = ranks.iter().zip(next.iter()).map(|(a, b)| (a - b).abs()).sum();
        ranks = next;
        if diff < tol { break; }
    }
    nodes.into_iter().enumerate().map(|(i, id)| (id, ranks[i])).collect()
}

/// Generic Community Detection on any `GraphProvider`.
pub fn detect_communities_on_provider<G: GraphProvider>(
    graph: &G,
    config: &crate::community::CommunityDetectionConfig,
) -> Result<Vec<crate::community::CommunityAssignment>> {
    let nodes = graph.node_ids();
    let num_real = nodes.len();
    if num_real == 0 { return Ok(Vec::new()); }
    let node_to_idx: HashMap<EntityId, usize> = nodes.iter().enumerate().map(|(i, &id)| (id, i)).collect();
    let mut local_adj: Vec<Vec<(usize, f32)>> = vec![Vec::new(); num_real];
    let mut degs: Vec<f32> = vec![0.0; num_real];
    for (u_idx, &u) in nodes.iter().enumerate() {
        for (v, w) in graph.outgoing_edges(u) {
            if let Some(&v_idx) = node_to_idx.get(&v) {
                let val = if w > 0.0 { w } else { 1.0 };
                local_adj[u_idx].push((v_idx, val));
                degs[u_idx] += val;
            }
        }
    }
    let total_2m: f32 = degs.iter().sum();
    let local_u64_ids: Vec<u64> = nodes.iter().map(|e| e.inner()).collect();
    if total_2m <= 0.0 {
        return Ok(nodes.iter().enumerate().map(|(i, &e)| crate::community::CommunityAssignment {
            entity_id: e, community_id: local_u64_ids[i], hyperedges_included: config.hyperedges_included,
        }).collect());
    }
    let mut communities = local_u64_ids.clone();
    let mut comm_degs: HashMap<u64, f32> = communities.iter().enumerate().map(|(i, &c)| (c, degs[i])).collect();
    for _ in 0..config.max_iterations {
        let mut changed = 0;
        for u in 0..num_real {
            let d_u = degs[u];
            if d_u <= 0.0 { continue; }
            let c_cur = communities[u];
            let mut weights: HashMap<u64, f32> = HashMap::new();
            for &(v, w) in &local_adj[u] { *weights.entry(communities[v]).or_default() += w; }
            let w_cur = weights.get(&c_cur).copied().unwrap_or(0.0);
            let d_c_cur = comm_degs.get(&c_cur).copied().unwrap_or(0.0);
            let mut max_gain = 1e-6f32;
            let mut best_c = None;
            for (&c_cand, &w_cand) in &weights {
                if c_cand == c_cur { continue; }
                let d_c = comm_degs.get(&c_cand).copied().unwrap_or(0.0);
                let gain = (w_cand - w_cur) - (d_u * (d_c - (d_c_cur - d_u))) / total_2m;
                if gain > max_gain { max_gain = gain; best_c = Some(c_cand); }
                else if (gain - max_gain).abs() <= 1e-6 && best_c.map_or(true, |b| c_cand < b) { best_c = Some(c_cand); }
            }
            if let Some(c_best) = best_c {
                if c_best != c_cur {
                    communities[u] = c_best;
                    *comm_degs.entry(c_cur).or_default() -= d_u;
                    *comm_degs.entry(c_best).or_default() += d_u;
                    changed += 1;
                }
            }
        }
        if changed == 0 { break; }
    }
    Ok(nodes.into_iter().enumerate().map(|(i, e)| crate::community::CommunityAssignment {
        entity_id: e, community_id: communities[i], hyperedges_included: config.hyperedges_included,
    }).collect())
}
