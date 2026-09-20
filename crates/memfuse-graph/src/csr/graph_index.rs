use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::{Mutex, RwLock};
use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};

use memfuse_core::{
    BoxFuture, DocId, Entity, EntityId, GraphIndex, GraphIndexStats, MemFuseError, Result,
    StorageEngine, TxId,
};
use crate::consistency_enforcement::{ConsistencyEnforcer, EdgeAssertion};
use crate::GraphIndexExt;

use super::types::{Edge, EdgePayload, EdgeType, InternalIndex, PersistedEdgePayload, StagedEdgePayload, MAX_TRAVERSAL_HOPS, MAX_VISITED_NODES, SCORE_DECAY, GRAPH_ENTITY_PREFIX, GRAPH_EDGE_PREFIX, GRAPH_ENTITY_DELETED_PREFIX, GRAPH_COMMUNITY_PREFIX};
use super::inner::{sentinel_entity, GraphInner, InnerWriteGuard};
use super::visibility::{is_edge_visible, is_edge_visible_bitemporal, is_edge_visible_business, is_suspicious_tx_id};
use super::graph_write::CsrGraph;


impl GraphIndexExt for CsrGraph {
    fn remove_entity<'a>(&'a self, tx: TxId, entity: EntityId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let (target_idx, outgoing_targets, incoming_sources) = {
                let inner = self.inner_read();
                let idx = match inner.id_map.get(&entity) {
                    Some(&i) => i,
                    None => return Ok(()),
                };

                let mut outgoing = Vec::new();
                let mut incoming = Vec::new();

                // a. Outgoing edges (entity -> *)
                if idx < inner.offsets.len() - 1 {
                    for j in inner.offsets[idx]..inner.offsets[idx + 1] {
                        let t_idx = inner.targets[j];
                        if !inner.tombstoned_edges.contains(&(idx, t_idx)) {
                            if let Some(&t_id) = inner.reverse_map.get(t_idx) {
                                outgoing.push(t_id);
                            }
                        }
                    }
                }
                if let Some(pending) = inner.pending_edges.get(&idx) {
                    for edge in pending {
                        if !inner.tombstoned_edges.contains(&(idx, edge.target)) {
                            if let Some(&t_id) = inner.reverse_map.get(edge.target) {
                                outgoing.push(t_id);
                            }
                        }
                    }
                }

                // b. Incoming edges (* -> entity)
                let num_nodes = inner.reverse_map.len();
                for node_idx in 0..num_nodes {
                    let source_id = match inner.reverse_map.get(node_idx) {
                        Some(&id) => id,
                        None => continue,
                    };

                    if node_idx < inner.offsets.len() - 1 {
                        for j in inner.offsets[node_idx]..inner.offsets[node_idx + 1] {
                            if inner.targets[j] == idx
                                && !inner.tombstoned_edges.contains(&(node_idx, idx))
                            {
                                incoming.push(source_id);
                            }
                        }
                    }
                    if let Some(pending) = inner.pending_edges.get(&node_idx) {
                        for edge in pending {
                            if edge.target == idx
                                && !inner.tombstoned_edges.contains(&(node_idx, idx))
                            {
                                incoming.push(source_id);
                            }
                        }
                    }
                }

                (idx, outgoing, incoming)
            };

            // Tombstone outgoing edges (entity -> to)
            for to_id in outgoing_targets {
                GraphIndex::remove_edge(self, tx, entity, to_id).await?;
            }

            // Tombstone incoming edges (from -> entity)
            for from_id in incoming_sources {
                GraphIndex::remove_edge(self, tx, from_id, entity).await?;
            }

            // c. Den Knoten selbst aus inner.id_map entfernen
            {
                let mut inner = self.inner_write();
                inner.id_map.remove(&entity);
                if target_idx < inner.entities.len() {
                    inner.entities[target_idx] = sentinel_entity();
                }
                inner.communities.remove(&entity);
                inner.is_dirty = true;
            }

            // LSM-Storage Marker / Deletion schreiben
            if let Some(ref storage) = self.storage {
                let entity_key = [GRAPH_ENTITY_PREFIX, entity.as_bytes().as_slice()].concat();
                storage.delete(tx, &entity_key).await?;

                let deleted_key =
                    [GRAPH_ENTITY_DELETED_PREFIX, entity.as_bytes().as_slice()].concat();
                storage
                    .put(tx, &deleted_key, &tx.inner().to_le_bytes())
                    .await?;
            }

            // d. Committe die Änderungen
            GraphIndex::commit(self, tx).await?;

            Ok(())
        })
    }
}

impl Default for CsrGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphIndex for CsrGraph {
    fn add_entity<'a>(&'a self, tx: TxId, entity: Entity) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            debug_assert!(
            tx != TxId::INVALID && tx.is_valid_origin(),
            "TxId {} verletzt AGT-GRAPH-001 Origin-Invariante — Sentinel TxId(0) oder Wall-Clock-abgeleitete IDs korrumpieren rollback_to_tx()-Kausalordnung",
            tx
        );
            // AGT-GRAPH-001: Heuristik — wall-clock-abgeleitete oder unallozierte TxIds warnen.
            if is_suspicious_tx_id(tx) {
                tracing::warn!(
                tx_id = tx.inner(),
                hint = if tx == TxId::INVALID { "Sentinel TxId(0)" } else { "Wall-Clock-ns-Bereich" },
                "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId in add_entity (weder im plausiblen next_tx-Bereich noch im INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]) — \
                 möglicherweise unalloziert oder aus Wall-Clock-Nanosekunden abgeleitet. \
                 Rollback-Korrelation kann verletzt sein."
            );
            }
            // Lazy index allocation: Entity indices are assigned in commit(),
            // avoiding premature mutation of id_map/reverse_map on rollback.
            {
                let mut inner = self.inner_write();
                inner
                    .staged_entities
                    .insert((tx, entity.id), entity.clone());
            }

            if let Some(ref storage) = self.storage {
                self.persist_entity(storage.as_ref(), tx, &entity).await?;
            }
            Ok(())
        })
    }

    fn add_edge<'a>(&'a self, tx: TxId, edge: memfuse_core::Edge) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            debug_assert!(
            tx != TxId::INVALID && tx.is_valid_origin(),
            "TxId {} verletzt AGT-GRAPH-001 Origin-Invariante — Sentinel TxId(0) oder Wall-Clock-abgeleitete IDs korrumpieren rollback_to_tx()-Kausalordnung",
            tx
        );
            // AGT-GRAPH-001: Heuristik — wall-clock-abgeleitete oder unallozierte TxIds warnen.
            if is_suspicious_tx_id(tx) {
                tracing::warn!(
                tx_id = tx.inner(),
                hint = if tx == TxId::INVALID { "Sentinel TxId(0)" } else { "Wall-Clock-ns-Bereich" },
                "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId in add_edge (weder im plausiblen next_tx-Bereich noch im INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]) — \
                 möglicherweise unalloziert oder aus Wall-Clock-Nanosekunden abgeleitet. \
                 Rollback-Korrelation kann verletzt sein."
            );
            }
            if !edge.weight.is_finite() || edge.weight < 0.0 {
                return Err(MemFuseError::InvalidInput(format!(
                    "Invalid edge weight {}: weight must be finite and non-negative",
                    edge.weight
                )));
            }

            // Consistency-Check wenn aktiviert
            if let Some(ref enforcer_lock) = self.consistency_enforcer {
                let pred_hash = *blake3::hash(edge.label.as_bytes()).as_bytes();
                let assertion = EdgeAssertion {
                    subject: edge.from.inner(),
                    predicate_hash: pred_hash,
                    object_repr: edge.to.as_bytes(),
                };
                let mut enforcer = enforcer_lock.write();
                if let Some(pattern) = enforcer.check_before_insert(&assertion) {
                    if pattern.suppressed {
                        tracing::warn!(
                            suppression_count = pattern.contradiction_count,
                            "ConsistencyEnforcer: edge insertion suppressed by conflict pattern"
                        );
                        return Err(MemFuseError::PolicyViolation(
                            "Contradictory edge suppressed by consistency enforcer".to_string(),
                        ));
                    }
                    tracing::warn!(
                        "ConsistencyEnforcer: contradictory edge detected (not yet suppressed)"
                    );
                }
            }

            let tx_valid_from = edge.tx_valid_from.or(Some(tx));

            // Register source document provenance for cascading invalidation
            if let Some(doc_id) = edge.source_doc_id {
                self.doc_edge_index.record(doc_id, (edge.from, edge.to));
            }

            // Lazy index allocation: Store EntityIds directly in staged_edges.
            // Internal indices via get_or_create_index are allocated only during commit(),
            // ensuring rollback does not leak entity indices into id_map/reverse_map.
            {
                let mut inner = self.inner_write();
                inner
                    .staged_edges
                    .entry((tx, edge.from))
                    .or_default()
                    .push(StagedEdgePayload {
                        target: edge.to,
                        weight: edge.weight,
                        tx_valid_from,
                        tx_valid_to: edge.tx_valid_to,
                        business_valid_from: edge.business_valid_from,
                        business_valid_to: edge.business_valid_to,
                        source_doc_id: edge.source_doc_id,
                    });
            }

            if let Some(ref storage) = self.storage {
                let payload = PersistedEdgePayload {
                    weight: edge.weight,
                    tx_valid_from,
                    tx_valid_to: edge.tx_valid_to,
                    business_valid_from: edge.business_valid_from,
                    business_valid_to: edge.business_valid_to,
                    source_doc_id: edge.source_doc_id,
                };
                self.persist_edge(storage.as_ref(), tx, &edge.from, &edge.to, &payload)
                    .await?;
            }
            Ok(())
        })
    }

    fn personalized_page_rank<'a>(
        &'a self,
        seed_nodes: &'a [EntityId],
        config: &'a memfuse_core::PprConfig,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let deleted_view = self.deleted_view().await;
            // Befund 2.1: Call compact_async() to prevent Tokio runtime stalls during O(V+E) rebuilds.
            // NOTE: Intermediate mitigation; exclusive write-lock scoping remains open for IP-08.
            if let Err(err) = self.compact_async().await {
                tracing::warn!(
                    error = %err,
                    "compact_async failed during personalized_page_rank compaction"
                );
            }
            let inner = self.inner_read();
            Ok(crate::ppr::compute_ppr(
                &inner,
                seed_nodes,
                config,
                &deleted_view,
            ))
        })
    }

    fn traverse_at<'a>(
        &'a self,
        start_node: EntityId,
        max_hops: usize,
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            self.traverse_at_time(start_node, max_hops, TxId::new(seq_no))
                .await
        })
    }

    fn traverse_at_time<'a>(
        &'a self,
        start: EntityId,
        max_hops: usize,
        as_of: TxId,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            self.traverse_at_bitemporal(start, max_hops, as_of, None)
                .await
        })
    }

    fn traverse_at_bitemporal<'a>(
        &'a self,
        start: EntityId,
        max_hops: usize,
        as_of_tx: TxId,
        as_of_business: Option<i64>,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            if max_hops > 100 {
                return Err(MemFuseError::InvalidInput(format!(
                    "max_hops {max_hops} exceeds upper safety limit of 100"
                )));
            }
            if max_hops > MAX_TRAVERSAL_HOPS as usize {
                tracing::warn!(
                requested_max_hops = max_hops,
                effective_max_hops = MAX_TRAVERSAL_HOPS,
                "traverse_at_bitemporal requested max_hops ({max_hops}) exceeds internal cap MAX_TRAVERSAL_HOPS ({MAX_TRAVERSAL_HOPS}); capping traversal depth"
            );
            }
            let inner = self.inner_read();
            let start_idx = match inner.id_map.get(&start) {
                Some(&idx) => idx,
                None => return Ok(Vec::new()),
            };

            if inner.entity_at(start_idx).is_none() {
                return Ok(Vec::new());
            }

            let effective_max = (max_hops as u8).min(MAX_TRAVERSAL_HOPS);

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
                        tracing::warn!(
                        visited_count = visited.len(),
                        max_visited = MAX_VISITED_NODES,
                        "traverse_at_bitemporal visited node limit reached ({MAX_VISITED_NODES}); halting graph expansion"
                    );
                        break;
                    }

                    // 1. CSR traversal (compacted edges)
                    if node_idx < inner.offsets.len() - 1 {
                        let start_edge = inner.offsets[node_idx];
                        let end_edge = inner.offsets[node_idx + 1];

                        for edge_idx in start_edge..end_edge {
                            let neighbor_idx = inner.targets[edge_idx];
                            if inner.tombstoned_edges.contains(&(node_idx, neighbor_idx)) {
                                continue;
                            }
                            let tx_valid_from = inner.tx_valid_from_at(edge_idx);
                            let tx_valid_to = inner.tx_valid_to_at(edge_idx);
                            let business_valid_from = inner.business_valid_from_at(edge_idx);
                            let business_valid_to = inner.business_valid_to_at(edge_idx);

                            if !is_edge_visible_bitemporal(
                                tx_valid_from,
                                tx_valid_to,
                                as_of_tx,
                                business_valid_from,
                                business_valid_to,
                                as_of_business,
                            ) {
                                continue;
                            }
                            let weight = inner.weights[edge_idx];
                            let next_score = current_score * SCORE_DECAY * weight;

                            if (!visited.contains_key(&neighbor_idx)
                                || visited[&neighbor_idx] < next_score)
                                && inner.entity_at(neighbor_idx).is_some()
                            {
                                if !visited.contains_key(&neighbor_idx)
                                    && visited.len() + queue.len() >= MAX_VISITED_NODES
                                {
                                    tracing::warn!(
                                    visited_and_queued = visited.len() + queue.len(),
                                    max_visited = MAX_VISITED_NODES,
                                    "traverse_at_bitemporal visited node limit reached ({MAX_VISITED_NODES}); halting neighbor expansion"
                                );
                                    break;
                                }
                                queue.push_back((neighbor_idx, hop + 1, next_score));
                            }
                        }
                    }

                    // 2. Delta buffer traversal (uncompacted committed edges)
                    if let Some(pending) = inner.pending_edges.get(&node_idx) {
                        for edge in pending {
                            let neighbor_idx = edge.target;
                            if inner.tombstoned_edges.contains(&(node_idx, neighbor_idx)) {
                                continue;
                            }
                            if !is_edge_visible_bitemporal(
                                edge.tx_valid_from,
                                edge.tx_valid_to,
                                as_of_tx,
                                edge.business_valid_from,
                                edge.business_valid_to,
                                as_of_business,
                            ) {
                                continue;
                            }
                            let next_score = current_score * SCORE_DECAY * edge.weight;

                            if (!visited.contains_key(&neighbor_idx)
                                || visited[&neighbor_idx] < next_score)
                                && inner.entity_at(neighbor_idx).is_some()
                            {
                                if !visited.contains_key(&neighbor_idx)
                                    && visited.len() + queue.len() >= MAX_VISITED_NODES
                                {
                                    tracing::warn!(
                                    visited_and_queued = visited.len() + queue.len(),
                                    max_visited = MAX_VISITED_NODES,
                                    "traverse_at_bitemporal visited node limit reached ({MAX_VISITED_NODES}); halting neighbor expansion"
                                );
                                    break;
                                }
                                queue.push_back((neighbor_idx, hop + 1, next_score));
                            }
                        }
                    }
                }
            }

            visited.remove(&start_idx);

            let mut results: Vec<(EntityId, f32)> = visited
                .into_iter()
                .filter_map(|(idx, score)| inner.reverse_map.get(idx).map(|&id| (id, score)))
                .collect();

            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            Ok(results)
        })
    }

    fn traverse<'a>(
        &'a self,
        start: EntityId,
        max_hops: usize,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            if max_hops > 100 {
                return Err(MemFuseError::InvalidInput(format!(
                    "max_hops {max_hops} exceeds upper safety limit of 100"
                )));
            }
            if max_hops > MAX_TRAVERSAL_HOPS as usize {
                tracing::warn!(
                requested_max_hops = max_hops,
                effective_max_hops = MAX_TRAVERSAL_HOPS,
                "traverse requested max_hops ({max_hops}) exceeds internal cap MAX_TRAVERSAL_HOPS ({MAX_TRAVERSAL_HOPS}); capping traversal depth"
            );
            }

            // Merge-read: read directly from both compacted CSR arrays AND uncompacted pending_edges delta buffer.
            // No full compact() call is required before traversal.
            let inner = self.inner_read();
            let start_idx = match inner.id_map.get(&start) {
                Some(&idx) => idx,
                None => return Ok(Vec::new()), // Start node not in graph
            };

            // If the start node itself is not committed, we shouldn't start traversal from it
            if inner.entity_at(start_idx).is_none() {
                return Ok(Vec::new());
            }

            let effective_max = (max_hops as u8).min(MAX_TRAVERSAL_HOPS);

            // BFS with score decay
            let mut visited: HashMap<InternalIndex, f32> = HashMap::new();
            let mut queue: VecDeque<(InternalIndex, u8, f32)> = VecDeque::new();

            queue.push_back((start_idx, 0, 1.0));

            while let Some((node_idx, hop, current_score)) = queue.pop_front() {
                if hop > effective_max {
                    continue;
                }

                // Only keep the best score per node
                let existing = visited.entry(node_idx).or_insert(0.0);
                if current_score > *existing {
                    *existing = current_score;
                }

                if hop < effective_max {
                    if visited.len() >= MAX_VISITED_NODES {
                        tracing::warn!(
                        visited_count = visited.len(),
                        max_visited = MAX_VISITED_NODES,
                        "traverse visited node limit reached ({MAX_VISITED_NODES}); halting graph expansion"
                    );
                        break;
                    }

                    // 1. CSR traversal (compacted edges)
                    if node_idx < inner.offsets.len() - 1 {
                        let start_edge = inner.offsets[node_idx];
                        let end_edge = inner.offsets[node_idx + 1];

                        for edge_idx in start_edge..end_edge {
                            let neighbor_idx = inner.targets[edge_idx];
                            if inner.tombstoned_edges.contains(&(node_idx, neighbor_idx)) {
                                continue;
                            }
                            let weight = inner.weights[edge_idx];
                            let next_score = current_score * SCORE_DECAY * weight;

                            if !visited.contains_key(&neighbor_idx)
                                || visited[&neighbor_idx] < next_score
                            {
                                // Only visit nodes that have a committed entity (FIND-GRA-001)
                                if inner.entity_at(neighbor_idx).is_some() {
                                    if !visited.contains_key(&neighbor_idx)
                                        && visited.len() + queue.len() >= MAX_VISITED_NODES
                                    {
                                        tracing::warn!(
                                        visited_and_queued = visited.len() + queue.len(),
                                        max_visited = MAX_VISITED_NODES,
                                        "traverse visited node limit reached ({MAX_VISITED_NODES}); halting neighbor expansion"
                                    );
                                        break;
                                    }
                                    queue.push_back((neighbor_idx, hop + 1, next_score));
                                }
                            }
                        }
                    }

                    // 2. Delta buffer traversal (uncompacted committed edges)
                    if let Some(pending) = inner.pending_edges.get(&node_idx) {
                        for edge in pending {
                            let neighbor_idx = edge.target;
                            if inner.tombstoned_edges.contains(&(node_idx, neighbor_idx)) {
                                continue;
                            }
                            let next_score = current_score * SCORE_DECAY * edge.weight;

                            if (!visited.contains_key(&neighbor_idx)
                                || visited[&neighbor_idx] < next_score)
                                && inner.entity_at(neighbor_idx).is_some()
                            {
                                if !visited.contains_key(&neighbor_idx)
                                    && visited.len() + queue.len() >= MAX_VISITED_NODES
                                {
                                    tracing::warn!(
                                    visited_and_queued = visited.len() + queue.len(),
                                    max_visited = MAX_VISITED_NODES,
                                    "traverse visited node limit reached ({MAX_VISITED_NODES}); halting neighbor expansion"
                                );
                                    break;
                                }
                                queue.push_back((neighbor_idx, hop + 1, next_score));
                            }
                        }
                    }
                }
            }

            // Remove the start node from results
            visited.remove(&start_idx);

            let mut results: Vec<(EntityId, f32)> = visited
                .into_iter()
                .filter_map(|(idx, score)| inner.reverse_map.get(idx).map(|&id| (id, score)))
                .collect();

            // Sort by score descending
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            Ok(results)
        })
    }

    fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            debug_assert!(
            tx != TxId::INVALID && tx.is_valid_origin(),
            "TxId {} verletzt AGT-GRAPH-001 Origin-Invariante — Sentinel TxId(0) oder Wall-Clock-abgeleitete IDs korrumpieren rollback_to_tx()-Kausalordnung",
            tx
        );
            // AGT-GRAPH-001: Heuristik — wall-clock-abgeleitete oder unallozierte TxIds warnen.
            if is_suspicious_tx_id(tx) {
                tracing::warn!(
                tx_id = tx.inner(),
                hint = if tx == TxId::INVALID { "Sentinel TxId(0)" } else { "Wall-Clock-ns-Bereich" },
                "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId in commit (weder im plausiblen next_tx-Bereich noch im INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]) — \
                 möglicherweise unalloziert oder aus Wall-Clock-Nanosekunden abgeleitet. \
                 Rollback-Korrelation kann verletzt sein."
            );
            }

            let mut inner = self.inner_write();

            // 1. Commit entities
            let mut tx_entities = Vec::new();
            inner.staged_entities.retain(|&(t, id), entity| {
                if t == tx {
                    tx_entities.push((id, entity.clone()));
                    false
                } else {
                    true
                }
            });
            if !tx_entities.is_empty() {
                tx_entities.sort_by_key(|(id, _)| *id);
                for (id, entity) in tx_entities {
                    let idx = inner.get_or_create_index(id);
                    if idx >= inner.entities.len() {
                        inner.entities.resize(idx + 1, sentinel_entity());
                    }
                    inner.entities[idx] = entity;
                }
                inner.is_dirty = true;
            }

            // 2. Commit edges (lazy index resolution occurs here)
            let mut tx_edges = Vec::new();
            inner.staged_edges.retain(|&(t, from_id), edges| {
                if t == tx {
                    tx_edges.push((from_id, std::mem::take(edges)));
                    false
                } else {
                    true
                }
            });
            if !tx_edges.is_empty() {
                tx_edges.sort_by_key(|(from_id, _)| *from_id);
                for (from_id, edges) in tx_edges {
                    let from_idx = inner.get_or_create_index(from_id);
                    let mut converted_edges = Vec::with_capacity(edges.len());
                    for edge in edges {
                        let to_idx = inner.get_or_create_index(edge.target);
                        if let Some(doc_id) = edge.source_doc_id {
                            inner
                                .doc_to_edges
                                .entry(doc_id)
                                .or_default()
                                .insert((from_id, edge.target));
                        }
                        converted_edges.push(EdgePayload {
                            target: to_idx,
                            weight: edge.weight,
                            tx_valid_from: edge.tx_valid_from,
                            tx_valid_to: edge.tx_valid_to,
                            business_valid_from: edge.business_valid_from,
                            business_valid_to: edge.business_valid_to,
                            source_doc_id: edge.source_doc_id,
                        });
                    }
                    for edge in &converted_edges {
                        inner.add_to_out_weight_sum(from_idx, edge.weight);
                    }
                    let count = converted_edges.len();
                    inner
                        .pending_edges
                        .entry(from_idx)
                        .or_default()
                        .extend(converted_edges);
                    inner.pending_edge_count += count;
                }
                inner.is_dirty = true;
            }

            // 3. Commit removals
            if let Some(tx_removals) = inner.staged_removals.remove(&tx) {
                for (from_id, to_id) in tx_removals {
                    let from_idx = inner.id_map.get(&from_id).copied();
                    let to_idx = inner.id_map.get(&to_id).copied();
                    if let (Some(f_idx), Some(t_idx)) = (from_idx, to_idx) {
                        if let Some(pending) = inner.pending_edges.get_mut(&f_idx) {
                            pending.retain(|edge| edge.target != t_idx);
                        }
                        inner.tombstoned_edges.insert((f_idx, t_idx));
                        inner.is_dirty = true;
                        inner.recompute_node_out_weight_sum(f_idx);
                    }
                }
            }

            // Auto-rebuild CSR arrays if pending delta buffer reaches or exceeds threshold
            if inner.pending_edge_count >= self.config.rebuild_threshold {
                inner.compact();
            }

            self.last_tx_id.fetch_max(tx.inner(), Ordering::SeqCst);

            Ok(())
        })
    }

    fn remove_edge<'a>(
        &'a self,
        tx: TxId,
        from: EntityId,
        to: EntityId,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            debug_assert!(
            tx != TxId::INVALID && tx.is_valid_origin(),
            "TxId {} verletzt AGT-GRAPH-001 Origin-Invariante — Sentinel TxId(0) oder Wall-Clock-abgeleitete IDs korrumpieren rollback_to_tx()-Kausalordnung",
            tx
        );
            if is_suspicious_tx_id(tx) {
                tracing::warn!(
                tx_id = tx.inner(),
                hint = if tx == TxId::INVALID { "Sentinel TxId(0)" } else { "Wall-Clock-ns-Bereich" },
                "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId in remove_edge (weder im plausiblen next_tx-Bereich noch im INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]) — \
                 möglicherweise unalloziert oder aus Wall-Clock-Nanosekunden abgeleitet."
            );
            }
            {
                let mut inner = self.inner_write();
                inner
                    .staged_removals
                    .entry(tx)
                    .or_default()
                    .push((from, to));
            }
            if let Some(ref storage) = self.storage {
                self.delete_edge_persistence(storage.as_ref(), tx, &from, &to)
                    .await?;
            }
            Ok(())
        })
    }

    fn add_bidirectional<'a>(
        &'a self,
        tx: TxId,
        from: EntityId,
        to: EntityId,
        label: &'a str,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.add_edge(tx, memfuse_core::Edge::new(from, to, label))
                .await?;
            self.add_edge(tx, memfuse_core::Edge::new(to, from, label))
                .await?;
            Ok(())
        })
    }

    fn neighbors<'a>(&'a self, start: EntityId) -> BoxFuture<'a, Result<Vec<EntityId>>> {
        Box::pin(async move { self.neighbors(start).await })
    }

    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut inner = self.inner_write();
            inner.staged_entities.retain(|&(t, _), _| t != tx);
            inner.staged_edges.retain(|&(t, _), _| t != tx);
            inner.staged_removals.remove(&tx);
            Ok(())
        })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            // Physical rollback for CSR graph is driven by WAL replay or reloading state from storage.
            // In-memory staged transactions are handled by rollback().
            Ok(())
        })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(self.last_tx_id.load(Ordering::SeqCst))) })
    }

    fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
        Box::pin(async move { self.entity_count() })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>> {
        Box::pin(async move {
            let inner = self.inner_read();
            let num_entities = inner
                .entities
                .iter()
                .filter(|e| e.id != EntityId::new(0))
                .count();
            let num_edges = inner.targets.len()
                + inner.pending_edge_count
                + inner.staged_edges.values().map(|v| v.len()).sum::<usize>();

            let mem = (inner.reverse_map.len() * std::mem::size_of::<EntityId>())
                + (inner.entities.len() * std::mem::size_of::<Entity>())
                + (inner.offsets.len() * std::mem::size_of::<usize>())
                + (inner.targets.len() * std::mem::size_of::<usize>())
                + (inner.weights.len() * std::mem::size_of::<f32>());

            Ok(GraphIndexStats {
                num_entities,
                num_edges,
                memory_usage_bytes: mem,
            })
        })
    }
}
