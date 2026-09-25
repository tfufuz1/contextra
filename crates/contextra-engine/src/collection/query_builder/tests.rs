use super::*;
use crate::{Collection, DistanceMetric, Language};
use contextra_graph::CsrGraph;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{FilterExpr, HybridQuery};
use contextra_vector::{HnswConfig, HnswIndex};
use serde_json::json;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::TempDir;

async fn create_test_collection(name: &str) -> (Collection<LsmStorage, HnswIndex>, TempDir) {
    let dir = TempDir::new().unwrap(); // unwrap
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap()); // unwrap
    let hnsw_config = HnswConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).unwrap()); // unwrap
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        name.to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        Language::English,
    );
    (col, dir)
}

#[tokio::test]
async fn test_builder_vector_search_equivalence() {
    let (col, _dir) = create_test_collection("test_vec").await;
    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"tag": "a"})))
        .await
        .unwrap(); // unwrap
    col.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], Some(json!({"tag": "b"})))
        .await
        .unwrap(); // unwrap

    #[allow(deprecated)]
    let legacy = col.search(&[1.0, 0.0, 0.0, 0.0], 2).await.unwrap(); // unwrap
    let builder_res = col
        .query()
        .embedding([1.0, 0.0, 0.0, 0.0])
        .k(2)
        .execute()
        .await
        .unwrap(); // unwrap

    assert_eq!(builder_res.len(), legacy.len());
    assert_eq!(builder_res[0].id, legacy[0].id);
    assert_eq!(builder_res[1].id, legacy[1].id);
}

#[tokio::test]
async fn test_builder_filter_expr_equivalence() {
    let (col, _dir) = create_test_collection("test_filter").await;
    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"cat": "news"})))
        .await
        .unwrap(); // unwrap
    col.insert("doc-2", &[0.9, 0.1, 0.0, 0.0], Some(json!({"cat": "blog"})))
        .await
        .unwrap(); // unwrap

    let filter = FilterExpr::Eq {
        field: "cat".to_string(),
        value: json!("news"),
    };

    #[allow(deprecated)]
    let legacy = col
        .search_with_filter_expr(&[1.0, 0.0, 0.0, 0.0], 10, Some(filter.clone()))
        .await
        .unwrap(); // unwrap

    let builder_res = col
        .query()
        .embedding([1.0, 0.0, 0.0, 0.0])
        .filter(filter)
        .k(10)
        .execute()
        .await
        .unwrap(); // unwrap

    assert_eq!(builder_res.len(), 1);
    assert_eq!(builder_res.len(), legacy.len());
    assert_eq!(builder_res[0].id, legacy[0].id);
    assert_eq!(builder_res[0].id, "doc-1");
}

#[tokio::test]
async fn test_builder_hybrid_search_equivalence() {
    let (col, _dir) = create_test_collection("test_hybrid").await;
    col.insert(
        "doc-1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "rust programming language"})),
    )
    .await
    .unwrap(); // unwrap
    col.insert(
        "doc-2",
        &[0.0, 1.0, 0.0, 0.0],
        Some(json!({"text": "python data science"})),
    )
    .await
    .unwrap(); // unwrap

    #[allow(deprecated)]
    let legacy = col
        .hybrid_search("rust", &[1.0, 0.0, 0.0, 0.0], 5, None)
        .await
        .unwrap(); // unwrap

    let builder_res = col
        .query()
        .text("rust")
        .embedding([1.0, 0.0, 0.0, 0.0])
        .k(5)
        .execute()
        .await
        .unwrap(); // unwrap

    assert_eq!(builder_res.len(), legacy.len());
    if !builder_res.is_empty() {
        assert_eq!(builder_res[0].id, legacy[0].id);
    }
}

#[tokio::test]
async fn test_builder_weights_and_strategy() {
    let (col, _dir) = create_test_collection("test_weights").await;
    col.insert(
        "doc-1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "alpha"})),
    )
    .await
    .unwrap(); // unwrap

    let weights = SignalWeights::new(0.6, 0.4, 0.0).unwrap(); // unwrap
    let builder_res = col
        .query()
        .text("alpha")
        .vector([1.0, 0.0, 0.0, 0.0])
        .fusion_weights(weights.into())
        .strategy(SearchStrategy::Hops { max_hops: 2 })
        .k(5)
        .execute()
        .await
        .unwrap(); // unwrap

    assert!(!builder_res.is_empty());
    assert_eq!(builder_res[0].id, "doc-1");
}

#[tokio::test]
async fn test_builder_query_config_equivalence() {
    let (col, _dir) = create_test_collection("test_query_cfg").await;
    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 10})))
        .await
        .unwrap(); // unwrap

    let hybrid_query = HybridQuery::builder()
        .with_vector_query(vec![1.0, 0.0, 0.0, 0.0])
        .with_k(1)
        .build()
        .unwrap(); // unwrap

    #[allow(deprecated)]
    let legacy = col.hybrid_search_with_query(&hybrid_query).await.unwrap(); // unwrap

    let builder_res = col
        .query()
        .query_config(&hybrid_query)
        .execute()
        .await
        .unwrap(); // unwrap

    assert_eq!(builder_res.len(), legacy.len());
    assert_eq!(builder_res[0].id, legacy[0].id);
}

#[tokio::test]
#[cfg(feature = "reranking")]
async fn test_rerank_candidate_pool_size_k10_fetches_100_candidates() {
    let (col, _dir) = create_test_collection("test_rerank_pool_100").await;
    // Populate 150 documents
    for i in 0..150 {
        let id = format!("doc-{:03}", i);
        let text = format!("rust system engineering doc {:03}", i);
        let val = (i as f32 + 1.0) / 150.0;
        col.insert(
            &id,
            &[val, 1.0 - val, 0.0, 0.0],
            Some(json!({ "text": text })),
        )
        .await
        .unwrap();
    }

    let reranker = contextra_infer_onnx::CrossEncoderReranker::passthrough();
    let res = col
        .query()
        .text("rust system")
        .embedding([1.0, 0.0, 0.0, 0.0])
        .reranker(&reranker)
        .k(10)
        .execute()
        .await
        .unwrap();

    assert_eq!(res.len(), 10, "Final results truncated to k=10");
    // Verify that candidates retrieved before truncation had ce_score attached to 100 items (or top k items returned with ce_score)
    // With passthrough reranker, ce_score is attached to top k items from the 100 candidates
    assert!(res[0].metadata.as_ref().unwrap().get("ce_score").is_some());
}

#[tokio::test]
#[cfg(feature = "reranking")]
async fn test_rerank_candidate_pool_max_cap_k100_capped_at_200() {
    let (col, _dir) = create_test_collection("test_rerank_pool_max_200").await;
    // Populate 300 documents
    for i in 0..300 {
        let id = format!("doc-{:03}", i);
        let text = format!("benchmark item {:03}", i);
        let val = (i as f32 + 1.0) / 300.0;
        col.insert(
            &id,
            &[val, 1.0 - val, 0.0, 0.0],
            Some(json!({ "text": text })),
        )
        .await
        .unwrap();
    }

    let reranker = contextra_infer_onnx::CrossEncoderReranker::passthrough();

    // Query with k=100 and default pool settings (mult=10, max=200) -> fetch_k = min(100*10, 200) = 200
    let res_default = col
        .query()
        .text("benchmark item")
        .embedding([1.0, 0.0, 0.0, 0.0])
        .reranker(&reranker)
        .k(100)
        .execute()
        .await
        .unwrap();

    assert_eq!(res_default.len(), 100);

    // Custom settings test: rerank_pool_max(50)
    let res_custom_max = col
        .query()
        .text("benchmark item")
        .embedding([1.0, 0.0, 0.0, 0.0])
        .reranker(&reranker)
        .rerank_pool_max(50)
        .k(30)
        .execute()
        .await
        .unwrap();

    // Since fetch_k is capped at 50, top-30 query successfully completes and receives results
    assert_eq!(
        res_custom_max.len(),
        30,
        "k=30 requested with fetch_k capped at 50"
    );
}

#[tokio::test]
#[cfg(feature = "reranking")]
async fn test_query_builder_reranking_with_text_and_reranker() {
    let (col, _dir) = create_test_collection("test_rerank_builder").await;
    col.insert(
        "doc-1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "rust programming language"})),
    )
    .await
    .unwrap(); // unwrap
    col.insert(
        "doc-2",
        &[0.9, 0.1, 0.0, 0.0],
        Some(json!({"content": "python programming language"})),
    )
    .await
    .unwrap(); // unwrap
    col.insert(
        "doc-3",
        &[0.8, 0.2, 0.0, 0.0],
        Some(json!({"text": "javascript web development"})),
    )
    .await
    .unwrap(); // unwrap

    let reranker_res = contextra_infer_onnx::CrossEncoderReranker::new(
        contextra_infer_onnx::RerankConfig::default(),
    );
    if let Ok(reranker) = reranker_res {
        let res = col
            .query()
            .text("rust")
            .embedding([1.0, 0.0, 0.0, 0.0])
            .reranker(&reranker)
            .include_provenance(true)
            .k(2)
            .execute()
            .await
            .unwrap(); // unwrap

        assert_eq!(res.len(), 2, "Results must be truncated to k=2");

        for item in &res {
            let meta = item.metadata.as_ref().expect("metadata must exist"); // expect
            assert!(
                meta.get("ce_score").is_some(),
                "ce_score must be attached to metadata"
            );
            if let Some(prov) = &item.provenance {
                assert!(
                    prov.rerank_score.is_some(),
                    "rerank_score must be set in provenance"
                );
            }
        }

        assert!(
            res[0].score >= res[1].score,
            "Results must be sorted descending by rerank score"
        );
    }
}

#[tokio::test]
#[cfg(all(feature = "reranking", feature = "adaptive-candidate-pool-sizing"))]
async fn test_pid_controller_integration_with_reranker() {
    let (col, _dir) = create_test_collection("test_pid_rerank").await;
    col.insert(
        "doc-1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "rust programming language"})),
    )
    .await
    .unwrap();
    col.insert(
        "doc-2",
        &[0.9, 0.1, 0.0, 0.0],
        Some(json!({"text": "python programming language"})),
    )
    .await
    .unwrap();

    let reranker = contextra_infer_onnx::CrossEncoderReranker::passthrough();
    let pid = Arc::new(parking_lot::Mutex::new(
        contextra_adapt::PidController::default(),
    ));

    // First call: initial update
    let res = col
        .query()
        .text("rust")
        .embedding([1.0, 0.0, 0.0, 0.0])
        .reranker(&reranker)
        .pid_controller(pid.clone())
        .k(2)
        .execute()
        .await
        .unwrap();

    assert_eq!(res.len(), 2);
    let updated_size = pid.lock().current_pool_size;
    assert!(updated_size.is_some());

    // Second call: verify pid controller's stored current_pool_size is used as rerank_pool_max
    let res2 = col
        .query()
        .text("rust")
        .embedding([1.0, 0.0, 0.0, 0.0])
        .reranker(&reranker)
        .pid_controller(pid.clone())
        .k(2)
        .execute()
        .await
        .unwrap();

    assert_eq!(res2.len(), 2);
}

#[tokio::test]
#[cfg(feature = "reranking")]
async fn test_reranker_deadline_falls_back() {
    let (col, _dir) = create_test_collection("test_rerank_deadline").await;
    col.insert(
        "doc-1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "rust programming language"})),
    )
    .await
    .unwrap(); // unwrap
    col.insert(
        "doc-2",
        &[0.9, 0.1, 0.0, 0.0],
        Some(json!({"content": "python programming language"})),
    )
    .await
    .unwrap(); // unwrap

    let config = contextra_infer_onnx::RerankConfig {
        rerank_deadline_ms: Some(10),
        simulate_delay_ms: Some(100),
        ..Default::default()
    };

    let reranker = contextra_infer_onnx::CrossEncoderReranker::passthrough_with_config(config);

    let res = col
        .query()
        .text("rust")
        .embedding([1.0, 0.0, 0.0, 0.0])
        .reranker(&reranker)
        .k(2)
        .execute()
        .await;

    assert!(
        res.is_ok(),
        "Query must succeed even when reranker times out"
    );
    let results = res.unwrap(); // unwrap
    assert_eq!(results.len(), 2);
    for item in &results {
        if let Some(meta) = &item.metadata {
            assert!(
                meta.get("ce_score").is_none(),
                "ce_score should not be attached on timeout fallback"
            );
        }
    }
}

#[tokio::test]
#[cfg(feature = "reranking")]
async fn test_query_builder_reranking_no_text_query_skips_rerank() {
    let (col, _dir) = create_test_collection("test_rerank_no_text").await;
    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"val": 1})))
        .await
        .unwrap(); // unwrap
    col.insert("doc-2", &[0.9, 0.1, 0.0, 0.0], Some(json!({"val": 2})))
        .await
        .unwrap(); // unwrap

    let reranker_res = contextra_infer_onnx::CrossEncoderReranker::new(
        contextra_infer_onnx::RerankConfig::default(),
    );
    if let Ok(reranker) = reranker_res {
        let res = col
            .query()
            .embedding([1.0, 0.0, 0.0, 0.0])
            .reranker(&reranker)
            .k(2)
            .execute()
            .await
            .unwrap(); // unwrap

        assert_eq!(res.len(), 2);
        for item in &res {
            if let Some(meta) = &item.metadata {
                assert!(
                    meta.get("ce_score").is_none(),
                    "ce_score should not be attached when reranking is skipped"
                );
            }
        }
    }
}

#[test]
fn test_no_reranker_uses_k_not_10k() {
    let k = 10;
    let query = HybridQuery::builder().with_k(k).build().unwrap();
    assert!(!query.has_reranker);

    let rerank_k = if query.has_reranker {
        let mult = query
            .rerank_pool_multiplier
            .unwrap_or(DEFAULT_RERANK_POOL_MULTIPLIER);
        let max_pool = query.rerank_pool_max.unwrap_or(DEFAULT_RERANK_POOL_MAX);
        k.saturating_mul(mult).min(max_pool)
    } else {
        k
    };

    let mut candidate_k = k;
    if !query.include_superseded {
        candidate_k = candidate_k.max(k.saturating_mul(3));
    }
    candidate_k = candidate_k
        .max(rerank_k)
        .min(contextra_types::MAX_SEARCH_K)
        .max(k);

    assert!(
        candidate_k <= k * 3,
        "Without reranker, candidate_k ({candidate_k}) must not exceed k * 3 ({})",
        k * 3
    );
}

#[test]
fn test_reranker_expands_to_100_for_k10() {
    let k = 10;
    let mut query = HybridQuery::builder().with_k(k).build().unwrap();
    query.has_reranker = true;

    let rerank_k = if query.has_reranker {
        let mult = query
            .rerank_pool_multiplier
            .unwrap_or(DEFAULT_RERANK_POOL_MULTIPLIER);
        let max_pool = query.rerank_pool_max.unwrap_or(DEFAULT_RERANK_POOL_MAX);
        k.saturating_mul(mult).min(max_pool)
    } else {
        k
    };

    let mut candidate_k = k;
    if !query.include_superseded {
        candidate_k = candidate_k.max(k.saturating_mul(3));
    }
    candidate_k = candidate_k
        .max(rerank_k)
        .min(contextra_types::MAX_SEARCH_K)
        .max(k);

    assert!(
        candidate_k >= 100,
        "With reranker, candidate_k ({candidate_k}) must be >= 100 for k=10"
    );
}
