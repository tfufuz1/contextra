//! Memory lifecycle management, grounding validator traits, and distance calculator contracts.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: MemoryLifecycleManager, GroundingValidator, ResponseGroundingValidator & DistanceCalculator Trait-Definitionen.
// INVARIANTEN: Zero-panic doctrine, BoxFuture dyn-safety for async validators.

use super::BoxFuture;
use crate::types::{ContextChunk, DocId, TxId};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;

/// Distance calculator trait for vector comparison.
pub trait DistanceCalculator: Send + Sync {
    /// Computes the distance between two f32 vectors.
    fn compute_f32(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Computes the distance between two u8 vectors.
    fn compute_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>;
}

/// Report summarizing statistics of a memory lifecycle sweep operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleSweepReport {
    /// Total number of entries evaluated during the sweep.
    pub swept_count: u64,
    /// Number of entries deleted due to time-to-live (TTL) expiration.
    pub deleted_by_ttl: u64,
    /// Number of entries deleted due to importance score recency decay.
    pub deleted_by_decay: u64,
    /// Number of entries skipped because they are pinned or exempt.
    pub skipped_pinned: u64,
}

/// Actions planned during memory consolidation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ConsolidationAction {
    /// Keep document as is.
    Keep {
        /// ID of the document to keep.
        doc_id: DocId,
    },
    /// Merge two or more documents into a new consolidated entry.
    Merge {
        /// Source document IDs to merge.
        source_ids: Vec<DocId>,
        /// Hint or context summary for the consolidated entry.
        summary_hint: String,
    },
    /// Replace an old document with a new updated entry.
    Supersede {
        /// Old document ID to supersede.
        old_id: DocId,
        /// Replacement document ID.
        new_id: DocId,
    },
    /// Drop a document due to obsolete or low-relevance memory state.
    Drop {
        /// ID of the document to drop.
        doc_id: DocId,
    },
}

/// Trait controlling active Memory Lifecycle management: Decay sweep and Consolidation planning.
///
/// Decouples decision planning (`plan_consolidation`) from execution (`sweep`) for auditability.
pub trait MemoryLifecycleManager: Send + Sync {
    /// Performs a decay and TTL sweep.
    /// Returns a report summarizing deleted, retained, and skipped entries.
    fn sweep(&self, now_tx: TxId) -> impl Future<Output = Result<LifecycleSweepReport>> + Send;

    /// Plans consolidation of similar entries (Mem0 ADD/UPDATE/NOOP pattern).
    /// Returns an action plan without performing automatic execution.
    fn plan_consolidation(
        &self,
        candidates: &[DocId],
    ) -> impl Future<Output = Result<Vec<ConsolidationAction>>> + Send;
}

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
    /// or `Err(MemFuseError)` (e.g. `MemFuseError::PolicyViolation` or low-confidence signal)
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
