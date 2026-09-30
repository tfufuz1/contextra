use contextra_rank::fusion::{
    fuse_search_results_with_strategy, MetadataMergePriority, SearchResult,
};
use contextra_types::FusionStrategy;

fn make_doc(id: &str, score: f32) -> SearchResult {
    SearchResult {
        id: id.to_string(),
        score,
        metadata: None,
        matched_signals: vec![],
        provenance: None,
    }
}

#[test]
fn test_score_normalized_fallback_empty_signal_list() {
    // (a) Active signal with an empty list when total_active_signals > 1
    let result_sets = vec![
        (
            "vector".to_string(),
            vec![make_doc("doc1", 0.9), make_doc("doc2", 0.5)],
            1.0,
        ),
        ("text".to_string(), vec![], 1.0),
    ];

    let norm_res = fuse_search_results_with_strategy(
        result_sets.clone(),
        10,
        MetadataMergePriority::default(),
        true,
        None,
        FusionStrategy::ScoreNormalized,
    );

    let rrf_res = fuse_search_results_with_strategy(
        result_sets,
        10,
        MetadataMergePriority::default(),
        true,
        None,
        FusionStrategy::Rrf,
    );

    assert_eq!(norm_res.len(), rrf_res.len());
    for (norm_doc, rrf_doc) in norm_res.iter().zip(rrf_res.iter()) {
        assert_eq!(norm_doc.id, rrf_doc.id);
        assert_eq!(norm_doc.score, rrf_doc.score);
        assert!(norm_doc.score.is_finite());
    }
}

#[test]
fn test_score_normalized_fallback_constant_scores() {
    // (b) Signal with constant scores (MinMax degenerated max_s <= min_s)
    let result_sets = vec![
        (
            "vector".to_string(),
            vec![make_doc("doc1", 0.5), make_doc("doc2", 0.5)],
            1.0,
        ),
        (
            "text".to_string(),
            vec![make_doc("doc2", 0.8), make_doc("doc3", 0.3)],
            1.0,
        ),
    ];

    let norm_res = fuse_search_results_with_strategy(
        result_sets.clone(),
        10,
        MetadataMergePriority::default(),
        true,
        None,
        FusionStrategy::ScoreNormalized,
    );

    let rrf_res = fuse_search_results_with_strategy(
        result_sets,
        10,
        MetadataMergePriority::default(),
        true,
        None,
        FusionStrategy::Rrf,
    );

    assert_eq!(norm_res.len(), rrf_res.len());
    for (norm_doc, rrf_doc) in norm_res.iter().zip(rrf_res.iter()) {
        assert_eq!(norm_doc.id, rrf_doc.id);
        assert_eq!(norm_doc.score, rrf_doc.score);
        assert!(norm_doc.score.is_finite());
    }
}

#[test]
fn test_score_normalized_fallback_non_finite_scores() {
    // (c) Signal with NaN or Infinity doc scores
    let nan_sets = vec![
        (
            "vector".to_string(),
            vec![make_doc("doc1", f32::NAN), make_doc("doc2", 0.7)],
            1.0,
        ),
        (
            "text".to_string(),
            vec![make_doc("doc1", 0.8), make_doc("doc3", 0.4)],
            1.0,
        ),
    ];

    let norm_nan_res = fuse_search_results_with_strategy(
        nan_sets.clone(),
        10,
        MetadataMergePriority::default(),
        true,
        None,
        FusionStrategy::ScoreNormalized,
    );

    let rrf_nan_res = fuse_search_results_with_strategy(
        nan_sets,
        10,
        MetadataMergePriority::default(),
        true,
        None,
        FusionStrategy::Rrf,
    );

    assert_eq!(norm_nan_res.len(), rrf_nan_res.len());
    for (norm_doc, rrf_doc) in norm_nan_res.iter().zip(rrf_nan_res.iter()) {
        assert_eq!(norm_doc.id, rrf_doc.id);
        assert_eq!(norm_doc.score, rrf_doc.score);
        assert!(norm_doc.score.is_finite());
    }

    let inf_sets = vec![
        (
            "vector".to_string(),
            vec![make_doc("doc1", f32::INFINITY), make_doc("doc2", 0.7)],
            1.0,
        ),
        (
            "text".to_string(),
            vec![make_doc("doc1", 0.8), make_doc("doc3", 0.4)],
            1.0,
        ),
    ];

    let norm_inf_res = fuse_search_results_with_strategy(
        inf_sets.clone(),
        10,
        MetadataMergePriority::default(),
        true,
        None,
        FusionStrategy::ScoreNormalized,
    );

    let rrf_inf_res = fuse_search_results_with_strategy(
        inf_sets,
        10,
        MetadataMergePriority::default(),
        true,
        None,
        FusionStrategy::Rrf,
    );

    assert_eq!(norm_inf_res.len(), rrf_inf_res.len());
    for (norm_doc, rrf_doc) in norm_inf_res.iter().zip(rrf_inf_res.iter()) {
        assert_eq!(norm_doc.id, rrf_doc.id);
        assert_eq!(norm_doc.score, rrf_doc.score);
        assert!(norm_doc.score.is_finite());
    }
}
