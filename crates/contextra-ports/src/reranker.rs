//! Reranker port trait for cross-encoder post-retrieval ranking.

use super::BoxFuture;
use crate::{RerankResult, Result};

/// Trait for post-retrieval cross-encoder reranking models.
pub trait Reranker: Send + Sync {
    /// Reranks candidates for a given query text.
    fn rerank<'a>(
        &'a self,
        query: &'a str,
        candidates: &'a [String],
    ) -> BoxFuture<'a, Result<Vec<RerankResult>>>;

    /// Records implicit feedback or outcome labels for calibration.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und nicht-blockierend.
    /// # Crash
    /// Reiner flüchtiger RAM-Zustand oder No-Op; keine Seiteneffekte.
    fn record_implicit_feedback(&self, _results: &[RerankResult], _implicit_relevant_k: usize) {}

    /// Returns the maximum allowed execution time in milliseconds before timing out, if configured.
    ///
    /// # Fehler
    /// Keine; statische Deadline-Abfrage.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
    fn rerank_deadline_ms(&self) -> Option<u64> {
        Some(500)
    }
}
