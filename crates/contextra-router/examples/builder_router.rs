//! Standalone builder example for `contextra-router` (WP-RT-1.4)
//!
//! Demonstrates fluent RouterEngineBuilder construction with minimal in-example mock ports.

use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{RouterEngineBuilder, RoutingOutcome, SlmProfile};
use contextra_types::{ContextChunk, ContextWindow, EntityId, Result, TokenBudget};
use std::sync::Arc;

struct MinimalSearch;
impl HybridSearchProvider for MinimalSearch {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        _query_embedding: &'a [f32],
        _top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            Ok(vec![ContextChunk {
                doc_id: contextra_types::DocId::new(100),
                content: "Builder example chunk".to_string(),
                relevance: 0.92,
                token_count: 5,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            }])
        })
    }
}

struct MinimalCommunity;
impl CommunityResolver for MinimalCommunity {
    fn get_community<'a>(&'a self, _entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(Some(1)) })
    }
}

struct MinimalPreparer;
impl ContextPreparer for MinimalPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        _budget: &TokenBudget,
        _relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        let total_tokens = chunks.iter().map(|c| c.token_count).sum();
        Ok(ContextWindow {
            chunks,
            total_tokens,
            truncated: false,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let profile = SlmProfile::new(
        "phi-3-mini",
        "http://localhost:8080/mcp",
        vec![1],
        TokenBudget::new(4096, 512),
        0.5,
    );

    let router = RouterEngineBuilder::new()
        .search_provider(Arc::new(MinimalSearch))
        .community_resolver(Arc::new(MinimalCommunity))
        .context_preparer(Arc::new(MinimalPreparer))
        .add_profile(profile)
        .build()?;

    let query_vector = vec![0.2f32; 4];
    let query_text = "How to use the router builder?";

    let decision = router.route(&query_vector, query_text).await?;
    println!("Selected Profile: {}", decision.profile.name);
    println!("Decision ID: {}", decision.decision_id.inner());

    let recorded = router.record_outcome(decision.decision_id, RoutingOutcome::Success);
    println!("Outcome recorded: {}", recorded);

    Ok(())
}
