use contextra_rank::{GlobalFusionConfig, GlobalFusionStrategy, SearchResult};
use contextra_types::ContextraError;
use serde_json::json;

fn make_search_result(id: &str, score: f32, community_id: Option<u64>) -> SearchResult {
    let metadata = community_id.map(|cid| json!({ "community_id": cid }));
    SearchResult {
        id: id.to_string(),
        score,
        metadata,
        matched_signals: vec!["vector".to_string()],
        provenance: None,
    }
}

#[test]
fn test_global_fusion_community_aware_rrf() {
    let config = GlobalFusionConfig {
        max_community_nodes: Some(10),
        min_community_size: Some(2),
        k_rrf: 60.0,
    };
    let strategy = GlobalFusionStrategy::new(config);

    // Community 100: 3 items
    let set_v1 = vec![
        make_search_result("doc1", 0.9, Some(100)),
        make_search_result("doc2", 0.8, Some(100)),
        make_search_result("doc3", 0.7, Some(100)),
    ];
    let set_t1 = vec![
        make_search_result("doc2", 15.0, Some(100)),
        make_search_result("doc1", 12.0, Some(100)),
        make_search_result("doc3", 10.0, Some(100)),
    ];

    // Community 200: 1 item (will be filtered out by min_community_size = 2)
    let set_v2 = vec![make_search_result("doc4", 0.95, Some(200))];

    // Community 300: 2 items
    let set_v3 = vec![
        make_search_result("doc5", 0.85, Some(300)),
        make_search_result("doc6", 0.75, Some(300)),
    ];

    let result_sets = vec![
        ("vector".to_string(), [set_v1, set_v2, set_v3].concat(), 1.0),
        ("text".to_string(), set_t1, 1.0),
    ];

    let fused = strategy.fuse(result_sets, 10).expect("fusion succeeds");

    // Community 200 doc4 excluded due to min_community_size = 2
    assert!(!fused.iter().any(|res| res.id == "doc4"));

    // Remaining items should be doc1, doc2, doc3, doc5, doc6
    let fused_ids: Vec<&str> = fused.iter().map(|res| res.id.as_str()).collect();
    assert!(fused_ids.contains(&"doc1"));
    assert!(fused_ids.contains(&"doc2"));
    assert!(fused_ids.contains(&"doc3"));
    assert!(fused_ids.contains(&"doc5"));
    assert!(fused_ids.contains(&"doc6"));

    // Verify metadata annotation aggregated_topic_result = true
    for res in &fused {
        let meta = res.metadata.as_ref().expect("metadata present");
        assert_eq!(meta.get("aggregated_topic_result"), Some(&json!(true)));
        assert!(meta.get("community_id").is_some());
    }
}

#[test]
fn test_global_fusion_max_community_nodes_cap() {
    let config = GlobalFusionConfig {
        max_community_nodes: Some(2),
        min_community_size: Some(1),
        k_rrf: 60.0,
    };
    let strategy = GlobalFusionStrategy::new(config);

    let candidates = vec![
        make_search_result("doc1", 0.9, Some(100)),
        make_search_result("doc2", 0.8, Some(100)),
        make_search_result("doc3", 0.7, Some(100)),
    ];

    let result_sets = vec![("vector".to_string(), candidates, 1.0)];
    let fused = strategy.fuse(result_sets, 10).expect("fusion succeeds");

    assert_eq!(fused.len(), 2);
}

#[test]
fn test_global_fusion_empty_community_index_degrades_gracefully() {
    let strategy = GlobalFusionStrategy::default();

    // Candidates without any community metadata
    let candidates = vec![
        SearchResult {
            id: "doc1".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc2".to_string(),
            score: 0.8,
            metadata: Some(json!({ "author": "alice" })),
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
    ];

    let result_sets = vec![("vector".to_string(), candidates, 1.0)];
    let err = strategy
        .fuse(result_sets, 10)
        .expect_err("should return InvalidInput error");

    match err {
        ContextraError::InvalidInput(msg) => {
            assert!(
                msg.contains("Community index is empty") || msg.contains("contextra_consolidate")
            );
        }
        other => panic!("expected ContextraError::InvalidInput, got {other:?}"),
    }
}
