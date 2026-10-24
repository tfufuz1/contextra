// FILE-CONTEXT
// ZWECK: Testet Multi-Modal Sub-Queries (Text, Vektor, Graph) im Multi-Step Retrieval mit Mock-Rewriter und Mock-Embedder.

use contextra_db::multistep::{MultiStepConfig, MultiStepEngine};
use contextra_db::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{BoxFuture, QueryRewriteOutput, QueryRewriter, TextEmbeddingEngine};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{Result, ScoredEntry};
use contextra_vector::{HnswConfig, HnswIndex};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

struct MultiModalMockRewriter {
    outputs: Vec<Vec<QueryRewriteOutput>>,
}

impl QueryRewriter for MultiModalMockRewriter {
    fn rewrite_structured<'a>(
        &'a self,
        _original_query: &'a str,
        _current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, Result<Vec<QueryRewriteOutput>>> {
        Box::pin(async move {
            if !self.outputs.is_empty() {
                Ok(self.outputs[0].clone())
            } else {
                Ok(vec![])
            }
        })
    }
}

struct FailingMockRewriter;

impl QueryRewriter for FailingMockRewriter {
    fn rewrite_structured<'a>(
        &'a self,
        _original_query: &'a str,
        _current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, Result<Vec<QueryRewriteOutput>>> {
        Box::pin(async move {
            Err(contextra_types::ContextraError::Internal(
                "Rewriter failed".into(),
            ))
        })
    }
}

struct MockEmbedder {
    dimension: usize,
}

impl TextEmbeddingEngine for MockEmbedder {
    fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>> {
        let dim = self.dimension;
        Box::pin(async move { Ok(vec![1.0; dim]) })
    }
}

async fn create_test_collection() -> Arc<Collection<LsmStorage>> {
    let dir = tempdir().expect("tempdir");
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("lsm storage"));
    let hnsw_config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).expect("hnsw index"));
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    Arc::new(col)
}

#[tokio::test]
async fn test_all_three_modalities_handled() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 1.0, 1.0, 1.0],
        Some(serde_json::json!({"text": "rust hybrid search"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 2,
        quality_threshold: 0.99, // Force round 2
        min_quality_hits: 2,
        latency_budget_ms: 1000.0,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);

    let rewriter = MultiModalMockRewriter {
        outputs: vec![vec![QueryRewriteOutput {
            text_query: Some("rust search".to_string()),
            semantic_query: Some("semantic query".to_string()),
            anchor_entities: vec!["1".to_string()],
        }]],
    };
    let embedder = MockEmbedder { dimension: 4 };

    let result = engine
        .search_with_embedder(
            "rust",
            &[1.0, 1.0, 1.0, 1.0],
            5,
            Some(&rewriter),
            Some(&embedder),
        )
        .await
        .expect("search");

    assert_eq!(result.rounds_executed, 2);
    assert_eq!(result.sub_queries, vec!["rust search"]);
    assert_eq!(result.sub_vectors.len(), 1);
    assert_eq!(result.sub_vectors[0], vec![1.0, 1.0, 1.0, 1.0]);
    assert_eq!(result.sub_rewrites.len(), 1);
}

#[tokio::test]
async fn test_fallback_on_rewriter_error() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust language"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 2,
        quality_threshold: 0.99,
        min_quality_hits: 2,
        latency_budget_ms: 1000.0,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);
    let rewriter = FailingMockRewriter;

    let result = engine
        .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
        .await
        .expect("search should fallback on rewriter error");

    assert_eq!(result.rounds_executed, 2);
    // Fallback to original query
    assert_eq!(result.sub_queries, vec!["rust"]);
    assert!(!result.results.is_empty());
}

#[tokio::test]
async fn test_determinism_same_inputs() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust language"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 2,
        quality_threshold: 0.99,
        min_quality_hits: 2,
        latency_budget_ms: 1000.0,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);

    let rewriter1 = MultiModalMockRewriter {
        outputs: vec![vec![QueryRewriteOutput::text_only("rust lang")]],
    };
    let rewriter2 = MultiModalMockRewriter {
        outputs: vec![vec![QueryRewriteOutput::text_only("rust lang")]],
    };

    let res1 = engine
        .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter1))
        .await
        .expect("search 1");

    let res2 = engine
        .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter2))
        .await
        .expect("search 2");

    assert_eq!(res1.sub_queries, res2.sub_queries);
    assert_eq!(res1.results.len(), res2.results.len());
    if !res1.results.is_empty() && !res2.results.is_empty() {
        assert_eq!(res1.results[0].id, res2.results[0].id);
    }
}
