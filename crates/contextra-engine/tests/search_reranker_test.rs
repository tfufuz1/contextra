#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig};
use contextra_ports::{BoxFuture, RerankResult, Reranker};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug)]
struct DeterministicTestReranker;

impl Reranker for DeterministicTestReranker {
    fn rerank<'a>(
        &'a self,
        _query: &'a str,
        candidates: &'a [String],
    ) -> BoxFuture<'a, contextra_types::Result<Vec<RerankResult>>> {
        Box::pin(async move {
            let mut results = Vec::with_capacity(candidates.len());
            for (i, _) in candidates.iter().enumerate() {
                let score = (candidates.len() - i) as f32 * 1.5;
                results.push(RerankResult {
                    index: i,
                    original_index: i,
                    score,
                    calibrated_score: score,
                });
            }
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            Ok(results)
        })
    }

    fn rerank_deadline_ms(&self) -> Option<u64> {
        Some(1000)
    }

    fn record_implicit_feedback(&self, _evaluated_candidates: &[RerankResult], _top_k: usize) {}
}

#[tokio::test]
async fn test_search_with_deterministic_reranker() -> contextra_types::Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Arc::new(Contextra::open_with_config(tmp_dir.path(), config).await?);
    let col = db.collection("default").await?;

    db.insert("doc_1", &[0.1, 0.2, 0.3, 0.4], Some(json!({"text": "Alpha document content"}))).await?;
    db.insert("doc_2", &[0.2, 0.3, 0.4, 0.5], Some(json!({"text": "Beta document content"}))).await?;

    let reranker = DeterministicTestReranker;

    let search_results = col
        .query()
        .text("document")
        .k(10)
        .reranker(&reranker)
        .execute()
        .await?;

    assert!(!search_results.is_empty(), "Search results should not be empty");

    for res in &search_results {
        assert!(!res.score.is_nan(), "Score for document {} must not be NaN", res.id);
        assert!(!res.score.is_infinite(), "Score for document {} must be finite", res.id);
    }

    let db_owned = Arc::try_unwrap(db)
        .map_err(|_| contextra_types::ContextraError::Internal("Arc unwrap failed".into()))?;
    db_owned.close().await?;
    Ok(())
}
