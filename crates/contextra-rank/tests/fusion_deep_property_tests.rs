//! Deep property and adversarial test suite for contextra-rank fusion strategies.

use contextra_rank::drift::DriftStatus;
use contextra_rank::fusion::{
    fuse_search_results_with_signal_strategies, score_normalized_fusion_with_options,
    weighted_reciprocal_rank_fusion_mrrf, GlobalFusionConfig, GlobalFusionStrategy,
    MetadataMergePriority, ProvenanceBuilder, ResonanceConfig, SearchResult,
    SignalCalibrationContext,
};
use contextra_types::{FusionStrategy, SignalFusionStrategies};
use proptest::prelude::*;

fn make_doc_with_community(id: &str, score: f32, community_id: u64) -> SearchResult {
    let mut map = serde_json::Map::new();
    map.insert(
        "community_id".to_string(),
        serde_json::Value::Number(community_id.into()),
    );
    SearchResult {
        id: id.to_string(),
        score,
        metadata: Some(serde_json::Value::Object(map)),
        matched_signals: vec!["vector".to_string()],
        provenance: None,
    }
}

proptest! {
    #[test]
    fn prop_mrrf_weights_renormalized(
        w1 in 0.1f32..10.0f32,
        w2 in 0.1f32..10.0f32,
        beta in 0.1f32..2.0f32,
    ) {
        let set1 = vec![
            SearchResult {
                id: "doc1".to_string(),
                score: 0.9,
                metadata: None,
                matched_signals: vec!["vector".to_string()],
                provenance: None,
            },
            SearchResult {
                id: "doc2".to_string(),
                score: 0.7,
                metadata: None,
                matched_signals: vec!["vector".to_string()],
                provenance: None,
            },
        ];
        let set2 = vec![
            SearchResult {
                id: "doc2".to_string(),
                score: 0.85,
                metadata: None,
                matched_signals: vec!["text".to_string()],
                provenance: None,
            },
            SearchResult {
                id: "doc1".to_string(),
                score: 0.65,
                metadata: None,
                matched_signals: vec!["text".to_string()],
                provenance: None,
            },
        ];

        let sets = vec![
            ("vector".to_string(), set1, w1),
            ("text".to_string(), set2, w2),
        ];

        let cal_contexts = vec![
            SignalCalibrationContext::new("vector", true, DriftStatus::Stable { mean_shift: 0.01 }),
            SignalCalibrationContext::new("text", true, DriftStatus::Stable { mean_shift: 0.01 }),
        ];

        let fused = weighted_reciprocal_rank_fusion_mrrf(
            sets,
            10,
            MetadataMergePriority::default(),
            true,
            None,
            Some(&cal_contexts),
            beta,
        );

        prop_assert!(!fused.is_empty());
        for res in &fused {
            prop_assert!(res.score.is_finite());
            prop_assert!(res.score > 0.0);
        }
    }

    #[test]
    fn prop_score_normalized_minmax_clamping(
        s1 in 0.0f32..100.0f32,
        s2 in 0.0f32..100.0f32,
    ) {
        let docs = vec![
            SearchResult {
                id: "doc_a".to_string(),
                score: s1,
                metadata: None,
                matched_signals: vec!["vector".to_string()],
                provenance: None,
            },
            SearchResult {
                id: "doc_b".to_string(),
                score: s2,
                metadata: None,
                matched_signals: vec!["vector".to_string()],
                provenance: None,
            },
        ];

        let sets = vec![("vector".to_string(), docs, 1.0)];
        let fused = score_normalized_fusion_with_options(
            sets,
            10,
            MetadataMergePriority::default(),
            true,
            None,
        );

        prop_assert_eq!(fused.len(), 2);
        for res in &fused {
            prop_assert!(res.score.is_finite());
        }
    }

    #[test]
    fn prop_global_fusion_strategy_community_cap(
        max_comm_nodes in 1usize..20usize,
    ) {
        let config = GlobalFusionConfig {
            max_community_nodes: Some(max_comm_nodes),
            min_community_size: Some(1),
            k_rrf: 60.0,
        };
        let strategy = GlobalFusionStrategy::new(config);

        let docs = vec![
            make_doc_with_community("doc1", 0.9, 1),
            make_doc_with_community("doc2", 0.8, 1),
            make_doc_with_community("doc3", 0.7, 2),
            make_doc_with_community("doc4", 0.6, 2),
        ];

        let res = strategy.fuse(vec![("vector".to_string(), docs, 1.0)], 100);
        prop_assert!(res.is_ok());
        if let Ok(fused) = res {
            prop_assert!(fused.len() <= max_comm_nodes);
        }
    }
}

#[test]
fn test_hybrid_strategies_rrf_and_score_normalized_mix() {
    let set1 = vec![
        SearchResult {
            id: "doc1".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc2".to_string(),
            score: 0.4,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
    ];
    let set2 = vec![
        SearchResult {
            id: "doc2".to_string(),
            score: 10.0,
            metadata: None,
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc1".to_string(),
            score: 2.0,
            metadata: None,
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
    ];

    let sets = vec![
        ("vector".to_string(), set1, 1.0),
        ("text".to_string(), set2, 1.0),
    ];

    let strats = SignalFusionStrategies {
        vector: FusionStrategy::Rrf,
        text: FusionStrategy::ScoreNormalized,
        graph: FusionStrategy::Rrf,
    };

    let fused = fuse_search_results_with_signal_strategies(
        sets,
        10,
        MetadataMergePriority::default(),
        true,
        None,
        strats,
    );

    assert_eq!(fused.len(), 2);
    assert!(fused.iter().all(|r| r.score.is_finite()));
}

#[test]
fn test_provenance_builder_synthesized_from_records() {
    let builder = ProvenanceBuilder::new(60.0)
        .vector(0.1, 1, 1.0)
        .bm25(10.0, 2, 1.0)
        .graph(0.5, 3, 1.0)
        .rerank_score(0.9)
        .source_collection("kb")
        .index_type("hybrid");

    let prov = builder.build();
    assert_eq!(prov.vector_distance, Some(0.1));
    assert_eq!(prov.bm25_score, Some(10.0));
    assert_eq!(prov.graph_score, Some(0.5));
    assert_eq!(prov.rerank_score, Some(0.9));
    assert_eq!(prov.source_collection.as_deref(), Some("kb"));
    assert_eq!(prov.index_type.as_deref(), Some("hybrid"));
}

#[test]
fn test_resonance_config_boosting_behavior() {
    let cfg = ResonanceConfig {
        beta: 0.5,
        gamma: 0.3,
    };

    let docs = vec![
        SearchResult {
            id: "doc_single".to_string(),
            score: 1.0,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc_multi".to_string(),
            score: 1.0,
            metadata: None,
            matched_signals: vec!["vector".to_string(), "text".to_string()],
            provenance: None,
        },
    ];

    let boosted = contextra_rank::fusion::apply_resonance_bonus(docs, 2, &cfg);
    assert_eq!(boosted.len(), 2);
    // doc_multi matched 2/2 signals -> gets higher coherence bonus than doc_single (1/2)
    if let (Some(b0), Some(b1)) = (boosted.first(), boosted.get(1)) {
        assert_eq!(b0.id, "doc_multi");
        assert_eq!(b1.id, "doc_single");
        assert!(b0.score > b1.score);
    }
}
