//! Standalone example for `contextra-router` (Building-Block Architecture M10)
//!
//! Demonstrates isolated router engine configuration, profile registration,
//! and request routing with pure mock ports without external storage or database dependencies.

use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{RouterEngine, RoutingOutcome, SlmProfile};
use contextra_types::{ContextChunk, ContextWindow, EntityId, Result, TokenBudget};
use std::sync::Arc;

struct ExampleSearchProvider;

impl HybridSearchProvider for ExampleSearchProvider {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        _query_embedding: &'a [f32],
        _top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            Ok(vec![ContextChunk {
                doc_id: contextra_types::DocId::new(42),
                content: "Contextra isolated router example context".to_string(),
                relevance: 0.95,
                token_count: 6,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            }])
        })
    }
}

struct ExampleCommunityResolver;

impl CommunityResolver for ExampleCommunityResolver {
    fn get_community<'a>(&'a self, _entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(Some(1)) })
    }
}

struct ExampleContextPreparer;

impl ContextPreparer for ExampleContextPreparer {
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

    let search = Arc::new(ExampleSearchProvider);
    let community = Arc::new(ExampleCommunityResolver);
    let preparer = Arc::new(ExampleContextPreparer);

    let router = RouterEngine::new(search, community, preparer, vec![profile], None);

    let query_vector = vec![0.1f32; 4];
    let query_text = "How does isolated routing work in Contextra?";

    println!("Routing query to best SLM profile...");
    let decision = router.route(&query_vector, query_text).await?;

    println!("Selected Profile: {}", decision.profile.name);
    println!("Decision ID: {}", decision.decision_id.inner());
    println!("Total Tokens: {}", decision.context.total_tokens);

    let recorded = router.record_outcome(decision.decision_id, RoutingOutcome::Success);
    println!("Outcome recorded successfully: {}", recorded);

    Ok(())
}
