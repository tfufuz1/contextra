//! Closure tests for SAOS symbols in `contextra-types` (Task J34 closure).
//!
//! Verifies that all 13 `HybridQueryBuilder` (`QueryOptionsBuilder`) setters produce
//! expected configuration fields when built via the public production construction path.

use contextra_types::{
    EntityId, FilterExpr, FusionStrategy, FusionWeights, GraphTraversalStrategy, HybridQuery,
    MemoryType, OnSignalFailure, QueryOptions, QueryOptionsBuilder, SignalFusionStrategies,
};

#[test]
fn test_j34closure_hybrid_query_builder_all_13_setters() {
    let filter = FilterExpr::Eq {
        field: "department".to_string(),
        value: serde_json::json!("research"),
    };
    let weights = FusionWeights::new(0.6, 0.3, 0.1).expect("valid weights");
    let fusion_strat = SignalFusionStrategies {
        vector: FusionStrategy::ScoreNormalized,
        text: FusionStrategy::Rrf,
        graph: FusionStrategy::Rrf,
    };
    let graph_strat = GraphTraversalStrategy::Hops { max_hops: 5 };

    let query: HybridQuery = HybridQuery::builder()
        .with_text_query("neural search")
        .with_vector_query(vec![0.5, 0.25, 0.125])
        .with_graph_start_node("node_beta")
        .with_graph_strategy(graph_strat.clone())
        .with_fusion_weights(weights.clone())
        .with_fusion_strategy(fusion_strat)
        .with_filter(filter.clone())
        .with_same_community_as(EntityId::new(99))
        .with_memory_type_filter(vec![MemoryType::Episodic, MemoryType::Semantic])
        .with_include_superseded(true)
        .with_include_provenance(true)
        .with_rerank_pool_multiplier(20)
        .with_rerank_pool_max(150)
        .with_on_signal_failure(OnSignalFailure::Degrade)
        .with_k(15)
        .build()
        .expect("successful query build");

    assert_eq!(query.text_query.as_deref(), Some("neural search"));
    assert_eq!(query.vector_query.as_deref(), Some(&[0.5, 0.25, 0.125][..]));
    assert_eq!(query.graph_start_node.as_deref(), Some("node_beta"));
    assert_eq!(query.graph_strategy, graph_strat);
    assert_eq!(query.fusion_weights, weights);
    assert_eq!(query.fusion_strategy, fusion_strat);
    assert_eq!(query.filter, Some(filter));
    assert_eq!(query.same_community_as, Some(EntityId::new(99)));
    assert_eq!(
        query.memory_type_filter,
        Some(vec![MemoryType::Episodic, MemoryType::Semantic])
    );
    assert!(query.include_superseded);
    assert!(query.include_provenance);
    assert_eq!(query.rerank_pool_multiplier, Some(20));
    assert_eq!(query.rerank_pool_max, Some(150));
    assert_eq!(query.on_signal_failure, OnSignalFailure::Degrade);
    assert_eq!(query.k, 15);
}

#[test]
fn test_j34closure_query_options_builder_alias_parity() {
    let filter = FilterExpr::Eq {
        field: "status".to_string(),
        value: serde_json::json!("active"),
    };
    let weights = FusionWeights::new(0.4, 0.4, 0.2).expect("valid weights");

    let options: QueryOptions = QueryOptionsBuilder::new()
        .with_text_query("closure alias test")
        .with_vector_query(vec![0.1, 0.2])
        .with_graph_start_node("start_1")
        .with_graph_strategy(GraphTraversalStrategy::Hops { max_hops: 2 })
        .with_fusion_weights(weights.clone())
        .with_fusion_strategy(FusionStrategy::ScoreNormalized)
        .with_filter(filter.clone())
        .with_same_community_as(EntityId::new(7))
        .with_memory_type_filter(vec![MemoryType::Working])
        .with_include_superseded(false)
        .with_include_provenance(false)
        .with_rerank_pool_multiplier(8)
        .with_rerank_pool_max(80)
        .with_on_signal_failure(OnSignalFailure::Fail)
        .build()
        .expect("options build ok");

    assert_eq!(options.text_query.as_deref(), Some("closure alias test"));
    assert_eq!(options.vector_query.as_deref(), Some(&[0.1, 0.2][..]));
    assert_eq!(options.graph_start_node.as_deref(), Some("start_1"));
    assert_eq!(
        options.graph_strategy,
        GraphTraversalStrategy::Hops { max_hops: 2 }
    );
    assert_eq!(options.fusion_weights, weights);
    assert_eq!(
        options.fusion_strategy,
        SignalFusionStrategies::uniform(FusionStrategy::ScoreNormalized)
    );
    assert_eq!(options.filter, Some(filter));
    assert_eq!(options.same_community_as, Some(EntityId::new(7)));
    assert_eq!(
        options.memory_type_filter,
        Some(vec![MemoryType::Working])
    );
    assert!(!options.include_superseded);
    assert!(!options.include_provenance);
    assert_eq!(options.rerank_pool_multiplier, Some(8));
    assert_eq!(options.rerank_pool_max, Some(80));
    assert_eq!(options.on_signal_failure, OnSignalFailure::Fail);
}
