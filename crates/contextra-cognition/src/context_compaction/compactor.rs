use contextra_core::{
    ContextChunk, ContextSegment, DocId, LlmTextGenerator, Result, StorageEngine,
    TenantId, TokenBudget, VectorIndex,
};
use contextra_engine::collection::Collection;
use contextra_engine::ProvenanceRecord;

use super::session::ConsolidationSession;
use super::types::{CompactedContext, CompactionStrategy, StatusToken};

/// Context Compaction Engine.
#[derive(Debug)]
pub struct ContextCompactor {
    budget: TokenBudget,
    strategy: CompactionStrategy,
}

impl ContextCompactor {
    /// Creates a new `ContextCompactor` with given `TokenBudget` and `CompactionStrategy`.
    pub fn new(budget: TokenBudget, strategy: CompactionStrategy) -> Self {
        Self { budget, strategy }
    }

    /// Kompaktiert eine Liste von Chunks auf das Token-Budget.
    ///
    /// Priorisiert nach Relevanz-Score. Tool-Output-Chunks (erkennbar an Metadata-Key "tool_output")
    /// werden zuerst kompaktiert.
    pub fn compact(&self, chunks: Vec<ContextChunk>) -> CompactedContext {
        let _source_doc_ids: Vec<DocId> = chunks.iter().map(|c| c.doc_id).collect();
        let max_tokens = self.budget.available();
        let mut tokens_used = 0;
        let mut retained = Vec::new();
        let mut status_tokens = Vec::new();

        // Sortierung: Tool-Outputs ans Ende (werden zuerst kompaktiert)
        let mut sorted = chunks;
        sorted.sort_by(|a, b| {
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
                // Kompaktierung
                match self.strategy {
                    CompactionStrategy::StatusToken => {
                        let summary = Self::generate_status_token(&chunk);
                        status_tokens.push(StatusToken {
                            summary,
                            replaced_tokens: chunk_tokens,
                            replaced_doc_ids: vec![chunk.doc_id],
                        });
                    }
                    CompactionStrategy::Truncate => {
                        // Chunk wird verworfen
                    }
                    CompactionStrategy::Summarize | CompactionStrategy::LlmSummarize { .. } => {
                        // Synchronous fallback in compact(): Status-Token.
                        // For full async LLM summarization, call consolidate_via_llm().
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

    // AI-TAG[SMELL][MINOR][RESOLVED] Async LLM-Summarization for context compaction (ID: AGT-DB-004) (TS:2026-08-28T00:00:00Z)
    /// Consolidates multiple context chunks into a single summarized chunk using an external LLM via Ollama.
    ///
    /// Preserves strict provenance tracking in `source_doc_ids`. If the LLM call fails, the error is
    /// returned directly to the caller (no silent fallback to `StatusToken`).
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

        let mut source_doc_ids = Vec::with_capacity(chunks.len());
        let mut prompt_content = String::new();

        for chunk in chunks {
            source_doc_ids.push(chunk.doc_id);
            prompt_content.push_str(&format!(
                "- Chunk [DocId: {}]: {}\n",
                chunk.doc_id.0, chunk.content
            ));
        }

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

        let estimated_tokens = crate::context::ContextManager::estimate_tokens(&summary_text);

        // Combine metadata if present
        let mut combined_metadata = serde_json::Map::new();
        combined_metadata.insert("llm_summarized".to_string(), serde_json::Value::Bool(true));
        combined_metadata.insert(
            "source_doc_count".to_string(),
            serde_json::Value::Number(chunks.len().into()),
        );

        let prov = ProvenanceRecord::synthesized_from(&source_doc_ids);
        if let Ok(prov_val) = serde_json::to_value(&prov) {
            combined_metadata.insert("provenance".to_string(), prov_val);
        }

        // Generate a distinct deterministic DocId from the combination of source doc_ids
        let synthesized_doc_id = {
            let key = chunks
                .iter()
                .map(|c| c.doc_id.inner().to_string())
                .collect::<Vec<_>>()
                .join(":");
            DocId::from_key(&format!("contextra:consolidated:{key}")).unwrap_or(chunks[0].doc_id)
        };

        let max_relevance = chunks.iter().fold(0.0f32, |max, c| max.max(c.relevance));

        let consolidated_chunk = ContextChunk {
            doc_id: synthesized_doc_id,
            content: summary_text,
            relevance: max_relevance,
            token_count: estimated_tokens,
            metadata: Some(serde_json::Value::Object(combined_metadata)),
            contextual_prefix: None,
            links: Vec::new(),
        };

        let tokens_used = consolidated_chunk.combined_token_count();

        Ok(CompactedContext {
            retained_chunks: vec![consolidated_chunk],
            status_tokens: Vec::new(),
            tokens_used,
            source_doc_ids,
        })
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
    /// # Parameter
    /// - `collection`: die Ziel-Collection
    /// - `source_doc_ids`: DocIds der zu konsolidierenden Dokumente
    /// - `target_doc_id`: Ziel-DocId für das konsolidierte Dokument
    /// - `generator`: LLM-Provider für die Zusammenfassung
    /// - `max_retries`: maximale Wiederholungsanzahl bei OCC-Konflikten (empfohlen: 3)
    ///
    /// # OCC-Retry-Protokoll
    /// Bei einem Konflikt ruft refresh() die veränderten DocIds ab.
    /// Falls nur ein Teil der Source-Dokumente geändert wurde, wird nur dieser Teil
    /// neu zusammengefasst (delta re-summarization).
    /// Exponentieller Backoff: 10ms → 20ms → 40ms (jitter: +/- 5ms)
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
                    // Exponentieller Backoff mit Jitter
                    let base_ms = 10u64 * (1u64 << attempt.min(6));
                    let jitter_ms: u64 = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.subsec_nanos() as u64 % 11)
                        .unwrap_or(0);
                    tokio::time::sleep(std::time::Duration::from_millis(base_ms + jitter_ms)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
