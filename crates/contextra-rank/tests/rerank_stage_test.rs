use contextra_ports::{BoxFuture, Reranker};
use contextra_rank::{RerankStage, SearchResult};
use contextra_types::{RerankResult, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct MockCountReranker {
    call_count: Arc<AtomicUsize>,
    candidate_count: Arc<AtomicUsize>,
}

impl Reranker for MockCountReranker {
    fn rerank<'a>(
        &'a self,
        _query: &'a str,
        candidates: &'a [String],
    ) -> BoxFuture<'a, Result<Vec<RerankResult>>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        self.candidate_count
            .fetch_add(candidates.len(), Ordering::SeqCst);
        Box::pin(async move {
            Ok(candidates
                .iter()
                .enumerate()
                .map(|(i, _)| RerankResult {
                    index: i,
                    original_index: i,
                    score: 0.9 - (i as f32 * 0.1),
                    calibrated_score: 0.9 - (i as f32 * 0.1),
                })
                .collect())
        })
    }
}

#[tokio::test]
async fn test_rerank_stage_top_k_only_never_full_traversal() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let candidate_count = Arc::new(AtomicUsize::new(0));
    let reranker: Arc<dyn Reranker> = Arc::new(MockCountReranker {
        call_count: Arc::clone(&call_count),
        candidate_count: Arc::clone(&candidate_count),
    });

    let results: Vec<SearchResult> = (0..20)
        .map(|i| SearchResult {
            id: format!("doc_{:02}", i),
            score: 0.5 - (i as f32 * 0.01),
            metadata: Some(serde_json::json!({"text": format!("content_{}", i)})),
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        })
        .collect();

    let stage = RerankStage::new(Some(reranker), 3).with_max_seq(42);
    assert_eq!(stage.max_seq(), Some(42));

    let reranked = stage.execute("query", results).await.unwrap();

    assert_eq!(reranked.len(), 3, "Output must be truncated to top_k");
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        candidate_count.load(Ordering::SeqCst),
        3,
        "Reranker must only process top_k candidates, never full traversal"
    );
}

#[tokio::test]
async fn test_rerank_stage_tie_breaking_determinism() {
    // Candidates with equal score must be resolved deterministically by doc ID ascending
    let results = vec![
        SearchResult {
            id: "doc_z".to_string(),
            score: 0.8,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        },
        SearchResult {
            id: "doc_a".to_string(),
            score: 0.8,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        },
        SearchResult {
            id: "doc_m".to_string(),
            score: 0.8,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        },
    ];

    let stage = RerankStage::new(None, 10);
    let sorted = stage.execute("query", results).await.unwrap();

    assert_eq!(sorted[0].id, "doc_a");
    assert_eq!(sorted[1].id, "doc_m");
    assert_eq!(sorted[2].id, "doc_z");
}

#[tokio::test]
async fn test_rerank_stage_without_reranker() {
    let results = vec![
        SearchResult {
            id: "doc_2".to_string(),
            score: 0.5,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        },
        SearchResult {
            id: "doc_1".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        },
    ];

    let stage = RerankStage::new(None, 1);
    let res = stage.execute("query", results).await.unwrap();

    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, "doc_1");
}
