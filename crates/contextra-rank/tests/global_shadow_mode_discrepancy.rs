use contextra_rank::{
    weighted_reciprocal_rank_fusion, GlobalFusionConfig, GlobalFusionStrategy, SearchResult,
};
use serde_json::json;

fn make_search_result(id: &str, score: f32, community_id: Option<u64>) -> SearchResult {
    let metadata = community_id.map(|cid| json!({ "community_id": cid }));
    SearchResult {
        id: id.to_string(),
        score,
        metadata,
        matched_signals: vec!["vector".to_string(), "text".to_string()],
        provenance: None,
    }
}

/// ShadowMode Discrepancy Evaluator comparing `Global` fusion against `Hybrid` baseline fusion.
#[test]
fn test_global_shadow_mode_discrepancy_logging() {
    let global_config = GlobalFusionConfig {
        max_community_nodes: Some(10),
        min_community_size: Some(1),
        k_rrf: 60.0,
    };
    let global_strategy = GlobalFusionStrategy::new(global_config);

    // Synthetic candidate distribution across 2 communities
    let candidates_v = vec![
        make_search_result("doc1", 0.95, Some(10)),
        make_search_result("doc2", 0.85, Some(10)),
        make_search_result("doc3", 0.75, Some(20)),
        make_search_result("doc4", 0.65, Some(20)),
    ];

    let candidates_t = vec![
        make_search_result("doc3", 18.0, Some(20)),
        make_search_result("doc1", 15.0, Some(10)),
        make_search_result("doc4", 12.0, Some(20)),
        make_search_result("doc2", 10.0, Some(10)),
    ];

    let result_sets = vec![
        ("vector".to_string(), candidates_v.clone(), 1.0),
        ("text".to_string(), candidates_t.clone(), 1.0),
    ];

    // 1. Baseline evaluation using Hybrid (flat weighted RRF)
    let baseline_fused = weighted_reciprocal_rank_fusion(result_sets.clone(), 10);

    // 2. Candidate evaluation using Global (Community-aware RRF)
    let global_fused = global_strategy
        .fuse(result_sets, 10)
        .expect("Global fusion in ShadowMode succeeds");

    // 3. Measure discrepancy metrics (Jaccard distance on top-k set & rank overlap)
    let top_k = 3;
    let baseline_top_ids: Vec<&str> = baseline_fused
        .iter()
        .take(top_k)
        .map(|r| r.id.as_str())
        .collect();
    let global_top_ids: Vec<&str> = global_fused
        .iter()
        .take(top_k)
        .map(|r| r.id.as_str())
        .collect();

    let intersection_count = baseline_top_ids
        .iter()
        .filter(|id| global_top_ids.contains(id))
        .count();

    let discrepancy_ratio = 1.0 - (intersection_count as f32 / top_k as f32);

    tracing::info!(
        baseline_top = ?baseline_top_ids,
        global_top = ?global_top_ids,
        discrepancy_ratio,
        "ShadowMode Discrepancy Evaluation: Global vs Hybrid Baseline"
    );

    // Verify both strategies produced results and discrepancy metric is computed cleanly
    assert!(!baseline_fused.is_empty());
    assert!(!global_fused.is_empty());
    assert!(
        discrepancy_ratio >= 0.0 && discrepancy_ratio <= 1.0,
        "discrepancy_ratio must be bounded between 0.0 and 1.0"
    );
}
