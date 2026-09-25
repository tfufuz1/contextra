//! Hyperedge suggestion and LLM validation module (Spec §13, Part 2).
//!
//! Derives candidate hyperedges from entity co-occurrence in source documents,
//! and provides LLM validation before candidates are committed via `relate_n_ary`.

use crate::error::GraphMutationError;
use contextra_ports::TextGenerator;
use contextra_types::{DocId, EntityId};
use std::collections::BTreeMap;
use thiserror::Error;

/// Maximum allowed participants in a hyperedge candidate (Combinatorial explosion guard, P5).
pub const MAX_RELATE_PARTICIPANTS: usize = 64;

/// A candidate hyperedge derived from document co-occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperEdgeCandidate {
    /// Canonical sorted list of participating entity IDs.
    pub participants: Vec<EntityId>,
    /// Number of distinct documents in which this exact entity combination co-occurs.
    pub co_occurrence_count: usize,
    /// Sorted, deduplicated list of source document IDs where this combination co-occurs.
    pub source_doc_ids: Vec<DocId>,
}

/// A candidate hyperedge after LLM semantic validation.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedHyperEdgeCandidate {
    /// The underlying co-occurrence candidate.
    pub candidate: HyperEdgeCandidate,
    /// Suggested semantic predicate string.
    pub predicate: String,
    /// LLM validation confidence score in [0.0, 1.0].
    pub llm_confidence: f32,
    /// Whether the candidate was accepted as a semantically meaningful relation.
    pub accepted: bool,
}

/// Errors specific to hyperedge suggestion and validation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HyperedgeSuggestError {
    /// Candidate count exceeds the maximum LLM calls allowed per cycle.
    #[error("LLM validation budget exceeded: {0} candidates exceed limit")]
    LlmValidationBudgetExceeded(usize),
}

impl From<HyperedgeSuggestError> for GraphMutationError {
    fn from(err: HyperedgeSuggestError) -> Self {
        GraphMutationError::Internal(err.to_string())
    }
}

/// Computes hyperedge candidates from document entity co-occurrences.
///
/// Purely deterministic function (no `Rng`, P28). For each document, deduplicates
/// and sorts entity IDs. The exact entity combination co-occurring in a document
/// is counted across all input documents. Combinations with total count $< \text{min\_co\_occurrence}$
/// are discarded. Combinations with $> \text{MAX\_RELATE\_PARTICIPANTS}$ (64) participants
/// are discarded (not truncated) to protect against combinatorial explosion.
///
/// Output candidates are sorted deterministically in ascending lexicographical order of `participants`.
pub fn compute_co_occurrence_candidates(
    entities_per_document: &[(DocId, Vec<EntityId>)],
    min_co_occurrence: usize,
) -> Vec<HyperEdgeCandidate> {
    let mut combinations_map: BTreeMap<Vec<EntityId>, Vec<DocId>> = BTreeMap::new();

    for (doc_id, entities) in entities_per_document {
        let mut sorted_entities = entities.clone();
        sorted_entities.sort_unstable_by_key(|e| e.inner());
        sorted_entities.dedup();

        let count = sorted_entities.len();
        if count < 2 || count > MAX_RELATE_PARTICIPANTS {
            continue;
        }

        combinations_map
            .entry(sorted_entities)
            .or_default()
            .push(*doc_id);
    }

    let mut candidates = Vec::new();

    for (participants, mut doc_ids) in combinations_map {
        doc_ids.sort_unstable_by_key(|d| d.inner());
        doc_ids.dedup();

        let co_occurrence_count = doc_ids.len();

        if co_occurrence_count >= min_co_occurrence {
            candidates.push(HyperEdgeCandidate {
                participants,
                co_occurrence_count,
                source_doc_ids: doc_ids,
            });
        }
    }

    candidates.sort_by(|a, b| a.participants.cmp(&b.participants));
    candidates
}

/// Validates hyperedge candidates using an LLM text generator.
///
/// For each candidate (up to `max_llm_calls_per_cycle`), queries the LLM to confirm
/// if the joint occurrence implies a semantically meaningful relation and suggest a predicate.
///
/// # Budget Guard
/// If `candidates.len() > max_llm_calls_per_cycle`, returns `Err(GraphMutationError)`
/// wrapping `HyperedgeSuggestError::LlmValidationBudgetExceeded`.
pub async fn validate_candidates_with_llm(
    candidates: &[HyperEdgeCandidate],
    entity_labels: &dyn Fn(EntityId) -> Option<String>,
    generator: &dyn TextGenerator,
    max_llm_calls_per_cycle: usize,
) -> Result<Vec<ValidatedHyperEdgeCandidate>, GraphMutationError> {
    if candidates.len() > max_llm_calls_per_cycle {
        return Err(HyperedgeSuggestError::LlmValidationBudgetExceeded(candidates.len()).into());
    }

    let mut validated = Vec::with_capacity(candidates.len());

    for candidate in candidates {
        let labels_str = candidate
            .participants
            .iter()
            .map(|&id| {
                entity_labels(id)
                    .map(|l| format!("{l} (ID: {})", id.inner()))
                    .unwrap_or_else(|| format!("Entity({})", id.inner()))
            })
            .collect::<Vec<_>>()
            .join(", ");

        let prompt = format!(
            "Do the following entities form a meaningful semantic relationship?\nEntities: [{labels_str}]\nReply with 'YES: <predicate>' or 'NO'."
        );

        let response = generator
            .generate_text(&prompt)
            .await
            .map_err(|e| GraphMutationError::Internal(e.to_string()))?;

        let trimmed = response.trim();
        let upper = trimmed.to_uppercase();

        let (accepted, predicate, confidence) = if upper.starts_with("NO")
            || upper.contains("NOT MEANINGFUL")
            || upper.contains("FALSE")
            || upper.contains("INVALID")
            || trimmed.is_empty()
        {
            (false, String::new(), 0.0f32)
        } else if let Some(rest) = trimmed.strip_prefix("YES:") {
            let pred = rest.trim().to_string();
            let final_pred = if pred.is_empty() {
                "co_occurrence".to_string()
            } else {
                pred
            };
            (true, final_pred, 0.9f32)
        } else if trimmed.starts_with("YES") || trimmed.starts_with("yes") {
            (true, "co_occurrence".to_string(), 0.9f32)
        } else {
            (true, trimmed.to_string(), 0.8f32)
        };

        validated.push(ValidatedHyperEdgeCandidate {
            candidate: candidate.clone(),
            predicate,
            llm_confidence: confidence,
            accepted,
        });
    }

    Ok(validated)
}
