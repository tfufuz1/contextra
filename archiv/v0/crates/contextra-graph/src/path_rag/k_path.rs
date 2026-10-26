//! Budgeted k-Path Diffusion for PathRAG Engine.
//!
//! Hard node-budgeted bidirectional Dijkstra path discovery to prevent memory explosion
//! when traversing hub entities with high out-degree.

use crate::path_rag::{PathGraph, PathRAGEngine};
use ahash::AHashSet;
use contextra_types::EntityId;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Configuration parameters for [`KPathDiffusion`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KPathConfig {
    /// Target number of paths to discover.
    pub k: usize,
    /// Maximum number of distinct graph nodes visited across forward/backward search.
    pub max_visited_nodes: usize,
    /// Maximum path length in hops (`nodes.len() - 1 <= max_hops`).
    pub max_hops: usize,
}

impl Default for KPathConfig {
    fn default() -> Self {
        Self {
            k: 3,
            max_visited_nodes: 1000,
            max_hops: 4,
        }
    }
}

/// Result returned by [`KPathDiffusion::find_k_paths`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KPathResult {
    /// Up to `k` discovered paths between source and target.
    pub paths: Vec<Vec<EntityId>>,
    /// Indicates whether the search was halted due to `max_visited_nodes` limit.
    pub budget_exhausted: bool,
}

/// Trait for budgeted k-path diffusion search over entity graphs.
pub trait KPathDiffusion: Send + Sync {
    /// Discovers up to `k` paths between `source` and `target` bounded by `config`.
    ///
    /// Budget exhaustion (`budget_exhausted: true`) is returned as a regular `Ok(KPathResult)`.
    fn find_k_paths(
        &self,
        source: EntityId,
        target: EntityId,
        config: &KPathConfig,
    ) -> contextra_types::Result<KPathResult>;
}

#[derive(Debug, Clone)]
struct HeapItem {
    dist_bits: u32,
    path: Vec<EntityId>,
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.dist_bits == other.dist_bits && self.path == other.path
    }
}

impl Eq for HeapItem {}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .dist_bits
            .cmp(&self.dist_bits)
            .then_with(|| self.path.len().cmp(&other.path.len()))
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn is_simple_path(path: &[EntityId]) -> bool {
    let mut seen = AHashSet::with_capacity(path.len());
    path.iter().all(|x| seen.insert(*x))
}

impl<G: PathGraph> KPathDiffusion for PathRAGEngine<G> {
    fn find_k_paths(
        &self,
        source: EntityId,
        target: EntityId,
        config: &KPathConfig,
    ) -> contextra_types::Result<KPathResult> {
        if config.k == 0 {
            return Ok(KPathResult {
                paths: vec![],
                budget_exhausted: false,
            });
        }

        if source == target {
            return Ok(KPathResult {
                paths: vec![vec![source]],
                budget_exhausted: false,
            });
        }

        if config.max_visited_nodes == 0 {
            return Ok(KPathResult {
                paths: vec![],
                budget_exhausted: true,
            });
        }

        let mut visited_set: AHashSet<EntityId> = AHashSet::new();
        let mut budget_exhausted = false;

        visited_set.insert(source);
        if visited_set.len() >= config.max_visited_nodes {
            if !visited_set.contains(&target) {
                return Ok(KPathResult {
                    paths: vec![],
                    budget_exhausted: true,
                });
            }
        } else {
            visited_set.insert(target);
            if visited_set.len() >= config.max_visited_nodes && config.max_visited_nodes < 2 {
                return Ok(KPathResult {
                    paths: vec![],
                    budget_exhausted: true,
                });
            }
        }

        let mut found_paths: Vec<Vec<EntityId>> = Vec::new();
        let mut seen_paths: AHashSet<Vec<EntityId>> = AHashSet::new();

        let mut fwd_paths: HashMap<EntityId, Vec<Vec<EntityId>>> = HashMap::new();
        let mut bwd_paths: HashMap<EntityId, Vec<Vec<EntityId>>> = HashMap::new();

        fwd_paths.insert(source, vec![vec![source]]);
        bwd_paths.insert(target, vec![vec![target]]);

        let mut heap_fwd: BinaryHeap<HeapItem> = BinaryHeap::new();
        let mut heap_bwd: BinaryHeap<HeapItem> = BinaryHeap::new();

        heap_fwd.push(HeapItem {
            dist_bits: 0.0f32.to_bits(),
            path: vec![source],
        });
        heap_bwd.push(HeapItem {
            dist_bits: 0.0f32.to_bits(),
            path: vec![target],
        });

        let mut steps = 0;
        let max_steps = config.max_hops * 1000 + 100;

        while (!heap_fwd.is_empty() || !heap_bwd.is_empty()) && steps < max_steps {
            steps += 1;

            if found_paths.len() >= config.k {
                break;
            }

            // Forward step
            if let Some(HeapItem {
                dist_bits,
                path: curr_path,
            }) = heap_fwd.pop()
            {
                let dist = f32::from_bits(dist_bits);
                let Some(&u) = curr_path.last() else {
                    continue;
                };
                let hops = curr_path.len() - 1;

                if hops < config.max_hops {
                    for (neighbor, weight) in self.get_expanded_neighbors(u) {
                        if curr_path.contains(&neighbor) {
                            continue;
                        }

                        let new_hops = hops + 1;
                        if new_hops > config.max_hops {
                            continue;
                        }

                        let mut next_path = curr_path.clone();
                        next_path.push(neighbor);

                        let new_dist = dist + (1.0 / weight.max(1e-8));

                        if !visited_set.contains(&neighbor) {
                            if visited_set.len() >= config.max_visited_nodes {
                                budget_exhausted = true;
                                break;
                            }
                            visited_set.insert(neighbor);
                        }

                        let node_fwd = fwd_paths.entry(neighbor).or_default();
                        if node_fwd.len() < config.k && !node_fwd.contains(&next_path) {
                            node_fwd.push(next_path.clone());

                            if let Some(bwd_list) = bwd_paths.get(&neighbor) {
                                for b_path in bwd_list {
                                    let mut full_path = next_path.clone();
                                    for &node in b_path.iter().rev().skip(1) {
                                        full_path.push(node);
                                    }
                                    if full_path.len() - 1 <= config.max_hops
                                        && is_simple_path(&full_path)
                                        && seen_paths.insert(full_path.clone())
                                    {
                                        found_paths.push(full_path);
                                        if found_paths.len() >= config.k {
                                            break;
                                        }
                                    }
                                }
                            }

                            if found_paths.len() >= config.k {
                                break;
                            }

                            heap_fwd.push(HeapItem {
                                dist_bits: new_dist.to_bits(),
                                path: next_path,
                            });
                        }
                    }
                }
            }

            if found_paths.len() >= config.k || budget_exhausted {
                break;
            }

            // Backward step
            if let Some(HeapItem {
                dist_bits,
                path: curr_path,
            }) = heap_bwd.pop()
            {
                let dist = f32::from_bits(dist_bits);
                let Some(&u) = curr_path.last() else {
                    continue;
                };
                let hops = curr_path.len() - 1;

                if hops < config.max_hops {
                    for (pred, weight) in self.get_expanded_predecessors(u) {
                        if curr_path.contains(&pred) {
                            continue;
                        }

                        let new_hops = hops + 1;
                        if new_hops > config.max_hops {
                            continue;
                        }

                        let mut next_path = curr_path.clone();
                        next_path.push(pred);

                        let new_dist = dist + (1.0 / weight.max(1e-8));

                        if !visited_set.contains(&pred) {
                            if visited_set.len() >= config.max_visited_nodes {
                                budget_exhausted = true;
                                break;
                            }
                            visited_set.insert(pred);
                        }

                        let node_bwd = bwd_paths.entry(pred).or_default();
                        if node_bwd.len() < config.k && !node_bwd.contains(&next_path) {
                            node_bwd.push(next_path.clone());

                            if let Some(fwd_list) = fwd_paths.get(&pred) {
                                for f_path in fwd_list {
                                    let mut full_path = f_path.clone();
                                    for &node in next_path.iter().rev().skip(1) {
                                        full_path.push(node);
                                    }
                                    if full_path.len() - 1 <= config.max_hops
                                        && is_simple_path(&full_path)
                                        && seen_paths.insert(full_path.clone())
                                    {
                                        found_paths.push(full_path);
                                        if found_paths.len() >= config.k {
                                            break;
                                        }
                                    }
                                }
                            }

                            if found_paths.len() >= config.k {
                                break;
                            }

                            heap_bwd.push(HeapItem {
                                dist_bits: new_dist.to_bits(),
                                path: next_path,
                            });
                        }
                    }
                }
            }
        }

        Ok(KPathResult {
            paths: found_paths,
            budget_exhausted,
        })
    }
}
