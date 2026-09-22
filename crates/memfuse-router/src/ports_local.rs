//! Deprecated: temporary local port traits have moved to `memfuse-ports`.

#[deprecated(
    since = "0.1.0",
    note = "Moved to memfuse_ports as part of Ring-Modell Welle 2 (siehe docs/refactor/router-db-edge-audit.md)"
)]
pub use memfuse_ports::{
    CommunityResolver, ContextPreparer, DriftStatusProvider, HybridSearchProvider,
};

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;

    #[test]
    fn test_ports_local_reexports() {
        fn _assert_community_resolver<T: CommunityResolver + ?Sized>() {}
        fn _assert_context_preparer<T: ContextPreparer + ?Sized>() {}
        fn _assert_hybrid_search_provider<T: HybridSearchProvider + ?Sized>() {}
        fn _assert_drift_status_provider<T: DriftStatusProvider + ?Sized>() {}
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
