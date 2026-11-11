//! # Lifecycle, Validation & Context Preparation Ports (Ring 0 / Class B)
//!
//! LINK crates/contextra-ports/src/lib.rs
//! LINK crates/contextra-ports/src/embedding.rs
//!
//! // ANCHOR[contextra-ports:LIFECYCLE-001]
//! // SECTION lifecycle-validation-contracts
//! // NOTE: Definiert Validierungs- und Budgetverträge für RAG-Pipelines.
//! // Invarianten:
//! // 1. Zero-Panic Doktrin für Validatoren.
//! // 2. BoxFuture dyn-Safety für asynchrone Validatoren.

use super::BoxFuture;
use crate::types::{ContextChunk, ContextWindow, TokenBudget};
use crate::Result;
use serde::{Deserialize, Serialize};

// ANCHOR[contextra-ports:GROUNDING-ASSESS-001]
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

// ANCHOR[contextra-ports:GROUNDING-VAL-001]
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

// ANCHOR[contextra-ports:RESP-GROUNDING-VAL-001]
/// Abstract contract for grounding validation of LLM-generated responses against raw source strings.
pub trait ResponseGroundingValidator: Send + Sync {
    /// Evaluates the grounding score for an LLM-generated response against source text slices.
    /// Returns a float score in [0.0, 1.0].
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei Evaluierungsfehlern oder ungültigen Eingaben.
    /// # Crash
    /// Reiner Rechenvorgang im Arbeitsspeicher; keine Seiteneffekte oder I/O.
    fn score_grounding(&self, response: &str, sources: &[&str]) -> Result<f32>;
}

// ANCHOR[contextra-ports:CTX-PREPARER-001]
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
// !SECTION lifecycle-validation-contracts
