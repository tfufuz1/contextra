#![allow(clippy::unwrap_used, clippy::expect_used)]
//! # Äquivalenz- und Determinismus-Tests für RouterEngineBuilder (WP-RT-1.4)

use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{RouterEngine, RouterEngineBuilder, SlmProfile};
use contextra_types::{ContextChunk, ContextWindow, EntityId, Result, TokenBudget};
use std::sync::Arc;

struct TestSearch;
impl HybridSearchProvider for TestSearch {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        _query_embedding: &'a [f32],
        _top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            Ok(vec![
                ContextChunk {
                    doc_id: contextra_types::DocId::new(1),
                    content: "alpha".to_string(),
                    relevance: 0.85,
                    token_count: 10,
                    metadata: None,
                    contextual_prefix: None,
                    links: vec![],
                },
                ContextChunk {
                    doc_id: contextra_types::DocId::new(2),
                    content: "beta".to_string(),
                    relevance: 0.65,
                    token_count: 15,
                    metadata: None,
                    contextual_prefix: None,
                    links: vec![],
                },
            ])
        })
    }
}

struct TestCommunity;
impl CommunityResolver for TestCommunity {
    fn get_community<'a>(&'a self, _entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(Some(1)) })
    }
}

struct TestPreparer;
impl ContextPreparer for TestPreparer {
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

// ANCHOR[contextra-router:TST-BLD-001]
#[tokio::test]
async fn test_builder_constructor_equivalence_and_determinism() {
    let profiles = vec![
        SlmProfile::new(
            "p1",
            "http://localhost:8001",
            vec![1],
            TokenBudget::new(2048, 256),
            0.5,
        ),
        SlmProfile::new(
            "p2",
            "http://localhost:8002",
            vec![2],
            TokenBudget::new(4096, 512),
            0.8,
        ),
    ];

    let search: Arc<dyn HybridSearchProvider> = Arc::new(TestSearch);
    let comm: Arc<dyn CommunityResolver> = Arc::new(TestCommunity);
    let prep: Arc<dyn ContextPreparer> = Arc::new(TestPreparer);

    let engine_ctor = RouterEngine::new(
        search.clone(),
        comm.clone(),
        prep.clone(),
        profiles.clone(),
        None,
    );

    let engine_builder = RouterEngineBuilder::new()
        .search_provider(search)
        .community_resolver(comm)
        .context_preparer(prep)
        .profiles(profiles)
        .build()
        .expect("builder build failed");

    let query_vector = vec![0.1f32; 8];
    let query_text = "test equivalence query";

    let decision_ctor = engine_ctor.route(&query_vector, query_text).await.unwrap();
    let decision_bld = engine_builder.route(&query_vector, query_text).await.unwrap();

    assert_eq!(decision_ctor.profile.name, decision_bld.profile.name);
    assert_eq!(decision_ctor.decision_id, decision_bld.decision_id);
    assert_eq!(decision_ctor.context.total_tokens, decision_bld.context.total_tokens);
    assert_eq!(decision_ctor.confidence, decision_bld.confidence);
}
