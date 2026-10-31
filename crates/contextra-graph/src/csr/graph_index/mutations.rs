//! Modul `mutations` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

use std::sync::atomic::Ordering;

use crate::consistency_enforcement::EdgeAssertion;
use crate::GraphIndexExt;
use contextra_ports::{BoxFuture, GraphIndex};
use contextra_types::{ContextraError, Entity, EntityId, Result, TxId};

use super::super::graph_write::CsrGraph;
use super::super::inner::sentinel_entity;
use super::super::types::{
    EdgePayload, PersistedEdgePayload, StagedEdgePayload, GRAPH_ENTITY_DELETED_PREFIX,
    GRAPH_ENTITY_PREFIX,
};
use super::super::visibility::is_suspicious_tx_id;
use super::collect_entity_incident_edges;

fn validate_tx_id_heuristic(tx: TxId, op_name: &str) {
    debug_assert!(
        tx != TxId::INVALID && tx.is_valid_origin(),
        "TxId {} verletzt AGT-GRAPH-001 Origin-Invariante — Sentinel TxId(0) oder Wall-Clock-abgeleitete IDs korrumpieren rollback_to_tx()-Kausalordnung",
        tx
    );
    if is_suspicious_tx_id(tx) {
        tracing::warn!(
            tx_id = tx.inner(),
            hint = if tx == TxId::INVALID { "Sentinel TxId(0)" } else { "Wall-Clock-ns-Bereich" },
            "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId in {} (weder im plausiblen next_tx-Bereich noch im INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]) — \
             möglicherweise unalloziert oder aus Wall-Clock-Nanosekunden abgeleitet.",
            op_name
        );
    }
}

impl GraphIndexExt for CsrGraph {
    fn remove_entity<'a>(&'a self, tx: TxId, entity: EntityId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let (target_idx, outgoing_targets, incoming_sources) = {
                let inner = self.inner_read();
                let idx = match inner.id_map.get(&entity) {
                    Some(&i) => i,
                    None => return Ok(()),
                };
                let (outgoing, incoming) = collect_entity_incident_edges(&inner, idx);
                (idx, outgoing, incoming)
            };

            for to_id in outgoing_targets {
                GraphIndex::remove_edge(self, tx, entity, to_id).await?;
            }
            for from_id in incoming_sources {
                GraphIndex::remove_edge(self, tx, from_id, entity).await?;
            }

            {
                let mut inner = self.inner_write();
                inner.id_map.remove(&entity);
                if target_idx < inner.entities.len() {
                    inner.entities[target_idx] = sentinel_entity();
                }
                inner.communities.remove(&entity);
                inner.is_dirty = true;
            }

            if let Some(ref storage) = self.storage {
                let entity_key = [GRAPH_ENTITY_PREFIX, entity.as_bytes().as_slice()].concat();
                storage.delete(tx, &entity_key).await?;

                let deleted_key =
                    [GRAPH_ENTITY_DELETED_PREFIX, entity.as_bytes().as_slice()].concat();
                storage
                    .put(tx, &deleted_key, &tx.inner().to_le_bytes())
                    .await?;
            }

            GraphIndex::commit(self, tx).await?;
            Ok(())
        })
    }
}

// ANCHOR[contextra-graph:ADD_ENTITY_I-0090]
/// Funktion `add_entity_impl` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub(crate) fn add_entity_impl<'a>(
    graph: &'a CsrGraph,
    tx: TxId,
    entity: Entity,
) -> BoxFuture<'a, Result<()>> {
    Box::pin(async move {
        validate_tx_id_heuristic(tx, "add_entity");
        {
            let mut inner = graph.inner_write();
            inner
                .staged_entities
                .insert((tx, entity.id), entity.clone());
        }

        if let Some(ref storage) = graph.storage {
            graph.persist_entity(storage.as_ref(), tx, &entity).await?;
        }
        Ok(())
    })
}

// ANCHOR[contextra-graph:ADD_EDGE_IMP-0091]
/// Funktion `add_edge_impl` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub(crate) fn add_edge_impl<'a>(
    graph: &'a CsrGraph,
    tx: TxId,
    edge: contextra_types::Edge,
) -> BoxFuture<'a, Result<()>> {
    Box::pin(async move {
        validate_tx_id_heuristic(tx, "add_edge");
        if !edge.weight.is_finite() || edge.weight < 0.0 {
            return Err(ContextraError::InvalidInput(format!(
                "Invalid edge weight {}: weight must be finite and non-negative",
                edge.weight
            )));
        }

        if let Some(ref enforcer_lock) = graph.consistency_enforcer {
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
                    return Err(ContextraError::PolicyViolation(
                        "Contradictory edge suppressed by consistency enforcer".to_string(),
                    ));
                }
                tracing::warn!(
                    "ConsistencyEnforcer: contradictory edge detected (not yet suppressed)"
                );
            }
        }

        let tx_valid_from = edge.tx_valid_from.or(Some(tx));

        if let Some(doc_id) = edge.source_doc_id {
            graph.doc_edge_index.record(doc_id, (edge.from, edge.to));
        }

        {
            let mut inner = graph.inner_write();
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

        if let Some(ref storage) = graph.storage {
            let payload = PersistedEdgePayload {
                weight: edge.weight,
                tx_valid_from,
                tx_valid_to: edge.tx_valid_to,
                business_valid_from: edge.business_valid_from,
                business_valid_to: edge.business_valid_to,
                source_doc_id: edge.source_doc_id,
            };
            graph
                .persist_edge(storage.as_ref(), tx, &edge.from, &edge.to, &payload)
                .await?;
        }
        Ok(())
    })
}

// ANCHOR[contextra-graph:COMMIT_IMPL-0092]
/// Funktion `commit_impl` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub(crate) fn commit_impl<'a>(graph: &'a CsrGraph, tx: TxId) -> BoxFuture<'a, Result<()>> {
    Box::pin(async move {
        validate_tx_id_heuristic(tx, "commit");
        let mut inner = graph.inner_write();

        commit_staged_entities(&mut inner, tx);
        commit_staged_edges(&mut inner, tx);
        commit_staged_removals(&mut inner, tx);

        if inner.pending_edge_count >= graph.config.rebuild_threshold {
            inner.compact();
        }

        graph.last_tx_id.fetch_max(tx.inner(), Ordering::SeqCst);
        Ok(())
    })
}

// ANCHOR[contextra-graph:REMOVE_EDGE_-0093]
/// Funktion `remove_edge_impl` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub(crate) fn remove_edge_impl<'a>(
    graph: &'a CsrGraph,
    tx: TxId,
    from: EntityId,
    to: EntityId,
) -> BoxFuture<'a, Result<()>> {
    Box::pin(async move {
        validate_tx_id_heuristic(tx, "remove_edge");
        {
            let mut inner = graph.inner_write();
            inner
                .staged_removals
                .entry(tx)
                .or_default()
                .push((from, to));
        }
        if let Some(ref storage) = graph.storage {
            graph
                .delete_edge_persistence(storage.as_ref(), tx, &from, &to)
                .await?;
        }
        Ok(())
    })
}

fn commit_staged_entities(inner: &mut super::super::inner::GraphInner, tx: TxId) {
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
}

fn commit_staged_edges(inner: &mut super::super::inner::GraphInner, tx: TxId) {
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
}

fn commit_staged_removals(inner: &mut super::super::inner::GraphInner, tx: TxId) {
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
}
