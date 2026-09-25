// FILE-CONTEXT
// ZWECK: Öffentliche Facade-Schnittstelle für LeanRAG Stage-3-Hyperkantenaggregatierung (Spec §5.3, §12.4).
// ARCHITEKTUR-INVARIANTE: contextra-cognition hängt von contextra-engine ab (nicht umgekehrt).
//  Die Methode consolidate_semantic_hyperedges ist daher als freie Funktion in contextra-cognition
//  implementiert, die &contextra_engine::collection::Collection<S, V> als Parameter entgegennimmt,
//  um eine zirkuläre Abhängigkeit zwischen contextra-engine und contextra-cognition zu verhindern
//  und die Ring-DAG-Ebenenhierarchie (§4) strikt einzuhalten.

//! Facade-Schnittstelle zur Ausführung der semantischen Hyperkanten-Konsolidierung (LeanRAG Stage 3).
//!
//! # Architektur-Hinweis
//! Gemäß der Ring-Abhängigkeitsmatrix (§4) hängt `contextra-cognition` von `contextra-engine` ab.
//! Eine `impl`-Methode direkt auf `contextra_engine::Collection` innerhalb von `contextra-engine` würde
//! eine unzulässige zirkuläre Abhängigkeit erzeugen. Stattdessen wird [`consolidate_semantic_hyperedges`]
//! als freie Funktion in `contextra-cognition` bereitgestellt.

use crate::aggregation_phase::{check_compaction_budget, AggregationConfig, AggregationPhaseResult};
use crate::consolidation_executor::execute_leanrag_aggregation_stage;
use crate::leanrag_input::{build_leanrag_inputs, DEFAULT_MAX_LEANRAG_NODES};
use crate::memory_consolidation::CommunityStabilityTracker;
use contextra_engine::collection::{Collection, StoredDocument};
use contextra_ports::{LlmTextGenerator, StorageEngine, TextEmbeddingEngine, VectorIndex};
use contextra_types::{DocId, Result};

/// Ausführung der semantischen Hyperkanten-Konsolidierung (LeanRAG Stage 3) auf einer Collection.
///
/// # Parameter
/// - `collection`: Referenz auf die Ziel-Collection.
/// - `embedder`: TextEmbeddingEngine zum Generieren von Embeddings für ehemals reine Textdokumente.
/// - `llm`: LlmTextGenerator für die generative Synthese abstrakter Alpha-Knoten.
/// - `config`: Konfigurationsparameter für die Aggregationsphase (GMM, Tau-Schwellen, Memory-Budgets).
///
/// # VOR-Bedingung & Budget-Prüfung
/// Vor der Vorbereitung der Eingaben oder Ausführung der Aggregation wird die Budget-Vorprüfung
/// via [`check_compaction_budget`] mit der `estimate_compaction_peak_bytes()`-Primitive des
/// Graph-Index aufgerufen (Spec §12.4). Falls das konfigurierte Speicherbudget überschritten ist,
/// wird die Konsolidierung sofort abgebrochen, ohne LLM-Aufrufe oder Cluster-Allokationen durchzuführen.
pub async fn consolidate_semantic_hyperedges<S: StorageEngine, V: VectorIndex>(
    collection: &Collection<S, V>,
    embedder: &dyn TextEmbeddingEngine,
    llm: &dyn LlmTextGenerator,
    config: &AggregationConfig,
) -> Result<AggregationPhaseResult> {
    // 1. Konfiguration validieren
    config.validate()?;

    // 2. Speicherbudget-Vorprüfung VOR jeglicher Allokation oder LLM-Aufrufen
    // (Ruft die estimate_compaction_peak_bytes()-Primitive des CsrGraph auf)
    let peak_bytes = collection.graph_index().estimate_compaction_peak_bytes();
    check_compaction_budget(peak_bytes, config)?;

    // 3. Turns/Dokumente aus der Collection laden
    let user_key_prefix = collection.user_key_prefix();
    let entries = collection.storage().scan_prefix(&user_key_prefix).await?;

    let mut turns: Vec<(DocId, Vec<f32>)> = Vec::new();
    for (k, v) in entries {
        if collection.name() == "default" && k.starts_with(b"__") {
            continue;
        }
        if let Ok(stored) = serde_json::from_slice::<StoredDocument>(&v) {
            if let Ok(doc_id) = DocId::from_key(&stored.id) {
                let text_opt = stored
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("text"))
                    .and_then(|v| v.as_str());

                let embedding = if stored.embedding.is_empty() {
                    if let Some(text) = text_opt {
                        match embedder.embed(text).await {
                            Ok(emb) => emb,
                            Err(_) => stored.embedding,
                        }
                    } else {
                        stored.embedding
                    }
                } else {
                    stored.embedding
                };
                turns.push((doc_id, embedding));
            }
        }
    }

    // 4. LeanRAG Inputs extrahieren (AggregationNode & AggregationEdge)
    let inputs = build_leanrag_inputs(collection, &turns, DEFAULT_MAX_LEANRAG_NODES);

    // 5. Transaktions-ID anfordern
    let wal_tx = collection.allocate_tx()?;

    // 6. Stage-3-Aggregation mit SuperEdgeSink-Kaskade ausführen
    let mut tracker = CommunityStabilityTracker::new();
    let (phase_result, _alpha_nodes) = execute_leanrag_aggregation_stage(
        collection,
        &inputs.nodes,
        &inputs.edges,
        config,
        llm,
        &mut tracker,
        wal_tx,
    )
    .await?;

    Ok(phase_result)
}
