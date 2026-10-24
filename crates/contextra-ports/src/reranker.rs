//! Reranker port trait for cross-encoder post-retrieval ranking.

use super::BoxFuture;
use contextra_types::{RerankResult, Result};

/// Trait for post-retrieval cross-encoder reranking models.
pub trait Reranker: Send + Sync {
    /// Reranks candidates for a given query text.
    fn rerank<'a>(
        &'a self,
        query: &'a str,
        candidates: &'a [String],
    ) -> BoxFuture<'a, Result<Vec<RerankResult>>>;

    /// Records implicit feedback or outcome labels for calibration.
    fn record_implicit_feedback(&self, _results: &[RerankResult], _implicit_relevant_k: usize) {}

    /// Returns the maximum allowed execution time in milliseconds before timing out, if configured.
    fn rerank_deadline_ms(&self) -> Option<u64> {
        Some(500)
    }
}
