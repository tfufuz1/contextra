//! Core diffusion loop and active region management for TL-HFD (Spec §21.1).

use super::error::TlHfdError;
use super::lovasz::{compute_lovasz_extension, truncate_participants};
use super::params::TlHfdParams;
use crate::path_rag::PathGraph;
use ahash::{AHashMap, AHashSet};
use contextra_types::EntityId;
use std::cmp::Ordering;

/// Internal representation of a deduplicated edge (binary or hyperedge) touching the active region.
#[derive(Debug, Clone)]
struct ActiveEdge {
    weight: f32,
    participants: Vec<EntityId>,
}

/// Runs Thresholded Local Hyper-Flow Diffusion over a generic [`PathGraph`].
///
/// # Invariants
/// - Deterministic execution: Floating-point operations and candidate activations follow strict `EntityId` total ordering.
/// - Zero Panic: Returns [`TlHfdError::NonFinite`] on non-finite values instead of panicking.
/// - Localized execution: Accesses only nodes and edges touching the active region $A$.
///
/// # Complexity
/// Time complexity $O(\text{max\_iterations} \cdot k \cdot |\text{Seeds}| \cdot \text{max\_edge\_size})$.
pub fn run_diffusion<G: PathGraph>(
    graph: &G,
    seeds: &[EntityId],
    params: &TlHfdParams,
) -> Result<AHashMap<EntityId, f32>, TlHfdError> {
    params.validate()?;

    if seeds.is_empty() {
        return Ok(AHashMap::default());
    }

    // Deduplicate seeds while preserving deterministic ordering
    let mut seeds_sorted: Vec<EntityId> = seeds.to_vec();
    seeds_sorted.sort_unstable();
    seeds_sorted.dedup();

    let seeds_set: AHashSet<EntityId> = seeds_sorted.iter().copied().collect();

    let mut active_nodes: AHashSet<EntityId> = AHashSet::with_capacity(
        seeds_sorted
            .len()
            .saturating_add(params.max_iterations as usize * params.max_top_k_expansion),
    );
    active_nodes.extend(seeds_sorted.iter().copied());

    let mut x: AHashMap<EntityId, f32> = AHashMap::with_capacity(
        seeds_sorted
            .len()
            .saturating_add(params.max_iterations as usize * params.max_top_k_expansion),
    );
    for &s in &seeds_sorted {
        x.insert(s, 0.0);
    }

    // Node degree cache
    let mut degree_cache: AHashMap<EntityId, f32> = AHashMap::new();

    let get_degree = |u: EntityId,
                      degree_cache: &mut AHashMap<EntityId, f32>,
                      graph: &G|
     -> f32 {
        *degree_cache.entry(u).or_insert_with(|| {
            let mut deg = 0.0f32;
            for (_nbr, w) in graph.neighbors_with_weights(u) {
                deg += w;
            }
            for hid in graph.hyperedges_for_entity(u) {
                if let Some(he) = graph.get_hyperedge(hid) {
                    deg += he.weight;
                }
            }
            deg.max(1e-6)
        })
    };

    let compute_lovasz_sums = |edges: &[ActiveEdge], x_map: &AHashMap<EntityId, f32>| {
        let mut sum_map: AHashMap<EntityId, f32> = AHashMap::new();
        for edge in edges {
            let truncated = truncate_participants(
                &edge.participants,
                |v| x_map.get(&v).copied().unwrap_or(0.0),
                params.max_hyperedge_sort_size,
            );

            let (f_e, u_max, u_min) = compute_lovasz_extension(&truncated, |v| {
                x_map.get(&v).copied().unwrap_or(0.0)
            });

            if f_e > 0.0 {
                if let Some(uma) = u_max {
                    *sum_map.entry(uma).or_insert(0.0) += edge.weight * f_e;
                }
                if let Some(umi) = u_min {
                    *sum_map.entry(umi).or_insert(0.0) -= edge.weight * f_e;
                }
            }
        }
        sum_map
    };

    for t in 0..params.max_iterations {
        // Collect active nodes sorted by EntityId for deterministic iteration
        let mut sorted_active: Vec<EntityId> = active_nodes.iter().copied().collect();
        sorted_active.sort_unstable();

        // Collect all unique edges touching active region A
        let mut active_edges: Vec<ActiveEdge> = Vec::new();
        let mut seen_binary: AHashSet<(EntityId, EntityId)> = AHashSet::new();
        let mut seen_hyper: AHashSet<u64> = AHashSet::new();

        for &u in &sorted_active {
            // Binary edges
            for (v, w) in graph.neighbors_with_weights(u) {
                let pair = if u < v { (u, v) } else { (v, u) };
                if seen_binary.insert(pair) {
                    active_edges.push(ActiveEdge {
                        weight: w,
                        participants: vec![pair.0, pair.1],
                    });
                }
            }

            // Hyperedges
            for hid in graph.hyperedges_for_entity(u) {
                let hid_val = hid.inner();
                if seen_hyper.insert(hid_val) {
                    let participants = match graph.get_hyperedge(hid) {
                        Some(he) => he
                            .participants
                            .iter()
                            .map(|p| p.entity)
                            .collect::<Vec<_>>(),
                        None => graph
                            .hyperedge_participants(hid)
                            .iter()
                            .map(|p| p.entity)
                            .collect::<Vec<_>>(),
                    };
                    let weight = graph.get_hyperedge(hid).map_or(1.0, |he| he.weight);
                    active_edges.push(ActiveEdge {
                        weight,
                        participants,
                    });
                }
            }
        }

        // Phase 1: Update active nodes
        let lovasz_p1 = compute_lovasz_sums(&active_edges, &x);
        let eta = 1.0 / (params.sigma * (t as f32 + 1.0));
        let mut max_delta_x = 0.0f32;

        for &u in &sorted_active {
            let d_u = get_degree(u, &mut degree_cache, graph);
            let l_u = lovasz_p1.get(&u).copied().unwrap_or(0.0);
            let xu_old = x.get(&u).copied().unwrap_or(0.0);

            let delta_u = if seeds_set.contains(&u) {
                params.delta * d_u
            } else {
                0.0
            };

            let g_u = l_u + params.sigma * d_u * xu_old - (delta_u - d_u);
            let xu_new = (xu_old - eta * (g_u / d_u)).max(0.0);

            x.insert(u, xu_new);

            let delta_x = (xu_new - xu_old).abs();
            if delta_x > max_delta_x {
                max_delta_x = delta_x;
            }
        }

        // Phase 2: Boundary Scoring for inactive neighbors (evaluated with updated x)
        let lovasz_p2 = compute_lovasz_sums(&active_edges, &x);
        let mut boundary_active_deg: AHashMap<EntityId, f32> = AHashMap::new();

        for edge in &active_edges {
            for &p in &edge.participants {
                if !active_nodes.contains(&p) {
                    *boundary_active_deg.entry(p).or_insert(0.0) += edge.weight;
                }
            }
        }

        let mut boundary_candidates: Vec<EntityId> = boundary_active_deg.keys().copied().collect();
        boundary_candidates.sort_unstable();

        let mut boundary_scores: Vec<(EntityId, f32)> = Vec::new();

        for &v in &boundary_candidates {
            let d_v = get_degree(v, &mut degree_cache, graph);
            let l_v = lovasz_p2.get(&v).copied().unwrap_or(0.0);

            // For v not in A: x_v = 0, delta_v = 0 -> g_v = L_v + d_v
            let g_v = l_v + d_v;
            let kappa_v = (-g_v / d_v).max(0.0);

            let d_in_v = boundary_active_deg.get(&v).copied().unwrap_or(0.0);
            let c_v = (d_in_v / d_v).powf(params.gamma);

            let s_v = kappa_v * c_v;
            if s_v > 0.0 {
                boundary_scores.push((v, s_v));
            }
        }

        // Phase 3: Activate Top-max_top_k_expansion boundary candidates
        boundary_scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        let to_activate_count = boundary_scores.len().min(params.max_top_k_expansion);
        let activated = &boundary_scores[..to_activate_count];

        for &(v, _s) in activated {
            active_nodes.insert(v);
            x.entry(v).or_insert(0.0);
        }

        // Check finite invariant for all active x
        for &val in x.values() {
            if !val.is_finite() {
                return Err(TlHfdError::NonFinite);
            }
        }

        // Termination condition
        if to_activate_count == 0 && max_delta_x < 1e-6 {
            break;
        }
    }

    // Filter x_u > 0.0
    let mut result = AHashMap::new();
    for (u, val) in x {
        if val > 0.0 {
            result.insert(u, val);
        }
    }

    Ok(result)
}
