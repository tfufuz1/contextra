//! Grounding validator traits and context preparer contracts.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: GroundingValidator, ResponseGroundingValidator & ContextPreparer Trait-Definitionen.
// INVARIANTEN: Zero-panic doctrine, BoxFuture dyn-safety for async validators.

use super::BoxFuture;
use crate::types::{ContextChunk, ContextWindow, TokenBudget};
use crate::Result;
use serde::{Deserialize, Serialize};

/// Result of a post-hoc grounding / attribution validation check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundingAssessment {
    /// Raw or calibrated confidence score in [0.0, 1.0] indicating attribution quality.
    pub score: f32,
    /// Indicates whether the grounding score meets the required threshold.
    pub is_grounded: bool,
    /// Optional detail explanation or ungrounded claims identified.
    pub reason: Option<String>,
}

/// Abstract contract for post-hoc hallucination / grounding validation.
pub trait GroundingValidator: Send + Sync {
    /// Validates an LLM-generated response against retrieval context chunks.
    ///
    /// Returns `Ok(GroundingAssessment)` if confidence meets threshold,
    /// or `Err(ContextraError)` (e.g. `ContextraError::PolicyViolation` or low-confidence signal)
    /// on grounding failure or abstention.
    fn validate_grounding<'a>(
        &'a self,
        response: &'a str,
        context_chunks: &'a [ContextChunk],
    ) -> BoxFuture<'a, Result<GroundingAssessment>>;
}

/// Abstract contract for grounding validation of LLM-generated responses against raw source strings.
pub trait ResponseGroundingValidator: Send + Sync {
    /// Evaluates the grounding score for an LLM-generated response against source text slices.
    /// Returns a float score in [0.0, 1.0].
    fn score_grounding(&self, response: &str, sources: &[&str]) -> Result<f32>;
}

// AI-TAG[ARCH][MINOR][RESOLVED] (virtuell verschoben von contextra-router/ports_local.rs per TODO(welle-3), siehe docs/refactor/router-db-edge-audit.md)
/// Contract for trimming and preparing context windows tailored to token budgets.
pub trait ContextPreparer: Send + Sync {
    /// Prepares and trims context chunks according to the provided token budget and relevance threshold.
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow>;
}
