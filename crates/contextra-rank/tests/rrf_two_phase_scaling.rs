use contextra_rank::{
    fuse_search_results_with_strategy, weighted_reciprocal_rank_fusion,
    weighted_reciprocal_rank_fusion_mrrf, weighted_reciprocal_rank_fusion_with_options,
    DriftStatus, MetadataMergePriority, SearchResult, SignalCalibrationContext,
};
use contextra_types::FusionStrategy;
use std::time::Instant;

fn make_synthetic_result_sets(
    count_per_signal: usize,
    overlap_offset: usize,
) -> Vec<(String, Vec<SearchResult>, f32)> {
    let signals = vec![
        ("vector".to_string(), 1.0_f32),
        ("text".to_string(), 0.8_f32),
        ("graph".to_string(), 0.5_f32),
    ];

    signals
        .into_iter()
        .enumerate()
        .map(|(sig_idx, (sig_name, weight))| {
            let start = sig_idx * overlap_offset;
            let results = (0..count_per_signal)
                .map(|i| {
                    let doc_num = start + i;
                    SearchResult {
                        id: format!("doc_{doc_num}"),
                        score: 1.0 / (1.0 + i as f32),
                        metadata: Some(serde_json::json!({
                            "sig": sig_name.as_str(),
                            "idx": i,
                        })),
                        matched_signals: vec![sig_name.clone()],
                        provenance: None,
                    }
                })
                .collect();
            (sig_name, results, weight)
        })
        .collect()
}

#[test]
fn benchmark_rrf_scaling_50k_and_400k() {
    let max_results = 50;

    // 1. Benchmark 50,000 candidates
    let sets_50k = make_synthetic_result_sets(25_000, 12_500);
    let start_50k = Instant::now();
    let res_50k = weighted_reciprocal_rank_fusion(sets_50k, max_results);
    let dur_50k = start_50k.elapsed();

    assert_eq!(res_50k.len(), max_results);

    println!("[BENCHMARK 50K Candidates]");
    println!("  Time: {:?}", dur_50k);

    // 2. Benchmark 400,000 candidates
    let sets_400k = make_synthetic_result_sets(200_000, 100_000);
    let start_400k = Instant::now();
    let res_400k = weighted_reciprocal_rank_fusion(sets_400k, max_results);
    let dur_400k = start_400k.elapsed();

    assert_eq!(res_400k.len(), max_results);

    println!("[BENCHMARK 400K Candidates]");
    println!("  Time: {:?}", dur_400k);
}

#[test]
fn test_differential_rrf_correctness_scenarios() {
    // Test 1: Complex multi-signal setup with metadata and provenance
    let set_vec = vec![
        SearchResult {
            id: "doc_1".to_string(),
            score: 0.9,
            metadata: Some(serde_json::json!({"tag": "vector", "score_v": 0.9})),
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc_2".to_string(),
            score: 0.8,
            metadata: Some(serde_json::json!({"tag": "vector", "score_v": 0.8})),
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc_3".to_string(),
            score: 0.7,
            metadata: Some(serde_json::json!({"tag": "vector", "score_v": 0.7})),
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
    ];

    let set_text = vec![
        SearchResult {
            id: "doc_2".to_string(),
            score: 12.5,
            metadata: Some(serde_json::json!({"tag": "text", "score_t": 12.5})),
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc_4".to_string(),
            score: 10.0,
            metadata: Some(serde_json::json!({"tag": "text", "score_t": 10.0})),
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc_1".to_string(),
            score: 8.0,
            metadata: Some(serde_json::json!({"tag": "text", "score_t": 8.0})),
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
    ];

    let set_custom = vec![
        SearchResult {
            id: "doc_3".to_string(),
            score: 0.95,
            metadata: Some(serde_json::json!({"custom_meta": 100})),
            matched_signals: vec!["custom_signal".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc_1".to_string(),
            score: 0.5,
            metadata: Some(serde_json::json!({"custom_meta": 50})),
            matched_signals: vec!["custom_signal".to_string()],
            provenance: None,
        },
    ];

    let input_sets = vec![
        ("vector".to_string(), set_vec, 1.0_f32),
        ("text".to_string(), set_text, 0.8_f32),
        ("custom_signal".to_string(), set_custom, 0.6_f32),
    ];

    let cal_ctxs = vec![
        SignalCalibrationContext::new(
            "vector",
            true,
            DriftStatus::Stable { mean_shift: 0.01 },
        ),
        SignalCalibrationContext::new("text", true, DriftStatus::Stable { mean_shift: 0.02 }),
    ];

    // Standard options fusion
    let res_options = weighted_reciprocal_rank_fusion_with_options(
        input_sets.clone(),
        10,
        MetadataMergePriority::VectorFirst,
        true,
        None,
    );

    assert_eq!(res_options.len(), 4);
    assert_eq!(res_options[0].id, "doc_1"); // doc_1 is rank 1 in vec, rank 3 in text, rank 2 in custom
    assert_eq!(res_options[1].id, "doc_2");
    assert_eq!(res_options[2].id, "doc_3");
    assert_eq!(res_options[3].id, "doc_4");

    // Check provenance presence
    for r in &res_options {
        assert!(r.provenance.is_some());
    }

    // mRRF calibrated fusion
    let res_mrrf = weighted_reciprocal_rank_fusion_mrrf(
        input_sets.clone(),
        10,
        MetadataMergePriority::TextFirst,
        true,
        None,
        Some(&cal_ctxs),
        0.5,
    );
    assert_eq!(res_mrrf.len(), 4);

    // Strategy fusion
    let res_strat = fuse_search_results_with_strategy(
        input_sets,
        10,
        MetadataMergePriority::VectorFirst,
        true,
        None,
        FusionStrategy::Rrf,
    );
    assert_eq!(res_strat.len(), 4);
}
