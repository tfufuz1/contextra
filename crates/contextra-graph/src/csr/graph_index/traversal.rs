use std::collections::{HashMap, VecDeque};

use contextra_ports::BoxFuture;
use contextra_types::{EntityId, Result, TxId};

use super::super::graph_write::CsrGraph;
use super::super::inner::GraphInner;
use super::super::types::{InternalIndex, MAX_TRAVERSAL_HOPS, MAX_VISITED_NODES, SCORE_DECAY};
use super::super::visibility::is_edge_visible_bitemporal;

pub(crate) fn traverse_at_bitemporal_impl<'a>(
    graph: &'a CsrGraph,
    start: EntityId,
    max_hops: usize,
    as_of_tx: TxId,
    as_of_business: Option<i64>,
) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
    Box::pin(async move {
        super::validate_traversal_params(max_hops)?;
        let inner = graph.inner_read();
        let start_idx = match inner.id_map.get(&start) {
            Some(&idx) => idx,
            None => return Ok(Vec::new()),
        };

        if inner.entity_at(start_idx).is_none() {
            return Ok(Vec::new());
        }

        let effective_max = (max_hops.min(255) as u8).min(MAX_TRAVERSAL_HOPS);
        let visited =
            execute_bfs_bitemporal(&inner, start_idx, effective_max, as_of_tx, as_of_business);
        Ok(collect_and_sort_traversal_results(
            &inner, start_idx, visited,
        ))
    })
}

pub(crate) fn traverse_impl<'a>(
    graph: &'a CsrGraph,
    start: EntityId,
    max_hops: usize,
) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
    Box::pin(async move {
        super::validate_traversal_params(max_hops)?;
        let inner = graph.inner_read();
        let start_idx = match inner.id_map.get(&start) {
            Some(&idx) => idx,
            None => return Ok(Vec::new()),
        };

        if inner.entity_at(start_idx).is_none() {
            return Ok(Vec::new());
        }

        let effective_max = (max_hops.min(255) as u8).min(MAX_TRAVERSAL_HOPS);
        let visited = execute_bfs_unconditional(&inner, start_idx, effective_max);
        Ok(collect_and_sort_traversal_results(
            &inner, start_idx, visited,
        ))
    })
}

fn collect_and_sort_traversal_results(
    inner: &GraphInner,
    start_idx: InternalIndex,
    mut visited: HashMap<InternalIndex, f32>,
) -> Vec<(EntityId, f32)> {
    visited.remove(&start_idx);
    let mut results: Vec<(EntityId, f32)> = visited
        .into_iter()
        .filter_map(|(idx, score)| inner.reverse_map.get(idx).map(|&id| (id, score)))
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

#[derive(Clone, Copy)]
struct BitemporalBfsArgs<'a> {
    inner: &'a GraphInner,
    as_of_tx: TxId,
    as_of_business: Option<i64>,
}

fn process_bitemporal_node_edges(
    args: BitemporalBfsArgs<'_>,
    node_idx: InternalIndex,
    current_score: f32,
    hop: u8,
    visited: &mut HashMap<InternalIndex, f32>,
    queue: &mut VecDeque<(InternalIndex, u8, f32)>,
) {
    let inner = args.inner;
    if node_idx < inner.offsets.len() - 1 {
        for edge_idx in inner.offsets[node_idx]..inner.offsets[node_idx + 1] {
            let neighbor_idx = inner.targets[edge_idx];
            if inner.tombstoned_edges.contains(&(node_idx, neighbor_idx)) {
                continue;
            }
            if !is_edge_visible_bitemporal(
                inner.tx_valid_from_at(edge_idx),
                inner.tx_valid_to_at(edge_idx),
                args.as_of_tx,
                inner.business_valid_from_at(edge_idx),
                inner.business_valid_to_at(edge_idx),
                args.as_of_business,
            ) {
                continue;
            }
            let next_score = current_score * SCORE_DECAY * inner.weights[edge_idx];
            if (!visited.contains_key(&neighbor_idx) || visited[&neighbor_idx] < next_score)
                && inner.entity_at(neighbor_idx).is_some()
            {
                if !visited.contains_key(&neighbor_idx)
                    && visited.len() + queue.len() >= MAX_VISITED_NODES
                {
                    break;
                }
                queue.push_back((neighbor_idx, hop + 1, next_score));
            }
        }
    }

    if let Some(pending) = inner.pending_edges.get(&node_idx) {
        for edge in pending {
            let neighbor_idx = edge.target;
            if inner.tombstoned_edges.contains(&(node_idx, neighbor_idx)) {
                continue;
            }
            if !is_edge_visible_bitemporal(
                edge.tx_valid_from,
                edge.tx_valid_to,
                args.as_of_tx,
                edge.business_valid_from,
                edge.business_valid_to,
                args.as_of_business,
            ) {
                continue;
            }
            let next_score = current_score * SCORE_DECAY * edge.weight;
            if (!visited.contains_key(&neighbor_idx) || visited[&neighbor_idx] < next_score)
                && inner.entity_at(neighbor_idx).is_some()
            {
                if !visited.contains_key(&neighbor_idx)
                    && visited.len() + queue.len() >= MAX_VISITED_NODES
                {
                    break;
                }
                queue.push_back((neighbor_idx, hop + 1, next_score));
            }
        }
    }
}

fn execute_bfs_bitemporal(
    inner: &GraphInner,
    start_idx: InternalIndex,
    effective_max: u8,
    as_of_tx: TxId,
    as_of_business: Option<i64>,
) -> HashMap<InternalIndex, f32> {
    let mut visited: HashMap<InternalIndex, f32> = HashMap::new();
    let mut queue: VecDeque<(InternalIndex, u8, f32)> = VecDeque::new();
    queue.push_back((start_idx, 0, 1.0));

    let bfs_args = BitemporalBfsArgs {
        inner,
        as_of_tx,
        as_of_business,
    };

    while let Some((node_idx, hop, current_score)) = queue.pop_front() {
        if hop > effective_max {
            continue;
        }

        let existing = visited.entry(node_idx).or_insert(0.0);
        if current_score > *existing {
            *existing = current_score;
        }

        if hop < effective_max {
            if visited.len() >= MAX_VISITED_NODES {
                break;
            }
            process_bitemporal_node_edges(
                bfs_args,
                node_idx,
                current_score,
                hop,
                &mut visited,
                &mut queue,
            );
        }
    }

    visited
}

fn process_unconditional_node_edges(
    inner: &GraphInner,
    node_idx: InternalIndex,
    current_score: f32,
    hop: u8,
    visited: &mut HashMap<InternalIndex, f32>,
    queue: &mut VecDeque<(InternalIndex, u8, f32)>,
) {
    if node_idx < inner.offsets.len() - 1 {
        for edge_idx in inner.offsets[node_idx]..inner.offsets[node_idx + 1] {
            let neighbor_idx = inner.targets[edge_idx];
            if inner.tombstoned_edges.contains(&(node_idx, neighbor_idx)) {
                continue;
            }
            let next_score = current_score * SCORE_DECAY * inner.weights[edge_idx];
            if (!visited.contains_key(&neighbor_idx) || visited[&neighbor_idx] < next_score)
                && inner.entity_at(neighbor_idx).is_some()
            {
                if !visited.contains_key(&neighbor_idx)
                    && visited.len() + queue.len() >= MAX_VISITED_NODES
                {
                    break;
                }
                queue.push_back((neighbor_idx, hop + 1, next_score));
            }
        }
    }

    if let Some(pending) = inner.pending_edges.get(&node_idx) {
        for edge in pending {
            let neighbor_idx = edge.target;
            if inner.tombstoned_edges.contains(&(node_idx, neighbor_idx)) {
                continue;
            }
            let next_score = current_score * SCORE_DECAY * edge.weight;
            if (!visited.contains_key(&neighbor_idx) || visited[&neighbor_idx] < next_score)
                && inner.entity_at(neighbor_idx).is_some()
            {
                if !visited.contains_key(&neighbor_idx)
                    && visited.len() + queue.len() >= MAX_VISITED_NODES
                {
                    break;
                }
                queue.push_back((neighbor_idx, hop + 1, next_score));
            }
        }
    }
}

fn execute_bfs_unconditional(
    inner: &GraphInner,
    start_idx: InternalIndex,
    effective_max: u8,
) -> HashMap<InternalIndex, f32> {
    let mut visited: HashMap<InternalIndex, f32> = HashMap::new();
    let mut queue: VecDeque<(InternalIndex, u8, f32)> = VecDeque::new();
    queue.push_back((start_idx, 0, 1.0));

    while let Some((node_idx, hop, current_score)) = queue.pop_front() {
        if hop > effective_max {
            continue;
        }

        let existing = visited.entry(node_idx).or_insert(0.0);
        if current_score > *existing {
            *existing = current_score;
        }

        if hop < effective_max {
            if visited.len() >= MAX_VISITED_NODES {
                break;
            }
            process_unconditional_node_edges(
                inner,
                node_idx,
                current_score,
                hop,
                &mut visited,
                &mut queue,
            );
        }
    }

    visited
}
