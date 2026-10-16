// FILE-CONTEXT
// ZWECK: Transitivity Veto Inspection & Orchestration for Near-Duplicate Memory Consolidation.
// INVARIANTEN: Zero-Panic; P24-Lokalität (O(N^3) nur auf vorgefilterten, budgetbegrenzten Kandidatenmengen).
// STAND: TS:2026-09-27T00:00:00Z

//! Transitivity Veto Inspektion & Orchestrierung.
//!
//! Stellt sicher, dass bei der Near-Duplicate-Erkennung keine Fehlmerges durch
//! transitive Verkettung auftreten (z. B. A ≈ B und B ≈ C, aber A ≉ C).

use crate::aggregation_phase::{AggregationConfig, AggregationNode};
use crate::memory_consolidation::{cosine_similarity, detect_near_duplicates};
use contextra_types::{DocId, EntityId};
use std::collections::HashSet;

/// Reine, synchrone Funktion, die für drei `AggregationNode`-Kandidaten (A, B, C)
/// und einen Ähnlichkeits-Schwellenwert prüft, ob eine transitive Verkettung
/// (z. B. A ähnlich B, B ähnlich C, aber A tatsächlich unähnlich C unterhalb des Schwellenwerts)
/// vorliegt.
///
/// **Rückgabe**:
/// - `false` (Veto: NICHT mergen), wenn eine transitive Verkettung mit Endpunkte-Unähnlichkeit vorliegt.
/// - `true` (Kein Veto: Merge zulässig), wenn keine transitive Diskrepanz vorliegt.
pub fn validate_transitivity_veto(
    node_a: &AggregationNode,
    node_b: &AggregationNode,
    node_c: &AggregationNode,
    threshold: f32,
) -> bool {
    let sim_ab = cosine_similarity(&node_a.embedding, &node_b.embedding);
    let sim_bc = cosine_similarity(&node_b.embedding, &node_c.embedding);
    let sim_ac = cosine_similarity(&node_a.embedding, &node_c.embedding);

    // B ist Brücke zwischen A und C, aber A und C sind unähnlich
    if sim_ab >= threshold && sim_bc >= threshold && sim_ac < threshold {
        return false;
    }
    // A ist Brücke zwischen B und C, aber B und C sind unähnlich
    if sim_ab >= threshold && sim_ac >= threshold && sim_bc < threshold {
        return false;
    }
    // C ist Brücke zwischen A und B, aber A und B sind unähnlich
    if sim_ac >= threshold && sim_bc >= threshold && sim_ab < threshold {
        return false;
    }

    true
}

/// Orchestrierungsfunktion für die Transitivitätsprüfung unter Einhaltung der P24-Lokalität.
///
/// **Ablauf**:
/// (a) Nutzt zuerst `detect_near_duplicates`, um eine vorgefilterte, kleine Kandidatenmenge zu erhalten.
/// (b) Prüft die Größe/Speicherbedarf der Kandidatenmenge gegen `AggregationConfig.max_compaction_peak_memory_mb`.
///     Bei Überschreitung wird die Transitivitätsprüfung für diesen Batch mit einer `tracing::warn!`-Meldung
///     übersprungen (kein Abbrechen der Konsolidierung).
/// (c) Führt erst dann `validate_transitivity_veto`-Aufrufe ausschließlich innerhalb der vorgefilterten,
///     budgetbegrenzten Menge durch.
pub fn filter_candidates_with_transitivity_veto(
    turns: &[(DocId, Vec<f32>)],
    threshold: f32,
    agg_cfg: &AggregationConfig,
) -> Vec<(DocId, DocId)> {
    // (a) Vorgefilterte Kandidatenpaare aus detect_near_duplicates beziehen
    let raw_pairs = detect_near_duplicates(turns, threshold);
    if raw_pairs.is_empty() {
        return Vec::new();
    }

    // Eindeutige Kandidaten-DocIds aus den Rohpaaren extrahieren
    let mut candidate_ids_set = HashSet::new();
    for (older, newer) in &raw_pairs {
        candidate_ids_set.insert(*older);
        candidate_ids_set.insert(*newer);
    }

    let candidate_count = candidate_ids_set.len();
    let turn_map: std::collections::HashMap<DocId, &Vec<f32>> =
        turns.iter().map(|(id, emb)| (*id, emb)).collect();

    let mut candidate_nodes = Vec::with_capacity(candidate_count);
    for doc_id in &candidate_ids_set {
        if let Some(emb) = turn_map.get(doc_id) {
            candidate_nodes.push(AggregationNode {
                entity: EntityId::from_doc_id(*doc_id),
                embedding: (*emb).clone(),
                type_id: 0,
            });
        }
    }

    // (b) Budget-Prüfung gegen AggregationConfig.max_compaction_peak_memory_mb
    let dim = candidate_nodes
        .first()
        .map(|n| n.embedding.len())
        .unwrap_or(0);
    let candidate_bytes = candidate_count
        * (dim * std::mem::size_of::<f32>() + std::mem::size_of::<AggregationNode>());
    let candidate_mem_mb = candidate_bytes.div_ceil(1024 * 1024);

    if candidate_mem_mb > agg_cfg.max_compaction_peak_memory_mb {
        tracing::warn!(
            candidate_count = candidate_count,
            candidate_mem_mb = candidate_mem_mb,
            limit_mb = agg_cfg.max_compaction_peak_memory_mb,
            "Transitivity check skipped: prefiltered candidate set memory budget exceeded"
        );
        return raw_pairs;
    }

    // (c) Paarweise validate_transitivity_veto-Aufrufe ausschließlich innerhalb der vorgefilterten Menge
    let mut filtered_pairs = Vec::with_capacity(raw_pairs.len());

    for (older_id, newer_id) in raw_pairs {
        let older_entity = EntityId::from_doc_id(older_id);
        let newer_entity = EntityId::from_doc_id(newer_id);

        let node_older = candidate_nodes.iter().find(|n| n.entity == older_entity);
        let node_newer = candidate_nodes.iter().find(|n| n.entity == newer_entity);

        if let (Some(node_a), Some(node_b)) = (node_older, node_newer) {
            let mut vetoed = false;
            for node_c in &candidate_nodes {
                if node_c.entity == node_a.entity || node_c.entity == node_b.entity {
                    continue;
                }
                if !validate_transitivity_veto(node_a, node_b, node_c, threshold) {
                    vetoed = true;
                    break;
                }
            }
            if !vetoed {
                filtered_pairs.push((older_id, newer_id));
            }
        }
    }

    filtered_pairs
}
