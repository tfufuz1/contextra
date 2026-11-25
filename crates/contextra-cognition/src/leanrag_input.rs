// FILE-CONTEXT
// ZWECK: Extraktion von LeanRAG-Eingabedaten (AggregationNode/AggregationEdge) aus einer Collection (Spec §21.4).
// INVARIANTEN: No unsafe code; keine Async-Locks in der synchronen Extraktionsfunktion; deterministische Sortierung.

//! Extraktion von LeanRAG-Eingabedaten ([`AggregationNode`], [`AggregationEdge`]) aus einer [`Collection`].

use crate::aggregation_phase::{AggregationEdge, AggregationNode};
use contextra_engine::collection::Collection;
use contextra_graph::hyperedge::HyperEdgeId;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{DocId, EntityId};
use std::collections::{HashMap, HashSet};

/// Standardmäßige Obergrenze für die Anzahl extrahierter Knoten im LeanRAG-Input.
pub const DEFAULT_MAX_LEANRAG_NODES: usize = 10_000;

/// Extrahiertes Eingabepaket für die LeanRAG Stage-3-Aggregation.
#[derive(Debug, Clone, PartialEq)]
pub struct LeanRagInputs {
    /// Liste der zugrundeliegenden Entitätsknoten.
    pub nodes: Vec<AggregationNode>,
    /// Liste der zugrundeliegenden Hyperkanten zwischen den Knoten.
    pub edges: Vec<AggregationEdge>,
}

/// Baut [`LeanRagInputs`] aus den Turns einer Collection und den assoziierten Graphendaten.
///
/// # Konvertierungsregeln
/// - **Knoten**: Für jeden Turn `(DocId, Vec<f32>)` wird eine [`EntityId`] per [`EntityId::from_doc_id`] abgeleitet.
///   Nur Entitäten, die im Graphen existieren (`graph.entity_exists`), werden berücksichtigt.
///   Der `type_id` ist standardmäßig `0`, da [`EntityId`] keine explizite Typ-Tombstone-Information trägt.
/// - **Kanten**: Für alle gültigen Knoten werden deren Hyperkanten via `graph.hyperedges_for_entity` abgerufen
///   und dedupliziert. Eine Hyperkante wird nur aufgenommen, wenn:
///   1. Sie existiert (`graph.get_hyperedge` liefert `Some`, d. h. nicht tombstoniert).
///   2. Sie mindestens 2 Teilnehmer besitzt (`participants.len() >= 2`).
///   3. **Alle** Teilnehmer der Hyperkante in der gefilterten Knotenmenge enthalten sind. Kanten mit
///      Teilnehmern außerhalb der Knotenmenge werden verworfen, um Haltungsinvarianten des GMM-Clustering zu wahren.
/// - **Prädikatstyp**: `hyperedge.predicate` ([`EdgeType`]) wird via Blake3-Hash der Bincode-Serialisierung
///   auf `u32` (die ersten 4 Bytes) konvertiert.
/// - **Determinismus**: Knoten werden kanonisch nach `EntityId` (aufsteigend) sortiert und auf `max_nodes` gekappt;
///   Kanten werden kanonisch nach `HyperEdgeId` sortiert.
pub fn build_leanrag_inputs<S: StorageEngine, V: VectorIndex>(
    collection: &Collection<S, V>,
    turns: &[(DocId, Vec<f32>)],
    max_nodes: usize,
) -> LeanRagInputs {
    let graph = collection.graph_index();

    // 1. Eindeutige Knoten aus turns sammeln, die im Graphen existieren
    let mut turn_nodes: HashMap<EntityId, Vec<f32>> = HashMap::new();
    for (doc_id, embedding) in turns {
        let entity = EntityId::from_doc_id(*doc_id);
        if graph.entity_exists(entity) {
            turn_nodes
                .entry(entity)
                .or_insert_with(|| embedding.clone());
        }
    }

    // Convert to AggregationNodes and sort deterministically
    let mut nodes: Vec<AggregationNode> = turn_nodes
        .into_iter()
        .map(|(entity, embedding)| AggregationNode {
            entity,
            embedding,
            type_id: 0,
        })
        .collect();

    nodes.sort_unstable_by_key(|n| n.entity.inner());
    if nodes.len() > max_nodes {
        nodes.truncate(max_nodes);
    }

    let node_entities: HashSet<EntityId> = nodes.iter().map(|n| n.entity).collect();

    // 2. Kanten sammeln
    let mut raw_edge_ids = HashSet::new();
    for node in &nodes {
        for edge_id in graph.hyperedges_for_entity(node.entity) {
            raw_edge_ids.insert(edge_id);
        }
    }

    let mut sorted_edge_ids: Vec<HyperEdgeId> = raw_edge_ids.into_iter().collect();
    sorted_edge_ids.sort_unstable_by_key(|e| e.0);

    let mut edges = Vec::new();
    for edge_id in sorted_edge_ids {
        if let Some(hyperedge) = graph.get_hyperedge(edge_id) {
            let participants: Vec<EntityId> =
                hyperedge.participants.iter().map(|rb| rb.entity).collect();

            // Kante muss >=2 Teilnehmer haben und ALLE Teilnehmer müssen in der gefilterten Knotenmenge sein
            if participants.len() >= 2 && participants.iter().all(|p| node_entities.contains(p)) {
                let pred_bytes = format!("{:?}", hyperedge.predicate).into_bytes();
                let hash = blake3::hash(&pred_bytes);
                let hash_arr = hash.as_bytes();
                let predicate_type =
                    u32::from_le_bytes([hash_arr[0], hash_arr[1], hash_arr[2], hash_arr[3]]);

                edges.push(AggregationEdge {
                    id: edge_id,
                    predicate_type,
                    participants,
                });
            }
        }
    }

    LeanRagInputs { nodes, edges }
}
