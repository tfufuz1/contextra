//! QueryRewriter port trait for Multi-Step Retrieval query expansion.

use crate::BoxFuture;
use contextra_types::{QueryRewriteOutput, Result, ScoredEntry};

/// Trait for query rewriting in multi-step iterative retrieval (LLM-agnostic).
pub trait QueryRewriter: Send + Sync {
    /// Generates alternative text sub-queries based on original query and previous search results.
    ///
    /// For backward compatibility, default implementation calls `rewrite_structured` and collects non-empty text queries.
    fn rewrite<'a>(
        &'a self,
        original_query: &'a str,
        current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, Result<Vec<String>>> {
        Box::pin(async move {
            let outputs = self.rewrite_structured(original_query, current_results).await?;
            let text_queries = outputs
                .into_iter()
                .filter_map(|out| out.text_query)
                .filter(|q| !q.trim().is_empty())
                .collect();
            Ok(text_queries)
        })
    }

    /// Generates multi-signal query reformulations (text, semantic, and graph anchors) for multi-step retrieval.
    ///
    /// Default implementation delegates to `rewrite` and wraps the resulting string queries as `QueryRewriteOutput::text_only`.
    fn rewrite_structured<'a>(
        &'a self,
        original_query: &'a str,
        current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, Result<Vec<QueryRewriteOutput>>> {
        Box::pin(async move {
            let queries = self.rewrite(original_query, current_results).await?;
            let outputs = queries
                .into_iter()
                .map(QueryRewriteOutput::text_only)
                .collect();
            Ok(outputs)
        })
    }
}
