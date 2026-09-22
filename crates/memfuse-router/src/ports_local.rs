// TODO(welle-3): nach memfuse-ports verschieben, sobald dieses Crate befüllt ist (siehe docs/refactor/router-db-edge-audit.md)
//! Local temporary port traits for decoupling `memfuse-router` from `memfuse-db`.

use memfuse_core::{BoxFuture, ContextChunk, ContextWindow, EntityId, Result, TokenBudget};

/// Contract for resolving graph community assignments for entities.
pub trait CommunityResolver: Send + Sync {
    /// Resolves the optional community ID for a given entity.
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>>;
}

/// Contract for executing hybrid (vector + text) queries for profile routing.
pub trait HybridSearchProvider: Send + Sync {
    /// Executes a hybrid query returning matched context chunks with relevance scores.
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>>;
}

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

/// Default no-op implementation for `CommunityResolver`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopCommunityResolver;

impl CommunityResolver for NoopCommunityResolver {
    fn get_community<'a>(&'a self, _entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(None) })
    }
}

/// Default passthrough implementation for `ContextPreparer`.
#[derive(Debug, Default, Clone, Copy)]
pub struct PassthroughContextPreparer;

impl ContextPreparer for PassthroughContextPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        _budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        let filtered_chunks: Vec<ContextChunk> = chunks
            .into_iter()
            .filter(|c| c.relevance >= relevance_threshold)
            .collect();
        let total_tokens = filtered_chunks.iter().map(|c| c.token_count).sum();
        Ok(ContextWindow {
            chunks: filtered_chunks,
            total_tokens,
            truncated: false,
        })
    }
}

/// Re-export contract for monitoring Lyapunov drift status across active profile watchers (ADR-080).
pub use memfuse_core::DriftStatusProvider;
