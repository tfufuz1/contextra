//! Deprecated: temporary local port traits have moved to `memfuse-ports`.

#[deprecated(
    since = "0.1.0",
    note = "Moved to memfuse_ports as part of Ring-Modell Welle 2 (siehe docs/refactor/router-db-edge-audit.md)"
)]
pub use memfuse_ports::{
    CommunityResolver, ContextChunk, ContextPreparer, ContextWindow, DriftStatusProvider,
    HybridSearchProvider, TokenBudget,
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
