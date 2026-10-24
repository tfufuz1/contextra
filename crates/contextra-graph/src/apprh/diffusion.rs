//! Core diffusion operator for Averaging-based Personalized PageRank for Hypergraphs (APPRH).

use super::error::ApprhError;
use super::params::ApprhParams;
use crate::path_rag::PathGraph;
use ahash::{AHashMap, AHashSet};
use contextra_types::EntityId;
use std::collections::VecDeque;

/// Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH).
///
/// # Determinism
/// Pushes and neighbor iterations are strictly ordered by [`EntityId`] to guarantee bit-identical
/// floating-point calculation across runs and seed order permutations.
///
/// # P24 Locality
/// Runtime and visited set size scale with the push budget $O(1 / (\alpha \cdot \epsilon))$,
/// independent of total graph size $|V| + |E|$.
///
/// # Errors
/// Returns [`ApprhError`] if parameters are invalid, seed list is empty, or non-finite float is produced.
pub fn run_apprh_push<G: PathGraph>(
    graph: &G,
    seeds: &[EntityId],
    params: &ApprhParams,
) -> Result<AHashMap<EntityId, f32>, ApprhError> {
    params.validate()?;

    if seeds.is_empty() {
        return Err(ApprhError::EmptySeeds);
    }

    // Sort and deduplicate seeds strictly by EntityId to ensure seed order invariance
    let mut sorted_seeds = seeds.to_vec();
    sorted_seeds.sort();
    sorted_seeds.dedup();

    let seed_count = sorted_seeds.len() as f32;
    let initial_residual = 1.0 / seed_count;

    if !initial_residual.is_finite() {
        return Err(ApprhError::NonFinite);
    }

    let mut p: AHashMap<EntityId, f32> = AHashMap::default();
    let mut r: AHashMap<EntityId, f32> = AHashMap::default();
    let mut queue: VecDeque<EntityId> = VecDeque::new();
    let mut in_queue: AHashSet<EntityId> = AHashSet::default();

    for &s in &sorted_seeds {
        r.insert(s, initial_residual);
        if in_queue.insert(s) {
            queue.push_back(s);
        }
    }

    let alpha = params.ppr.alpha;
    let epsilon = params.ppr.epsilon;
    let beta = params.hyperedge_decay_factor;
    let max_iterations = params.max_iterations;

    let mut steps = 0u32;

    while let Some(u) = queue.pop_front() {
        in_queue.remove(&u);
        steps += 1;
        if steps > max_iterations {
            break;
        }

        // 1. Collect and sort binary neighbors by EntityId and weight bit-pattern
        let mut binary_neighbors = graph.neighbors_with_weights(u);
        binary_neighbors.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.to_bits().cmp(&b.1.to_bits()))
        });
        binary_neighbors.dedup_by_key(|(v, _)| *v);

        // 2. Collect and sort hyperedges by HyperEdgeId
        let mut hedge_ids = graph.hyperedges_for_entity(u);
        hedge_ids.sort();
        hedge_ids.dedup();

        // 3. In APPRH, each hyperedge counts as 1 degree unit
        let binary_degree = binary_neighbors.len();
        let hyper_degree = hedge_ids.len();
        let degree = ((binary_degree + hyper_degree) as f32).max(1.0);

        let r_u = *r.get(&u).unwrap_or(&0.0);
        if !r_u.is_finite() {
            return Err(ApprhError::NonFinite);
        }

        if r_u / degree <= epsilon {
            continue;
        }

        // Add teleported rank share to p[u]
        let page_rank_share = alpha * r_u;
        if !page_rank_share.is_finite() {
            return Err(ApprhError::NonFinite);
        }
        *p.entry(u).or_insert(0.0) += page_rank_share;
        r.insert(u, 0.0);

        let push_mass = (1.0 - alpha) * r_u;
        if push_mass <= 0.0 {
            continue;
        }

        let base_push_share = push_mass / degree;

        // Track new nodes to push to queue in deterministic sorted EntityId order
        let mut new_push_nodes = Vec::new();

        // 4. Binary neighbor push
        for (v, _w) in binary_neighbors {
            let entry = r.entry(v).or_insert(0.0);
            *entry += base_push_share;
            // // NAN-CHECK-OK: Validate finite residual increment during APPRH binary neighbor push
            if !entry.is_finite() {
                return Err(ApprhError::NonFinite);
            }
            if !in_queue.contains(&v) {
                new_push_nodes.push(v);
            }
        }

        // 5. Averaging-based hyperedge push
        for hedge_id in hedge_ids {
            let participants = graph.hyperedge_participants(hedge_id);
            let mut other_participants: Vec<EntityId> = participants
                .iter()
                .map(|b| b.entity)
                .filter(|&e| e != u)
                .collect();
            other_participants.sort();
            other_participants.dedup();

            if !other_participants.is_empty() {
                let p_count = other_participants.len() as f32;
                let avg_participant_share = (base_push_share * beta) / p_count;

                for v in other_participants {
                    let entry = r.entry(v).or_insert(0.0);
                    *entry += avg_participant_share;
                    if !entry.is_finite() {
                        return Err(ApprhError::NonFinite);
                    }
                    if !in_queue.contains(&v) {
                        new_push_nodes.push(v);
                    }
                }
            }
        }

        // Deterministically sort newly activated nodes before enqueuing
        new_push_nodes.sort();
        new_push_nodes.dedup();
        for v in new_push_nodes {
            if in_queue.insert(v) {
                queue.push_back(v);
            }
        }
    }

    // Verify all scores in p are finite
    for &val in p.values() {
        if !val.is_finite() {
            return Err(ApprhError::NonFinite);
        }
    }

    Ok(p)
}
