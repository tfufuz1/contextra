//! Integration tests for SAOS types in `contextra-types` (Task J34).
//!
//! Verifies that all 13 `HybridQueryBuilder` (`QueryOptionsBuilder`) setters produce
//! expected configuration fields when built via the public production construction path.

use contextra_types::{
    EntityId, FilterExpr, FusionStrategy, FusionWeights, GraphTraversalStrategy, HybridQuery,
    MemoryType, OnSignalFailure, QueryOptions, QueryOptionsBuilder, SignalFusionStrategies,
};

#[test]
fn test_hybrid_query_builder_saos_all_13_setters_wired() {
    let filter = FilterExpr::Eq {
        field: "category".to_string(),
        value: serde_json::json!("science"),
    };
    let weights = FusionWeights::new(0.5, 0.3, 0.2).expect("valid weights");
    let fusion_strat = SignalFusionStrategies {
        vector: FusionStrategy::ScoreNormalized,
        text: FusionStrategy::Rrf,
        graph: FusionStrategy::Rrf,
    };
    let graph_strat = GraphTraversalStrategy::Hops { max_hops: 4 };

    // Invoke builder setter methods on HybridQuery::builder()
    let query: HybridQuery = HybridQuery::builder()
        .with_text_query("quantum computing")
        .with_vector_query(vec![0.1, 0.2, 0.3])
        .with_graph_start_node("node_alpha")
        .with_graph_strategy(graph_strat.clone())
        .with_fusion_weights(weights.clone())
        .with_fusion_strategy(fusion_strat)
        .with_filter(filter.clone())
        .with_same_community_as(EntityId::new(42))
        .with_memory_type_filter(vec![MemoryType::Semantic, MemoryType::Procedural])
        .with_include_superseded(true)
        .with_include_provenance(true)
        .with_rerank_pool_multiplier(15)
        .with_rerank_pool_max(300)
        .with_on_signal_failure(OnSignalFailure::Degrade)
        .with_k(20)
        .build()
        .expect("successful build");

    // Assert all 13 symbols are correctly populated
    assert_eq!(query.text_query.as_deref(), Some("quantum computing"));
    assert_eq!(query.graph_start_node.as_deref(), Some("node_alpha"));
    assert_eq!(query.graph_strategy, graph_strat);
    assert_eq!(query.fusion_weights, weights);
    assert_eq!(query.fusion_strategy, fusion_strat);
    assert_eq!(query.filter, Some(filter));
    assert_eq!(query.same_community_as, Some(EntityId::new(42)));
    assert_eq!(
        query.memory_type_filter,
        Some(vec![MemoryType::Semantic, MemoryType::Procedural])
    );
    assert!(query.include_superseded);
    assert!(query.include_provenance);
    assert_eq!(query.rerank_pool_multiplier, Some(15));
    assert_eq!(query.rerank_pool_max, Some(300));
    assert_eq!(query.on_signal_failure, OnSignalFailure::Degrade);
    assert_eq!(query.k, 20);
}

#[test]
fn test_query_options_builder_alias_parity() {
    // Verifies alias QueryOptionsBuilder / QueryOptions parity
    let options: QueryOptions = QueryOptionsBuilder::new()
        .with_text_query("alias search")
        .with_graph_start_node("seed_1")
        .with_graph_strategy(GraphTraversalStrategy::Hops { max_hops: 2 })
        .with_fusion_strategy(FusionStrategy::ScoreNormalized)
        .with_include_superseded(false)
        .with_include_provenance(true)
        .with_rerank_pool_multiplier(5)
        .with_rerank_pool_max(100)
        .with_on_signal_failure(OnSignalFailure::Fail)
        .build()
        .expect("options build ok");

    assert_eq!(options.text_query.as_deref(), Some("alias search"));
    assert_eq!(options.graph_start_node.as_deref(), Some("seed_1"));
    assert_eq!(
        options.graph_strategy,
        GraphTraversalStrategy::Hops { max_hops: 2 }
    );
    assert_eq!(
        options.fusion_strategy,
        SignalFusionStrategies::uniform(FusionStrategy::ScoreNormalized)
    );
    assert!(!options.include_superseded);
    assert!(options.include_provenance);
    assert_eq!(options.rerank_pool_multiplier, Some(5));
    assert_eq!(options.rerank_pool_max, Some(100));
    assert_eq!(options.on_signal_failure, OnSignalFailure::Fail);
}
