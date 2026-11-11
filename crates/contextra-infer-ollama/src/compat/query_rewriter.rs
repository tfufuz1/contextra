//! Port-Bridge für QueryRewriter.

use crate::query_rewriter::OllamaQueryRewriter;
use contextra_ports::{BoxFuture, QueryRewriter};
use contextra_types::{QueryRewriteOutput, ScoredEntry};

impl QueryRewriter for OllamaQueryRewriter {
    fn rewrite_structured<'a>(
        &'a self,
        original_query: &'a str,
        current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, contextra_types::Result<Vec<QueryRewriteOutput>>> {
        Box::pin(async move {
            let snippets: Vec<String> = current_results
                .iter()
                .take(self.config().max_context_results)
                .map(extract_snippet)
                .collect();
            let snippet_refs: Vec<&str> = snippets.iter().map(|s| s.as_str()).collect();

            let outputs = self
                .rewrite(original_query, &snippet_refs)
                .await
                .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

            Ok(outputs.into_iter().map(Into::into).collect())
        })
    }
}

impl OllamaQueryRewriter {
    /// Rewrites a query structured according to contextra-ports QueryRewriter contract.
    pub fn rewrite_structured<'a>(
        &'a self,
        original_query: &'a str,
        current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, contextra_types::Result<Vec<QueryRewriteOutput>>> {
        QueryRewriter::rewrite_structured(self, original_query, current_results)
    }
}

fn extract_snippet(entry: &ScoredEntry) -> String {
    if let Some(meta) = &entry.metadata {
        if let Some(text) = meta
            .get("text")
            .or_else(|| meta.get("content"))
            .and_then(|v| v.as_str())
        {
            return text.to_string();
        }
        if let Ok(json_str) = serde_json::to_string(meta) {
            return json_str;
        }
    }
    String::new()
}
