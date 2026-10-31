//! # Kontext-Kompaktor (`context_compaction/compactor.rs`)
//!
//! Stellt die zentrale Kompaktierungs-Engine [`ContextCompactor`] bereit.
//! Sie sortiert und filtert Chunks nach Relevanz, gepinntem Status und Tool-Output-Herkunft
//! und verdichtet sie ins definierte Token-Budget. Unterstützt zudem die LLM-gestützte
//! Konsolidierung mit automatischem Backoff-Retry bei Nebenläufigkeitskonflikten (OCC).
//!
//! ## Ring-Zugehörigkeit
//! Teil der Kognitionsschicht (Ring 3). Greift auf `Collection` und die Ports `Rng` und `LlmTextGenerator` zu.
//!
//! ## Invarianten
//! - **Zufall (Regel 3):** Nicht-kryptographischer Zufall über `Arc<dyn Rng>` für Backoff-Jitter.
//! - **OCC-Konflikt-Behandlung:** Optimistic Concurrency Control mit exponentiellem Backoff und Jitter.
//! - **Priorisierung:** Gepinnte Chunks (`is_pinned`) werden bevorzugt behalten,
//!   Tool-Outputs zuerst kompaktiert.

use contextra_engine::collection::Collection;
use contextra_engine::ProvenanceRecord;
use contextra_ports::{
    ContextSegment, LlmTextGenerator, Rng, SeededRng, StorageEngine, VectorIndex,
};
use contextra_types::{ContextChunk, DocId, Result, TenantId, TokenBudget};
use std::sync::Arc;

use super::session::ConsolidationSession;
use super::types::{CompactedContext, CompactionStrategy, StatusToken};

/// Context Compaction Engine.
#[derive(Clone)]
// ANCHOR[contextra-cognition:COMP-001]
/// Zentrale Kompaktierungs-Engine.
pub struct ContextCompactor {
    budget: TokenBudget,
    strategy: CompactionStrategy,
    rng: Arc<dyn Rng>,
}

impl std::fmt::Debug for ContextCompactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextCompactor")
            .field("budget", &self.budget)
            .field("strategy", &self.strategy)
            .field("rng", &"<dyn Rng>")
            .finish()
    }
}

fn build_consolidated_chunk_from_summary(
    summary_text: String,
    chunks: &[ContextChunk],
    source_doc_ids: &[DocId],
) -> ContextChunk {
    let estimated_tokens = crate::context::ContextManager::estimate_tokens(&summary_text);

    let mut combined_metadata = serde_json::Map::new();
    combined_metadata.insert("llm_summarized".to_string(), serde_json::Value::Bool(true));
    combined_metadata.insert(
        "source_doc_count".to_string(),
        serde_json::Value::Number(chunks.len().into()),
    );

    let prov = ProvenanceRecord::synthesized_from(source_doc_ids);
    if let Ok(prov_val) = serde_json::to_value(&prov) {
        combined_metadata.insert("provenance".to_string(), prov_val);
    }

    let synthesized_doc_id = {
        let key = chunks
            .iter()
            .map(|c| c.doc_id.inner().to_string())
            .collect::<Vec<_>>()
            .join(":");
        DocId::from_key(&format!("contextra:consolidated:{key}")).unwrap_or_else(|_| {
            chunks
                .first()
                .map(|c| c.doc_id)
                .unwrap_or_else(|| DocId::new(0))
        })
    };

    let max_relevance = chunks.iter().fold(0.0f32, |max, c| max.max(c.relevance));

    ContextChunk {
        doc_id: synthesized_doc_id,
        content: summary_text,
        relevance: max_relevance,
        token_count: estimated_tokens,
        metadata: Some(serde_json::Value::Object(combined_metadata)),
        contextual_prefix: None,
        links: Vec::new(),
    }
}

impl ContextCompactor {
    /// Creates a new `ContextCompactor` with given `TokenBudget`, `CompactionStrategy`, and `Arc<dyn Rng>`.
    ///
    /// # DOKUMENTATION / HINWEIS
    /// Der übergebene RNG dient ausschließlich für nicht-kryptographische Aufgaben wie Backoff-Jitter,
    /// Permutationen und Clustering. Er DARF NICHT für die Generierung von Schlüsseln, Salzen,
    /// Tokens oder kryptographischem Material verwendet werden.
    // ANCHOR[contextra-cognition:COMP-002]
    /// Erstellt einen ContextCompactor mit explizit injiziertem RNG.
    pub fn new_with_rng(
        budget: TokenBudget,
        strategy: CompactionStrategy,
        rng: Arc<dyn Rng>,
    ) -> Self {
        Self {
            budget,
            strategy,
            rng,
        }
    }

    /// Creates a new `ContextCompactor` with given `TokenBudget` and `CompactionStrategy`.
    ///
    /// # DOKUMENTATION / HINWEIS
    /// Nutzt `SeededRng::from_entropy()` zur Vermeidung konstanter Seeds im Produktionscode.
    /// Der übergebene oder genutzte RNG ist NICHT kryptographisch und darf nicht für Schlüssel oder Salze genutzt werden.
    #[deprecated(note = "use ContextCompactor::new_with_rng to explicitly inject an Arc<dyn Rng>")]
    // ANCHOR[contextra-cognition:COMP-003]
    /// Erstellt einen ContextCompactor mit Standard-RNG (veraltet).
    pub fn new(budget: TokenBudget, strategy: CompactionStrategy) -> Self {
        Self::new_with_rng(budget, strategy, Arc::new(SeededRng::from_entropy()))
    }

    /// Setzt den Zufallszahlengenerator für Backoff-Jitter.
    ///
    /// # DOKUMENTATION / HINWEIS
    /// Der übergebene RNG ist NICHT kryptographisch und darf nicht für Schlüssel oder Salze genutzt werden.
    // ANCHOR[contextra-cognition:COMP-004]
    /// Setzt den Zufallszahlengenerator für Backoff-Jitter.
    pub fn with_rng(mut self, rng: Arc<dyn Rng>) -> Self {
        self.rng = rng;
        self
    }

    /// Kompaktiert eine Liste von Chunks auf das Token-Budget.
    ///
    /// Priorisiert nach Relevanz-Score. Tool-Output-Chunks (erkennbar an Metadata-Key "tool_output")
    /// werden zuerst kompaktiert.
    // ANCHOR[contextra-cognition:COMP-005]
    /// Kompaktiert Chunks auf das konfigurierte Token-Budget.
    pub fn compact(&self, chunks: Vec<ContextChunk>) -> CompactedContext {
        let _source_doc_ids: Vec<DocId> = chunks.iter().map(|c| c.doc_id).collect();
        let max_tokens = self.budget.available();
        let mut tokens_used = 0;
        let mut retained = Vec::new();
        let mut status_tokens = Vec::new();

        let mut sorted = chunks;
        sorted.sort_by(|a, b| {
            let a_pinned = Self::is_pinned(a);
            let b_pinned = Self::is_pinned(b);
            if a_pinned != b_pinned {
                return b_pinned.cmp(&a_pinned);
            }
            let a_tool = Self::is_tool_output(a);
            let b_tool = Self::is_tool_output(b);
            match (a_tool, b_tool) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => b
                    .relevance
                    .partial_cmp(&a.relevance)
                    .unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        for chunk in sorted {
            let chunk_tokens = chunk.combined_token_count();

            if tokens_used + chunk_tokens <= max_tokens {
                tokens_used += chunk_tokens;
                retained.push(chunk);
            } else {
                match self.strategy {
                    CompactionStrategy::StatusToken => {
                        let summary = Self::generate_status_token(&chunk);
                        status_tokens.push(StatusToken {
                            summary,
                            replaced_tokens: chunk_tokens,
                            replaced_doc_ids: vec![chunk.doc_id],
                        });
                    }
                    CompactionStrategy::Truncate => {}
                    CompactionStrategy::Summarize | CompactionStrategy::LlmSummarize { .. } => {
                        let summary = Self::generate_status_token(&chunk);
                        status_tokens.push(StatusToken {
                            summary,
                            replaced_tokens: chunk_tokens,
                            replaced_doc_ids: vec![chunk.doc_id],
                        });
                    }
                }
            }
        }

        let source_doc_ids = retained.iter().map(|c| c.doc_id).collect();
        CompactedContext {
            retained_chunks: retained,
            status_tokens,
            tokens_used,
            source_doc_ids,
        }
    }

    /// Consolidates multiple context chunks into a single summarized chunk using an external LLM via Ollama.
    ///
    /// Preserves strict provenance tracking in `source_doc_ids`. If the LLM call fails, the error is
    /// returned directly to the caller (no silent fallback to `StatusToken`).
    // ANCHOR[contextra-cognition:COMP-006]
    /// Konsolidiert Chunks per LLM zu einem einzigen Zusammenfassungs-Chunk.
    pub async fn consolidate_via_llm(
        &self,
        tenant: TenantId,
        chunks: &[ContextChunk],
        generator: &(impl LlmTextGenerator + ?Sized),
        _model: &str,
    ) -> Result<CompactedContext> {
        if chunks.is_empty() {
            return Ok(CompactedContext {
                retained_chunks: Vec::new(),
                status_tokens: Vec::new(),
                tokens_used: 0,
                source_doc_ids: Vec::new(),
            });
        }

        let source_doc_ids: Vec<DocId> = chunks.iter().map(|c| c.doc_id).collect();

        let segments: Vec<ContextSegment> = chunks
            .iter()
            .map(|chunk| ContextSegment {
                chunk_id: chunk.doc_id.inner(),
                text: chunk.content.as_str(),
                model_fingerprint: None,
                rope_offset: None,
            })
            .collect();

        let summary_text = generator.generate_with_context(tenant, &segments).await?;
        let consolidated_chunk =
            build_consolidated_chunk_from_summary(summary_text, chunks, &source_doc_ids);
        let tokens_used = consolidated_chunk.combined_token_count();

        Ok(CompactedContext {
            retained_chunks: vec![consolidated_chunk],
            status_tokens: Vec::new(),
            tokens_used,
            source_doc_ids,
        })
    }

    fn is_pinned(chunk: &ContextChunk) -> bool {
        chunk
            .metadata
            .as_ref()
            .and_then(|m| m.get("pinned").or_else(|| m.get("is_pinned")))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    fn is_tool_output(chunk: &ContextChunk) -> bool {
        chunk
            .metadata
            .as_ref()
            .and_then(|m| m.get("tool_output"))
            .is_some()
    }

    fn generate_status_token(chunk: &ContextChunk) -> String {
        let preview: String = chunk.content.chars().take(80).collect();
        format!(
            "[Kompaktiert: {} Tokens — {}...]",
            chunk.token_count, preview
        )
    }

    /// Führt eine Konsolidierung durch und wiederholt bei OCC-Konflikten automatisch.
    ///
    /// # DOKUMENTATION / HINWEIS
    /// Der genutzte RNG ist NICHT kryptographisch und darf nicht für Schlüssel oder Salze genutzt werden.
    #[deprecated(
        note = "use consolidate_with_retry_with_rng directly with an explicit Rng instance"
    )]
    // ANCHOR[contextra-cognition:COMP-007]
    /// Führt Konsolidierung mit automatischem Retry bei OCC-Konflikten aus (veraltet).
    pub async fn consolidate_with_retry<S, V, G>(
        collection: &Collection<S, V>,
        source_doc_ids: &[DocId],
        target_doc_id: DocId,
        generator: &G,
        max_retries: usize,
    ) -> Result<()>
    where
        S: StorageEngine,
        V: VectorIndex,
        G: LlmTextGenerator + ?Sized,
    {
        let entropy_rng = SeededRng::from_entropy();
        Self::consolidate_with_retry_with_rng(
            collection,
            source_doc_ids,
            target_doc_id,
            generator,
            max_retries,
            &entropy_rng,
        )
        .await
    }

    /// Führt eine Konsolidierung durch und wiederholt bei OCC-Konflikten automatisch mit einem konfigurierbaren RNG.
    ///
    /// # DOKUMENTATION / HINWEIS
    /// Der übergebene RNG dient ausschließlich für Backoff-Jitter und ist NICHT kryptographisch.
    /// Er DARF NICHT für Schlüssel, Salze oder Sicherheits-Tokens verwendet werden.
    // ANCHOR[contextra-cognition:COMP-008]
    /// Führt Konsolidierung mit automatischem Retry und injiziertem RNG aus.
    pub async fn consolidate_with_retry_with_rng<S, V, G>(
        collection: &Collection<S, V>,
        source_doc_ids: &[DocId],
        target_doc_id: DocId,
        generator: &G,
        max_retries: usize,
        rng: &dyn Rng,
    ) -> Result<()>
    where
        S: StorageEngine,
        V: VectorIndex,
        G: LlmTextGenerator + ?Sized,
    {
        let mut attempt = 0usize;
        let current_sources = source_doc_ids.to_vec();
        loop {
            let mut session =
                ConsolidationSession::start(collection, &current_sources, target_doc_id).await?;

            let result = session.execute(generator).await;

            match result {
                Ok(()) => return Ok(()),
                Err(e) if e.is_occ_conflict() && attempt < max_retries => {
                    attempt += 1;
                    let changed = session.refresh().await?;
                    tracing::warn!(
                        attempt,
                        max_retries,
                        changed_docs = ?changed,
                        "OCC-Konflikt bei Konsolidierung — retry"
                    );
                    let base_ms = 10u64 * (1u64 << attempt.min(6));
                    let jitter_ms: u64 = rng.next_u64() % 11;
                    tokio::time::sleep(std::time::Duration::from_millis(base_ms + jitter_ms)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
