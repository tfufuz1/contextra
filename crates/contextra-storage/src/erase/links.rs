//! Cluster-local `DerivedFrom` graph with bounded erase closure (SPEC §9.6).
//!
//! A link plan is computed outside the commit lock and carries a graph
//! generation. The sequencer must apply it only if that generation still
//! matches, then persist the resulting link operation in the same durable
//! control frame as any dependent state transition.

use contextra_storage_ports::{ClusterId, Error};
use super::cluster_table::ClusterTable;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const DEFAULT_MAX_LINKS: usize = 8;
pub const DEFAULT_MAX_GATE_CLUSTERS: usize = 4096;
pub const DEFAULT_MAX_LINK_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlan {
    generation: u64,
    derived: ClusterId,
    sources: Vec<ClusterId>,
}

impl LinkPlan {
    pub fn generation(&self) -> u64 { self.generation }
    pub fn derived(&self) -> ClusterId { self.derived }
    pub fn sources(&self) -> &[ClusterId] { &self.sources }
}

#[derive(Debug, Clone)]
pub struct LinkGraph {
    /// `derived -> direct sources`; ordered collections make closure stable.
    sources: BTreeMap<ClusterId, BTreeSet<ClusterId>>,
    generation: u64,
    max_links: usize,
    max_gate_clusters: usize,
    max_depth: usize,
}

impl Default for LinkGraph {
    fn default() -> Self {
        Self {
            sources: BTreeMap::new(), generation: 0,
            max_links: DEFAULT_MAX_LINKS,
            max_gate_clusters: DEFAULT_MAX_GATE_CLUSTERS,
            max_depth: DEFAULT_MAX_LINK_DEPTH,
        }
    }
}

impl LinkGraph {
    pub fn new(max_links: usize, max_gate_clusters: usize, max_depth: usize) -> Result<Self, Error> {
        if max_links == 0 || max_gate_clusters == 0 || max_depth == 0 {
            return Err(Error::Limit { limit: "link graph limits must be nonzero" });
        }
        Ok(Self { sources: BTreeMap::new(), generation: 0, max_links, max_gate_clusters, max_depth })
    }

    pub fn generation(&self) -> u64 { self.generation }
    pub fn direct_sources(&self, derived: ClusterId) -> impl Iterator<Item = ClusterId> + '_ {
        self.sources.get(&derived).into_iter().flat_map(|set| set.iter().copied())
    }

    /// Computes, but does not publish, a link operation. The candidate graph
    /// is validated for cycles and depth before a plan can reach the writer.
    pub fn plan_link(&self, clusters: &ClusterTable, derived: ClusterId, sources: &[ClusterId]) -> Result<LinkPlan, Error> {
        if sources.is_empty() || sources.len() > self.max_links {
            return Err(Error::Limit { limit: "derived link source count" });
        }
        let unique: BTreeSet<_> = sources.iter().copied().collect();
        if unique.len() != sources.len() || unique.contains(&derived) {
            return Err(Error::Limit { limit: "duplicate or self-referential cluster link" });
        }
        clusters.require_live(derived)?;
        for source in &unique { clusters.require_live(*source)?; }
        let mut candidate = self.sources.clone();
        if candidate.contains_key(&derived) {
            return Err(Error::Limit { limit: "cluster already has a DerivedFrom link" });
        }
        candidate.insert(derived, unique.clone());
        validate_acyclic(&candidate, self.max_depth)?;
        // The new child is added to the erase closure of its sources and every
        // ancestor that depends on them. Check only those affected roots.
        for node in affected_ancestors(&self.sources, &unique) {
            if closure_in(&self.sources, node, self.max_gate_clusters)?.len() >= self.max_gate_clusters {
                return Err(Error::EraseFanoutExceeded);
            }
        }
        Ok(LinkPlan { generation: self.generation, derived, sources: unique.into_iter().collect() })
    }

    /// Publishes a previously planned link iff no intervening graph mutation
    /// invalidated its cycle/fanout proof.
    pub fn apply_link(&mut self, clusters: &ClusterTable, plan: LinkPlan) -> Result<(), Error> {
        if plan.generation != self.generation { return Err(Error::Conflict); }
        if self.sources.contains_key(&plan.derived) { return Err(Error::Conflict); }
        clusters.require_live(plan.derived)?;
        let next_generation = self.generation.checked_add(1).ok_or(Error::Exhausted)?;
        let set: BTreeSet<_> = plan.sources.into_iter().collect();
        if set.is_empty() || set.len() > self.max_links || set.contains(&plan.derived) {
            return Err(Error::Corrupt { place: "invalid persisted link operation" });
        }
        for source in &set { clusters.require_live(*source)?; }
        let mut candidate = self.sources.clone();
        candidate.insert(plan.derived, set.clone());
        validate_acyclic(&candidate, self.max_depth)?;
        for node in affected_ancestors(&self.sources, &set) {
            if closure_in(&self.sources, node, self.max_gate_clusters)?.len() >= self.max_gate_clusters {
                return Err(Error::EraseFanoutExceeded);
            }
        }
        self.sources = candidate;
        self.generation = next_generation;
        Ok(())
    }

    /// Returns source plus every transitive derived cluster. A child with
    /// multiple sources is included when any one source is in the closure.
    pub fn erase_closure(&self, source: ClusterId) -> Result<Vec<ClusterId>, Error> {
        let closure = closure_in(&self.sources, source, self.max_gate_clusters)?;
        if closure.len() > self.max_gate_clusters { return Err(Error::EraseFanoutExceeded); }
        Ok(closure.into_iter().collect())
    }

    /// Unlink is a durable control mutation; this method only applies its
    /// replayed/in-memory projection and rejects stale graph generations.
    pub fn unlink(&mut self, expected_generation: u64, derived: ClusterId) -> Result<(), Error> {
        if expected_generation != self.generation { return Err(Error::Conflict); }
        if self.sources.values().any(|parents| parents.contains(&derived)) {
            return Err(Error::Limit { limit: "linked derived cluster still has children" });
        }
        if self.sources.remove(&derived).is_none() { return Err(Error::Limit { limit: "cluster link not found" }); }
        self.generation = self.generation.checked_add(1).ok_or(Error::Exhausted)?;
        Ok(())
    }
}

fn validate_acyclic(graph: &BTreeMap<ClusterId, BTreeSet<ClusterId>>, max_depth: usize) -> Result<(), Error> {
    let mut reverse: BTreeMap<ClusterId, Vec<ClusterId>> = BTreeMap::new();
    let mut remaining = BTreeMap::<ClusterId, usize>::new();
    for (node, parents) in graph {
        remaining.insert(*node, parents.len());
        for parent in parents { reverse.entry(*parent).or_default().push(*node); }
    }
    for parent in reverse.keys() { remaining.entry(*parent).or_insert(0); }
    let mut queue: VecDeque<_> = remaining.iter().filter_map(|(id, count)| (*count == 0).then_some(*id)).collect();
    let mut depth = BTreeMap::<ClusterId, usize>::new();
    for node in remaining.keys() { depth.insert(*node, 0); }
    let mut visited = 0usize;
    while let Some(dependency) = queue.pop_front() {
        visited += 1;
        let dependency_depth = *depth.get(&dependency).unwrap_or(&0);
        if dependency_depth > max_depth { return Err(Error::Limit { limit: "derived link depth" }); }
        if let Some(children) = reverse.get(&dependency) {
            for child in children {
                let child_depth = depth.get_mut(child).ok_or(Error::Corrupt { place: "link depth index" })?;
                *child_depth = (*child_depth).max(dependency_depth + 1);
                let count = remaining.get_mut(child).ok_or(Error::Corrupt { place: "link indegree index" })?;
                *count -= 1;
                if *count == 0 { queue.push_back(*child); }
            }
        }
    }
    if visited != remaining.len() { return Err(Error::Limit { limit: "derived link cycle" }); }
    Ok(())
}

fn affected_ancestors(
    graph: &BTreeMap<ClusterId, BTreeSet<ClusterId>>,
    sources: &BTreeSet<ClusterId>,
) -> BTreeSet<ClusterId> {
    let mut reverse: BTreeMap<ClusterId, Vec<ClusterId>> = BTreeMap::new();
    for (derived, dependencies) in graph {
        for dependency in dependencies { reverse.entry(*dependency).or_default().push(*derived); }
    }
    let mut affected = sources.clone();
    let mut queue: VecDeque<_> = sources.iter().copied().collect();
    while let Some(node) = queue.pop_front() {
        if let Some(parents) = reverse.get(&node) {
            for parent in parents {
                if affected.insert(*parent) { queue.push_back(*parent); }
            }
        }
    }
    affected
}

fn closure_in(
    graph: &BTreeMap<ClusterId, BTreeSet<ClusterId>>,
    source: ClusterId,
    limit: usize,
) -> Result<BTreeSet<ClusterId>, Error> {
    let mut reverse: BTreeMap<ClusterId, Vec<ClusterId>> = BTreeMap::new();
    for (derived, parents) in graph {
        for parent in parents { reverse.entry(*parent).or_default().push(*derived); }
    }
    let mut closure = BTreeSet::from([source]);
    let mut queue = VecDeque::from([source]);
    while let Some(current) = queue.pop_front() {
        if let Some(children) = reverse.get(&current) {
            for derived in children {
                if closure.insert(*derived) {
                    if closure.len() > limit { return Err(Error::EraseFanoutExceeded); }
                    queue.push_back(*derived);
                }
            }
        }
    }
    Ok(closure)
}
