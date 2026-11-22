// FILE-CONTEXT Header (Format v3)
// ZWECK: Read-only tenant context retrieval tool implementing AgentTool.
// INVARIANTEN: Read-only execution against tenant collections; cost estimation; zero state mutation.
// STAND: TS:2026-10-29T00:00:00Z

//! Read-only tenant context retrieval tool.

use crate::context::AgentContext;
use crate::step::{AgentTool, StepResult};
use contextra_ports::BoxFuture;
use contextra_types::Result;
use serde_json::json;

/// Read-only tenant context retrieval tool implementing [`AgentTool`].
///
/// Searches documents within the tenant's collection context without mutating state.
#[derive(Debug, Clone)]
pub struct RetrieveTool {
    name: String,
}

impl Default for RetrieveTool {
    fn default() -> Self {
        Self {
            name: "retrieve".to_string(),
        }
    }
}

impl RetrieveTool {
    /// Creates a new `RetrieveTool` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `RetrieveTool` with a custom handler name.
    pub fn with_name(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl AgentTool for RetrieveTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn estimated_cost(&self, _input: &serde_json::Value) -> usize {
        10
    }

    #[allow(clippy::cast_possible_truncation)]
    fn execute<'a>(
        &'a self,
        ctx: &'a AgentContext,
        input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            let query = input
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();

            let target_col_name = input
                .get("collection")
                .and_then(|v| v.as_str())
                .unwrap_or(ctx.state_collection.name());

            let limit = input.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

            let coll = ctx.db.collection(target_col_name).await?;

            let raw_results = coll.scan_prefix("", Some(limit)).await?;
            let mut matches = Vec::new();

            for (key, val) in raw_results {
                let text_repr = val.to_string();
                if query.is_empty() || text_repr.contains(query) || key.contains(query) {
                    matches.push(json!({
                        "key": key,
                        "value": val
                    }));
                }
                if matches.len() >= limit {
                    break;
                }
            }

            Ok(StepResult {
                node_id: ctx.current_node.clone(),
                output: json!({
                    "query": query,
                    "collection": target_col_name,
                    "count": matches.len(),
                    "results": matches
                }),
                tokens_consumed: 10 + matches.len() * 2,
                next_edge: None,
            })
        })
    }

    fn timeout_ms(&self) -> u64 {
        10_000
    }

    fn is_retriable(&self) -> bool {
        true
    }

    fn max_retries(&self) -> u32 {
        2
    }
}
