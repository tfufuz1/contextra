// FILE-CONTEXT
// ZWECK: Multi-Step Iterative Retrieval Engine für komplexe Agenten-Abfragen (o-series Pattern).
// INVARIANTEN: RRF-Fusion über alle Runden; Abbruch bei Erreichen des Qualitätsschwellenwerts; Candidate Pool PID-reguliert in [50, 200].
// STAND: TS:2026-09-12T00:00:00Z

//! Multi-Step Iterative Retrieval Engine für komplexe Abfragen in KI-Agenten.
//!
//! # Architektur & Ring-Zugehörigkeit
//! Dieses Modul gehört zu `contextra-db` (Ring 3, Fassade) und implementiert ein
//! iteratives Abfragemuster (Retrieval-Augmented Generation / Agentic Search).
//!
//! # Funktionsweise für Entwickler
//! Komplexe Fragen können oft nicht in einem einzigen Suchdurchgang beantwortet werden.
//! Die Multi-Step-Engine arbeitet wie folgt:
//! 1. **Runde 1**: Standard-Hybrid-Suche (Vektor + BM25-Volltext) mit der Originalanfrage.
//! 2. **Qualitätsprüfung**: Es wird geprüft, ob genügend Treffer den Schwellenwert
//!    [`MultiStepConfig::quality_threshold`] überschreiten.
//! 3. **Runde 2..N**: Falls die Qualität unzureichend ist, generiert ein [`QueryRewriter`]
//!    automatisierte Teilfragen (Sub-Queries).
//! 4. **Multi-Signal-Abfragen**: Für Sub-Queries können neue Textanfragen, Vektoreinbettungen
//!    (via [`TextEmbeddingEngine`]) oder Wissensgraph-Knoten ([`EntityId`]) abgefragt werden.
//! 5. **RRF-Fusion**: Alle Zwischenergebnisse werden mittels RRF (Reciprocal Rank Fusion)
//!    zu einer finalen Rangliste kombiniert.
//! 6. **PID-Latenzschutz**: Ein PID-Regler steuert die Kandidatenpoolgröße und bricht
//!    bei Überschreitung des Zeitbudgets ([`LatencyBudgetGuard`]) geordnet ab (Fail-Safe).

use crate::pid_latency_controller::{LatencyBudgetGuard, DEFAULT_TARGET_LATENCY_MS};
use crate::{Collection, SearchResult};
use contextra_adapt::PidController;
pub use contextra_ports::QueryRewriter;
use contextra_ports::{Clock, QueryRewriteOutput, StorageEngine, SystemClock, TextEmbeddingEngine};
use contextra_types::{EntityId, Result, ScoredEntry};
use std::sync::Arc;

// ANCHOR[contextra-db:MULTISTEP-001]
/// Standardmäßige Obergrenze für generierte Teilfragen pro Iterationsrunde (N=3).
/// Verhindert kombinatorische Explosion und hält Latenzschranken ein.
pub const DEFAULT_MAX_SUB_QUERIES: usize = 3;

// ANCHOR[contextra-db:MULTISTEP-002]
/// Standardmäßiger Multiplikator zur Vergrößerung des Kandidatenpools vor dem Re-Ranking.
/// Verdoppelt die Trefferanzahl (2 * k), um genügend Recall vor Filtern zu sichern.
pub const DEFAULT_RERANK_POOL_MULTIPLIER: usize = 2;

// ANCHOR[contextra-db:MULTISTEP-003]
/// Konfiguration für das mehrstufige iterative Multi-Step Retrieval.
///
/// Steuert die Abbruchbedingungen, Qualitätsanforderungen und Zeitbudgets.
#[derive(Debug, Clone)]
pub struct MultiStepConfig {
    /// Maximale Anzahl an Iterationsrunden (Standard: 3).
    pub max_rounds: usize,
    /// Mindest-Score-Schwellenwert (0.0 bis 1.0): Liegen die Scores darunter,
    /// wird eine weitere Reformulierungsrunde initiiert.
    pub quality_threshold: f32,
    /// Minimale Anzahl an Treffern, die mindestens `quality_threshold` erreichen müssen.
    pub min_quality_hits: usize,
    /// Maximales Latenzbudget in Millisekunden für den gesamten Multi-Step-Zyklus (Standard: 100.0 ms).
    pub latency_budget_ms: f64,
    /// Multiplikator zur Vergrößerung des Kandidatenpools vor der Fusionierung (Standard: 2).
    pub rerank_pool_multiplier: usize,
}

impl Default for MultiStepConfig {
    /// Erzeugt Standardwerte: 3 Runden, Schwellenwert 0.5, 2 Qualitätstreffer, 100 ms Budget.
    fn default() -> Self {
        Self {
            max_rounds: 3,
            quality_threshold: 0.5,
            min_quality_hits: 2,
            latency_budget_ms: DEFAULT_TARGET_LATENCY_MS,
            rerank_pool_multiplier: DEFAULT_RERANK_POOL_MULTIPLIER,
        }
    }
}

// ANCHOR[contextra-db:MULTISTEP-004]
/// Ergebnis einer mehrstufigen Suche inklusive vollständiger Audit- und Nachvollziehbarkeitsdaten.
#[derive(Debug, Default)]
pub struct MultiStepResult {
    /// Die fusionierten Gesamtsuchergebnisse nach RRF (Reciprocal Rank Fusion).
    pub results: Vec<SearchResult>,
    /// Anzahl der tatsächlich durchlaufenen Iterationsrunden (1 bis max_rounds).
    pub rounds_executed: usize,
    /// Textuelle Teilfragen (Sub-Queries), die in den Folgerunden ausgeführt wurden.
    pub sub_queries: Vec<String>,
    /// Strukturierte Multi-Signal-Reformulierungen (Text, Semantik, Graph) der Runden 2..N.
    pub sub_rewrites: Vec<QueryRewriteOutput>,
    /// Berechnete dichte Vektoreinbettungen (Dense Embeddings) für semantische Sub-Queries.
    pub sub_vectors: Vec<Vec<f32>>,
}

// ANCHOR[contextra-db:MULTISTEP-005]
/// Multi-Step Retrieval Engine für iterative Abfragen.
///
/// Bietet adaptive, mehrstufige Retrieval-Zyklen auf einer zugrundeliegenden [`Collection`].
/// Nutzt einen optionalen [`QueryRewriter`] für LLM-basierte Anfrage-Reformulierungen
/// sowie einen internen PID-Regler zur dynamischen Latenzkontrolle.
pub struct MultiStepEngine<S: StorageEngine> {
    collection: Arc<Collection<S>>,
    config: MultiStepConfig,
    pid_controller: parking_lot::Mutex<PidController>,
    clock: Arc<dyn Clock>,
}

impl<S: StorageEngine> MultiStepEngine<S> {
    // ANCHOR[contextra-db:MULTISTEP-006]
    /// Erzeugt eine neue `MultiStepEngine` für eine gegebene [`Collection`].
    ///
    /// # Parameter
    /// - `collection`: Die Contextra-Kollektion, auf der gesucht wird.
    /// - `config`: Multi-Step-Konfiguration mit Budget- und Qualitätsschranken.
    ///
    /// # Nebenwirkungen & Ports
    /// Initialisiert den PID-Controller und bindet standardmäßig die Systemuhr ([`SystemClock`]) ein.
    pub fn new(collection: Arc<Collection<S>>, config: MultiStepConfig) -> Self {
        let pid = PidController::new(config.latency_budget_ms as f32, 50, 200, Some(100));
        Self {
            collection,
            config,
            pid_controller: parking_lot::Mutex::new(pid),
            clock: Arc::new(SystemClock::new()),
        }
    }

    // ANCHOR[contextra-db:MULTISTEP-007]
    /// Überschreibt den Zeit-Port mit einem benutzerdefinierten [`Clock`]-Port.
    ///
    /// # Zweck
    /// Ermöglicht deterministische Tests ohne Abhängigkeit von realer Wanduhr-Zeit
    /// (gemäß Regel 3 aus CLAUDE.md: Zeit nur über Ports).
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    // ANCHOR[contextra-db:MULTISTEP-008]
    /// Führt eine iterative Hybrid-Suche ohne separaten Vektor-Embedder durch.
    ///
    /// # Ablauf
    /// 1. Runde 1: Standard-Hybrid-Suche mit Originalanfrage und Originalvektor.
    /// 2. Qualitätsprüfung: Genügen die Treffer den Kriterien in [`MultiStepConfig`]?
    /// 3. Folgerunden: Generierung von Sub-Queries via [`QueryRewriter`].
    /// 4. RRF-Fusion: Zusammenführung aller Teilergebnisse.
    pub async fn search(
        &self,
        original_query: &str,
        vector: &[f32],
        k: usize,
        rewriter: Option<&dyn QueryRewriter>,
    ) -> Result<MultiStepResult> {
        self.search_with_embedder(original_query, vector, k, rewriter, None)
            .await
    }

    // ANCHOR[contextra-db:MULTISTEP-009]
    /// Führt eine iterative multi-signale Hybrid-Suche mit Vektor-Embedder durch.
    ///
    /// # Ablauf im Detail:
    /// 1. Eingabeprüfung & Schranke: `k` wird gegen `MAX_SEARCH_K` begrenzt.
    /// 2. Zeitwächter initialisieren: `LatencyBudgetGuard` überwacht das Millisekunden-Budget.
    /// 3. Runde 1:
    ///    - PID-Kandidatenpool ermitteln (geklemmt zwischen 50 und 200).
    ///    - Hybride Ausführung auf der Collection (Text + Vektor).
    ///    - Dauer messen und PID-Regler mit tatsächlicher Latenz aktualisieren.
    /// 4. Abbruchprüfung: Falls Qualität ausreicht oder kein Rewriter vorhanden ist,
    ///    direkt via Reciprocal Rank Fusion (RRF) fusionieren und zurückkehren.
    /// 5. Runden 2..N:
    ///    - Prüfen, ob Latenzbudget erschöpft ist; falls ja, mit Teilergebnissen abbrechen.
    ///    - Bisherige Treffer an `rewriter.rewrite_structured()` übergeben (mit Restzeit-Timeout).
    ///    - Für jede generierte Teilfrage: Textsuche, optionale Vektor-Neueinbettung
    ///      über `embedder` und Graph-Anchor-Einschränkung ausführen.
    ///    - Ergebnisse deduplizieren und zur Rundenliste hinzufügen.
    ///    - Prüfen, ob Qualitätsschwelle erreicht ist.
    /// 6. RRF-Fusion: Alle Ergebnismengen über RRF kombinieren.
    pub async fn search_with_embedder(
        &self,
        original_query: &str,
        vector: &[f32],
        k: usize,
        rewriter: Option<&dyn QueryRewriter>,
        embedder: Option<&dyn TextEmbeddingEngine>,
    ) -> Result<MultiStepResult> {
        use crate::fusion::reciprocal_rank_fusion;

        let k = k.min(contextra_types::MAX_SEARCH_K);
        let mut all_result_sets: Vec<Vec<SearchResult>> = Vec::new();
        let mut sub_queries: Vec<String> = Vec::new();
        let mut sub_rewrites: Vec<QueryRewriteOutput> = Vec::new();
        let mut sub_vectors: Vec<Vec<f32>> = Vec::new();
        let mut rounds_executed = 0;
        let budget_guard = LatencyBudgetGuard::new(self.config.latency_budget_ms);

        // Runde 1: Standard-Suche mit PID-reguliertem Kandidatenpool
        let start_nanos = self.clock.monotonic_nanos();
        let current_pid_val = self
            .pid_controller
            .lock()
            .current_pool_size()
            .unwrap_or_else(|| self.pid_controller.lock().min_pool_size);
        let candidate_k = (self.config.rerank_pool_multiplier * k)
            .clamp(50, current_pid_val)
            .clamp(50, 200);

        let round1 = self
            .collection
            .query()
            .text(original_query)
            .vector(vector)
            .k(candidate_k)
            .execute()
            .await?;

        let round1_elapsed_nanos = self.clock.monotonic_nanos().saturating_sub(start_nanos);
        let round1_dt = std::time::Duration::from_nanos(round1_elapsed_nanos);
        let round1_elapsed_ms = round1_dt.as_secs_f32() * 1000.0;
        // Update PID controller state for subsequent rounds and queries; returned pool size is stored in current_pool_size.
        self.pid_controller
            .lock()
            .update(round1_dt, round1_elapsed_ms);

        all_result_sets.push(round1);
        rounds_executed += 1;

        let round1_ref = all_result_sets.first().map(|v| v.as_slice()).unwrap_or(&[]);
        if self.quality_sufficient(round1_ref) || rewriter.is_none() {
            let fused = reciprocal_rank_fusion(all_result_sets, k);
            return Ok(MultiStepResult {
                results: fused,
                rounds_executed,
                sub_queries,
                sub_rewrites,
                sub_vectors,
            });
        }

        let rewriter = match rewriter {
            Some(r) => r,
            None => {
                let fused = reciprocal_rank_fusion(all_result_sets, k);
                return Ok(MultiStepResult {
                    results: fused,
                    rounds_executed,
                    sub_queries,
                    sub_rewrites,
                    sub_vectors,
                });
            }
        };

        for _round in 2..=self.config.max_rounds {
            if budget_guard.is_exceeded() {
                tracing::info!(
                    elapsed_ms = budget_guard.elapsed_ms(),
                    budget_ms = budget_guard.budget_ms(),
                    "MultiStep search budget exceeded; terminating expansion gracefully with partial results"
                );
                break;
            }

            let round_start_nanos = self.clock.monotonic_nanos();
            let pid_val = self
                .pid_controller
                .lock()
                .current_pool_size()
                .unwrap_or_else(|| self.pid_controller.lock().min_pool_size);
            let scaled_k = (self.config.rerank_pool_multiplier * k)
                .clamp(50, pid_val)
                .clamp(50, 200);

            let current_results = all_result_sets.last().map(|v| v.as_slice()).unwrap_or(&[]);
            let scored_entries: Vec<ScoredEntry> = current_results
                .iter()
                .map(|r| ScoredEntry {
                    id: r.id.clone(),
                    final_score: r.score,
                    metadata: r.metadata.clone(),
                })
                .collect();

            let remaining_ms = (self.config.latency_budget_ms - budget_guard.elapsed_ms()).max(0.0);
            let remaining_dur = std::time::Duration::from_secs_f64(remaining_ms / 1000.0);

            let rewrite_outputs_res = tokio::time::timeout(
                remaining_dur,
                rewriter.rewrite_structured(original_query, &scored_entries),
            )
            .await;

            let outputs_to_process = match rewrite_outputs_res {
                Ok(Ok(outputs)) => {
                    if outputs.is_empty() || outputs.iter().all(|o| o.is_empty()) {
                        break;
                    }
                    outputs.into_iter().take(DEFAULT_MAX_SUB_QUERIES).collect()
                }
                Ok(Err(e)) => {
                    tracing::warn!(
                        error = %e,
                        "QueryRewriter.rewrite_structured() failed in round; falling back to original query"
                    );
                    vec![QueryRewriteOutput::text_only(original_query)]
                }
                Err(_) => {
                    tracing::info!(
                        elapsed_ms = budget_guard.elapsed_ms(),
                        "QueryRewriter timed out under latency budget guard; falling back to original query"
                    );
                    vec![QueryRewriteOutput::text_only(original_query)]
                }
            };

            let mut round_results = Vec::new();

            for output in outputs_to_process {
                sub_rewrites.push(output.clone());

                let mut builder = self.collection.query().k(scaled_k);

                // Text modality
                let text_q = output.text_query.as_deref().unwrap_or(original_query);
                let text_q_final = if text_q.trim().is_empty() {
                    original_query
                } else {
                    text_q
                };
                builder = builder.text(text_q_final);
                sub_queries.push(text_q_final.to_string());

                // Semantic modality (Vector)
                let sub_vec = if let Some(sem_q) = &output.semantic_query {
                    if !sem_q.trim().is_empty() {
                        if let Some(emb) = embedder {
                            match emb.embed(sem_q).await {
                                Ok(v) => {
                                    sub_vectors.push(v.clone());
                                    Some(v)
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        semantic_query = %sem_q,
                                        error = %e,
                                        "Failed to re-embed semantic sub-query; falling back to original vector"
                                    );
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(ref v) = sub_vec {
                    builder = builder.vector(v);
                } else {
                    builder = builder.vector(vector);
                }

                // Graph modality (Anchors)
                if !output.anchor_entities.is_empty() {
                    let anchor_ids: Vec<EntityId> = output
                        .anchor_entities
                        .iter()
                        .filter_map(|e| EntityId::from_key(e).ok())
                        .collect();
                    if !anchor_ids.is_empty() {
                        builder = builder.anchor_entities(anchor_ids);
                    }
                }

                match builder.execute().await {
                    Ok(sub_res) => {
                        round_results.extend(sub_res);
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Multi-signal sub-query search failed; skipping"
                        );
                    }
                }
            }

            let round_elapsed_nanos = self
                .clock
                .monotonic_nanos()
                .saturating_sub(round_start_nanos);
            let round_dt = std::time::Duration::from_nanos(round_elapsed_nanos);
            let round_elapsed_ms = round_dt.as_secs_f32() * 1000.0;
            // Update PID controller state for subsequent rounds and queries; returned pool size is stored in current_pool_size.
            self.pid_controller
                .lock()
                .update(round_dt, round_elapsed_ms);

            let deduped_round = deduplicate_search_results(round_results);
            all_result_sets.push(deduped_round);
            rounds_executed += 1;

            let latest_results = all_result_sets.last().map(|v| v.as_slice()).unwrap_or(&[]);
            if self.quality_sufficient(latest_results) {
                break;
            }
        }

        let fused = reciprocal_rank_fusion(all_result_sets, k);
        Ok(MultiStepResult {
            results: fused,
            rounds_executed,
            sub_queries,
            sub_rewrites,
            sub_vectors,
        })
    }

    // ANCHOR[contextra-db:MULTISTEP-010]
    /// Prüft, ob die aktuellen Suchergebnisse die Qualitätsanforderungen erfüllen.
    ///
    /// Zählt Treffer mit Score >= `quality_threshold` und vergleicht mit `min_quality_hits`.
    fn quality_sufficient(&self, results: &[SearchResult]) -> bool {
        let high_quality = results
            .iter()
            .filter(|r| r.score >= self.config.quality_threshold)
            .count();
        high_quality >= self.config.min_quality_hits
    }
}

// ANCHOR[contextra-db:MULTISTEP-011]
/// Dedupliziert Suchergebnisse nach Dokumenten-ID unter Beibehaltung der Reihenfolge.
fn deduplicate_search_results(results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::with_capacity(results.len());
    for res in results {
        if seen.insert(res.id.clone()) {
            deduped.push(res);
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_graph::CsrGraph;
    use contextra_store::{LsmConfig, LsmStorage};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use tempfile::tempdir;

    use contextra_ports::BoxFuture;

    // Test-Rewriter, der vordefinierte Sub-Queries zurückgibt.
    struct DummyRewriter {
        responses: std::sync::Mutex<Vec<Vec<String>>>,
    }

    impl QueryRewriter for DummyRewriter {
        fn rewrite<'a>(
            &'a self,
            _original_query: &'a str,
            _current_results: &'a [ScoredEntry],
        ) -> BoxFuture<'a, Result<Vec<String>>> {
            Box::pin(async move {
                let mut guard = self.responses.lock().unwrap(); // unwrap
                if !guard.is_empty() {
                    Ok(guard.remove(0))
                } else {
                    Ok(vec![])
                }
            })
        }
    }

    // Hilfsfunktion: Erzeugt eine Test-Kollektion mit In-Memory/TempDir-LSM und HNSW.
    async fn create_test_collection() -> Arc<Collection<LsmStorage>> {
        let dir = tempdir().expect("tempdir"); // expect
        let lsm_config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("lsm storage")); // expect
        let hnsw_config = HnswConfig {
            dimension: 4,
            ..Default::default()
        };
        let index = Arc::new(HnswIndex::try_new(hnsw_config).expect("hnsw index")); // expect
        let graph = Arc::new(CsrGraph::new());
        let next_tx = Arc::new(AtomicU64::new(1));

        let col = Collection::new(
            "default".to_string(),
            storage,
            index,
            graph,
            next_tx,
            4,
            contextra_text::Language::English,
        );

        Arc::new(col)
    }

    // Prüft, dass bei ausreichender Qualität in Runde 1 keine weiteren Runden gestartet werden.
    #[tokio::test]
    async fn test_multistep_single_round_sufficient() {
        let col = create_test_collection().await;
        col.insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust programming"})),
        )
        .await
        .expect("insert"); // expect
        col.insert(
            "doc2",
            &[0.9, 0.1, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust language"})),
        )
        .await
        .expect("insert"); // expect

        let config = MultiStepConfig {
            max_rounds: 3,
            quality_threshold: 0.001,
            min_quality_hits: 1,
            latency_budget_ms: 1000.0,
            ..Default::default()
        };
        let engine = MultiStepEngine::new(col, config);

        let rewriter = DummyRewriter {
            responses: std::sync::Mutex::new(vec![vec!["sub query 1".to_string()]]),
        };

        let result = engine
            .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
            .await
            .expect("search"); // expect

        assert_eq!(result.rounds_executed, 1);
        assert!(result.sub_queries.is_empty());
        assert!(!result.results.is_empty());
    }

    // Prüft, dass bei zu geringer Qualität in Runde 1 eine Reformulierung via Rewriter ausgelöst wird.
    #[tokio::test]
    async fn test_multistep_query_rewriting_triggers() {
        let col = create_test_collection().await;
        col.insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust programming"})),
        )
        .await
        .expect("insert"); // expect

        let config = MultiStepConfig {
            max_rounds: 3,
            quality_threshold: 0.99, // high threshold, round 1 won't meet min_quality_hits=2
            min_quality_hits: 2,
            latency_budget_ms: 1000.0,
            ..Default::default()
        };
        let engine = MultiStepEngine::new(col, config);

        let rewriter = DummyRewriter {
            responses: std::sync::Mutex::new(vec![
                vec!["rust programming".to_string()], // round 2
                vec![],                               // round 3 (stop)
            ]),
        };

        let result = engine
            .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
            .await
            .expect("search"); // expect

        assert_eq!(result.rounds_executed, 2);
        assert_eq!(result.sub_queries, vec!["rust programming"]);
        assert!(!result.results.is_empty());
    }

    // Prüft das Verhalten, wenn kein QueryRewriter bereitgestellt wird (nur 1 Runde).
    #[tokio::test]
    async fn test_multistep_no_rewriter_provided() {
        let col = create_test_collection().await;
        let config = MultiStepConfig::default();
        let engine = MultiStepEngine::new(col, config);

        let result = engine
            .search("query", &[1.0, 0.0, 0.0, 0.0], 5, None)
            .await
            .expect("search"); // expect

        assert_eq!(result.rounds_executed, 1);
        assert!(result.sub_queries.is_empty());
    }

    // Prüft, dass Sub-Queries bei Bedarf auch nur BM25-Volltext verwenden.
    #[tokio::test]
    async fn test_multistep_subquery_uses_bm25_only() {
        let col = create_test_collection().await;
        col.insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust programming language"})),
        )
        .await
        .expect("insert"); // expect

        let config = MultiStepConfig {
            max_rounds: 2,
            quality_threshold: 0.99,
            min_quality_hits: 2,
            latency_budget_ms: 1000.0,
            ..Default::default()
        };
        let engine = MultiStepEngine::new(col, config);

        let rewriter = DummyRewriter {
            responses: std::sync::Mutex::new(vec![vec!["rust programming".to_string()]]),
        };

        let result = engine
            .search("original", &[0.1, 0.2, 0.3, 0.4], 5, Some(&rewriter))
            .await
            .expect("search"); // expect

        assert_eq!(result.rounds_executed, 2);
        assert_eq!(result.sub_queries, vec!["rust programming"]);
        assert!(!result.results.is_empty());
    }

    // Test-Rewriter, der absichtlich Fehler meldet.
    struct FailingRewriter;

    impl QueryRewriter for FailingRewriter {
        fn rewrite<'a>(
            &'a self,
            _original_query: &'a str,
            _current_results: &'a [ScoredEntry],
        ) -> BoxFuture<'a, Result<Vec<String>>> {
            Box::pin(async move {
                Err(contextra_types::ContextraError::Internal(
                    "Rewriter error".into(),
                ))
            })
        }
    }

    // Prüft, dass bei überschrittenem Latenzbudget Teilergebnisse ohne Abbruch zurückgegeben werden.
    #[tokio::test]
    async fn test_multistep_latency_budget_exceeded_returns_partial_result() {
        let col = create_test_collection().await;
        col.insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust programming"})),
        )
        .await
        .expect("insert");

        let config = MultiStepConfig {
            max_rounds: 3,
            quality_threshold: 0.99,
            min_quality_hits: 2,
            latency_budget_ms: 0.0001,
            ..Default::default()
        };
        let engine = MultiStepEngine::new(col, config);

        let rewriter = DummyRewriter {
            responses: std::sync::Mutex::new(vec![vec!["sub query 1".to_string()]]),
        };

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let result = engine
            .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
            .await
            .expect("search should succeed gracefully with partial results");

        assert_eq!(result.rounds_executed, 1);
        assert!(result.sub_queries.is_empty());
        assert!(!result.results.is_empty());
    }

    // Prüft, dass ein scheiternder Rewriter auf die Originalquery zurückfällt und keinen Absturz verursacht.
    #[tokio::test]
    async fn test_multistep_failing_rewriter_gracefully_stops() {
        let col = create_test_collection().await;
        col.insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust programming"})),
        )
        .await
        .expect("insert"); // expect

        let config = MultiStepConfig {
            max_rounds: 3,
            quality_threshold: 0.99,
            min_quality_hits: 2,
            latency_budget_ms: 1000.0,
            ..Default::default()
        };
        let engine = MultiStepEngine::new(col, config);
        let rewriter = FailingRewriter;

        let result = engine
            .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
            .await
            .expect("search should succeed gracefully even if rewriter fails"); // expect

        assert_eq!(result.rounds_executed, 3);
        assert_eq!(result.sub_queries, vec!["rust", "rust"]);
    }

    // Test-Rewriter mit künstlicher Verzögerung.
    struct SlowRewriter;

    impl QueryRewriter for SlowRewriter {
        fn rewrite<'a>(
            &'a self,
            _original_query: &'a str,
            _current_results: &'a [ScoredEntry],
        ) -> BoxFuture<'a, Result<Vec<String>>> {
            Box::pin(async move {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                Ok(vec!["sub query 1".to_string()])
            })
        }
    }

    // Prüft die Latenzbudget-Schranke bei langsamem Rewriting.
    #[tokio::test]
    async fn test_multistep_latency_budget_guard_exceeded() {
        let col = create_test_collection().await;
        col.insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({"text": "rust programming"})),
        )
        .await
        .expect("insert");

        let config = MultiStepConfig {
            max_rounds: 5,
            quality_threshold: 0.99,
            min_quality_hits: 2,
            latency_budget_ms: 10.0,
            ..Default::default()
        };
        let engine = MultiStepEngine::new(col, config);
        let rewriter = SlowRewriter;

        let result = engine
            .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
            .await
            .expect("search should succeed with partial results under budget exhaustion");

        assert_eq!(result.rounds_executed, 2);
        assert!(!result.results.is_empty());
    }

    // Prüft Latenzbudget-Erschöpfung auf einer leeren Datenbank.
    #[tokio::test]
    async fn test_multistep_latency_budget_exhaustion_on_empty_db() {
        let col = create_test_collection().await;
        let config = MultiStepConfig {
            max_rounds: 5,
            quality_threshold: 0.99,
            min_quality_hits: 2,
            latency_budget_ms: 1.0,
            ..Default::default()
        };
        let engine = MultiStepEngine::new(col, config);
        let rewriter = SlowRewriter;

        let result = engine
            .search("nonexistent", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
            .await
            .expect("search on empty DB over budget must return empty result without panic");

        assert!(result.results.is_empty());
    }
}
