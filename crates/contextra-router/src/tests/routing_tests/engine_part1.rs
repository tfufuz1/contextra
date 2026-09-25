use super::super::fixtures::*;
use crate::{RouterEngine, SlmProfile};
use contextra_db::{Contextra, ContextraConfig};
use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, EntityId, TokenBudget};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_router_engine_instantiation_with_mock_storage() {
    let storage = Arc::new(MockStorageEngine);
    let mut hnsw_config = contextra_index::HnswConfig::default();
    hnsw_config.dimension = 4;
    let hnsw = Arc::new(contextra_index::HnswIndex::try_new(hnsw_config).unwrap());
    let graph = Arc::new(contextra_graph::CsrGraph::new());
    let next_tx = Arc::new(std::sync::atomic::AtomicU64::new(1));
    let collection = Arc::new(contextra_db::Collection::new(
        "test_collection".to_string(),
        storage,
        hnsw,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    ));

    let profile = SlmProfile::new(
        "mock-slm",
        "http://localhost:8000",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );

    let adapter = Arc::new(CollectionAdapter::new(collection));
    let preparer = Arc::new(TestContextPreparer);
    let router: RouterEngine =
        RouterEngine::new(adapter.clone(), adapter, preparer, vec![profile], None);
    assert_eq!(router.profiles().len(), 1);
    assert_eq!(router.profiles()[0].name, "mock-slm");
}

#[tokio::test]
async fn test_route_deterministic_community_assignment() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    // Insert documents for two distinct domains/communities
    let vec_coding = vec![1.0, 0.0, 0.0, 0.0];
    let vec_docs = vec![0.0, 1.0, 0.0, 0.0];

    let coding_key = "coding_entity_1";
    let docs_key = "docs_entity_1";

    collection
        .insert(
            coding_key,
            &vec_coding,
            Some(json!({"text": "function rust_code() { return 42; }"})),
        )
        .await
        .unwrap(); // unwrap

    collection
        .insert(
            docs_key,
            &vec_docs,
            Some(json!({"text": "Dokumentation über Unternehmensrichtlinien."})),
        )
        .await
        .unwrap(); // unwrap

    // Relate entities to assign/update graph state and test get_community persistence
    let eid_coding = EntityId::from_key(coding_key).unwrap(); // unwrap
    let eid_docs = EntityId::from_key(docs_key).unwrap(); // unwrap

    collection
        .relate(coding_key, docs_key, "references")
        .await
        .unwrap(); // unwrap

    // Manually persist synthetic community IDs in storage for testing get_community:
    // 100 for coding, 200 for docs
    let tx = db.allocate_tx().unwrap(); // unwrap

    let comm_key_coding = format!("__graph:community:{}", eid_coding.inner()).into_bytes();
    let comm_key_docs = format!("__graph:community:{}", eid_docs.inner()).into_bytes();

    db.inner_storage()
        .put(tx, &comm_key_coding, &serde_json::to_vec(&100u64).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap
    db.inner_storage()
        .put(tx, &comm_key_docs, &serde_json::to_vec(&200u64).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap
    db.inner_storage().commit(tx).await.unwrap(); // unwrap

    let coding_profile = SlmProfile::new(
        "coding-slm",
        "http://localhost:9999/mcp",
        vec![100],
        TokenBudget::new(1000, 100),
        0.01,
    );

    let docs_profile = SlmProfile::new(
        "docs-slm",
        "http://localhost:9999/mcp",
        vec![200],
        TokenBudget::new(1000, 100),
        0.01,
    );

    let router = create_test_router(
        collection.clone(),
        vec![coding_profile.clone(), docs_profile.clone()],
        None,
    );

    // Query coding
    let decision_coding = router
        .route(&vec_coding, "rust_code")
        .await
        .expect("Routing coding"); // expect

    assert_eq!(decision_coding.profile.name, "coding-slm");
    assert!(!decision_coding.context.chunks.is_empty());

    // Query docs
    let decision_docs = router
        .route(&vec_docs, "Unternehmensrichtlinien")
        .await
        .expect("Routing docs"); // expect

    assert_eq!(decision_docs.profile.name, "docs-slm");
    assert!(!decision_docs.context.chunks.is_empty());
}

#[tokio::test]
async fn test_route_fallback_error_on_low_relevance() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    let vec_unrelated = vec![0.0, 0.0, 0.0, 1.0];
    collection
        .insert(
            "unrelated_doc",
            &vec_unrelated,
            Some(json!({"text": "unrelated content"})),
        )
        .await
        .unwrap(); // unwrap

    // High threshold profile that won't be met
    let strict_profile = SlmProfile::new(
        "strict-slm",
        "http://localhost:9999/mcp",
        vec![999], // non-existent community
        TokenBudget::new(1000, 100),
        0.99, // unreachable score threshold
    );

    let router = create_test_router(collection.clone(), vec![strict_profile], None);

    let result = router.route(&[0.1, 0.1, 0.1, 0.1], "search").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ContextraError::NotFound(msg) => {
            assert!(msg.contains("min_relevance_score") || msg.contains("Community-Zuordnung"));
        }
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_route_hot_reload_concurrent_safety() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    let vec_data = vec![1.0, 0.0, 0.0, 0.0];
    let key = "entity_1";
    collection
        .insert(key, &vec_data, Some(json!({"text": "sample text content"})))
        .await
        .unwrap(); // unwrap

    let eid = EntityId::from_key(key).unwrap(); // unwrap
    let tx = db.allocate_tx().unwrap(); // unwrap
    let comm_key = format!("__graph:community:{}", eid.inner()).into_bytes();
    db.inner_storage()
        .put(tx, &comm_key, &serde_json::to_vec(&10u64).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap
    db.inner_storage().commit(tx).await.unwrap(); // unwrap

    let profile_v1 = SlmProfile::new(
        "slm-v1",
        "http://localhost:8001/mcp",
        vec![10],
        TokenBudget::new(1000, 100),
        0.01,
    );

    let router = Arc::new(create_test_router(collection, vec![profile_v1], None));

    // Spawn 20 reader tasks continuously calling route()
    let mut handles = Vec::new();
    for _ in 0..20 {
        let r = router.clone();
        let vec_c = vec_data.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..50 {
                let res = r.route(&vec_c, "sample text content").await;
                assert!(res.is_ok());
                let decision = res.unwrap(); // unwrap
                assert!(
                    decision.profile.name == "slm-v1" || decision.profile.name == "slm-v2",
                    "Unexpected profile name: {}",
                    decision.profile.name
                );
            }
        }));
    }

    // Spawn background writer updating profiles dynamically
    let r_writer = router.clone();
    let writer_handle = tokio::spawn(async move {
        for i in 0..50 {
            let name = if i % 2 == 0 { "slm-v1" } else { "slm-v2" };
            let p = SlmProfile::new(
                name,
                "http://localhost:8001/mcp",
                vec![10],
                TokenBudget::new(1000, 100),
                0.01,
            );
            r_writer.update_profiles(vec![p]);
            tokio::task::yield_now().await;
        }
    });

    for h in handles {
        h.await.unwrap(); // unwrap
    }
    writer_handle.await.unwrap(); // unwrap
}

#[tokio::test]
async fn test_route_hot_reload_atomic_snapshot_determinism() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    let vec_data = vec![1.0, 0.0, 0.0, 0.0];
    let key = "entity_1";
    collection
        .insert(key, &vec_data, Some(json!({"text": "test content"})))
        .await
        .unwrap(); // unwrap

    let eid = EntityId::from_key(key).unwrap(); // unwrap
    let tx = db.allocate_tx().unwrap(); // unwrap
    let comm_key = format!("__graph:community:{}", eid.inner()).into_bytes();
    db.inner_storage()
        .put(tx, &comm_key, &serde_json::to_vec(&42u64).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap
    db.inner_storage().commit(tx).await.unwrap(); // unwrap

    let initial_profiles = vec![
        SlmProfile::new(
            "profile-a",
            "http://localhost/a",
            vec![42],
            TokenBudget::new(500, 50),
            0.01,
        ),
        SlmProfile::new(
            "profile-b",
            "http://localhost/b",
            vec![42],
            TokenBudget::new(500, 50),
            0.01,
        ),
    ];

    let router = create_test_router(collection, initial_profiles, None);

    // Pre-reload decision: deterministic tie-breaking picks profile-a (lower index 0)
    let d1 = router.route(&vec_data, "test content").await.unwrap(); // unwrap
    assert_eq!(d1.profile.name, "profile-a");

    // Hot reload profile configuration with new single profile
    let updated_profiles = vec![SlmProfile::new(
        "profile-c",
        "http://localhost/c",
        vec![42],
        TokenBudget::new(500, 50),
        0.01,
    )];
    router.update_profiles(updated_profiles);

    let d2 = router.route(&vec_data, "test content").await.unwrap(); // unwrap
    assert_eq!(d2.profile.name, "profile-c");
}
#[tokio::test]
async fn test_route_empty_profiles_err() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    let router = create_test_router(collection, vec![], None);
    let err = router.route(&[1.0, 0.0, 0.0, 0.0], "test").await;
    assert!(matches!(err, Err(ContextraError::NotFound(msg)) if msg.contains("Keine SLM-Profile")));
}

#[tokio::test]
async fn test_route_empty_search_results_err() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    let profile = SlmProfile::new(
        "slm",
        "http://localhost:9999/mcp",
        vec![1],
        TokenBudget::new(1000, 100),
        0.01,
    );
    let router = create_test_router(collection, vec![profile], None);

    let err = router.route(&[1.0, 0.0, 0.0, 0.0], "test").await;
    assert!(
        matches!(err, Err(ContextraError::NotFound(msg)) if msg.contains("Keine relevanten Suchergebnisse"))
    );
}

#[tokio::test]
async fn test_route_unparseable_entity_id() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    // Key that does not conform to EntityId format
    collection
        .insert(
            "plain_document_without_entity_id_prefix",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "plain doc text"})),
        )
        .await
        .unwrap(); // unwrap

    let profile = SlmProfile::new(
        "slm",
        "http://localhost:9999/mcp",
        vec![100],
        TokenBudget::new(1000, 100),
        0.0,
    );

    let router = create_test_router(collection, vec![profile], None);
    let result = router.route(&[1.0, 0.0, 0.0, 0.0], "plain doc").await;
    // Unparseable entity ID results in comm_id = None, which fails community matching for profile
    assert!(matches!(result, Err(ContextraError::NotFound(_))));
}

#[tokio::test]
async fn test_route_threshold_boundaries() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    let coding_key = "coding_entity_1";
    collection
        .insert(
            coding_key,
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "rust code text"})),
        )
        .await
        .unwrap(); // unwrap

    let eid = EntityId::from_key(coding_key).unwrap(); // unwrap
    let tx = db.allocate_tx().unwrap(); // unwrap
    let comm_key = format!("__graph:community:{}", eid.inner()).into_bytes();
    db.inner_storage()
        .put(tx, &comm_key, &serde_json::to_vec(&100u64).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap
    db.inner_storage().commit(tx).await.unwrap(); // unwrap

    let profile = SlmProfile::new(
        "slm-threshold",
        "http://localhost:9999/mcp",
        vec![100],
        TokenBudget::new(1000, 100),
        0.0, // Low min threshold to guarantee selection
    );

    let router = create_test_router(collection, vec![profile], None);
    let res = router.route(&[1.0, 0.0, 0.0, 0.0], "rust code").await;
    assert!(res.is_ok());
}

#[test]
fn test_route_determinism_and_tie_breaking() -> Result<(), Box<dyn std::error::Error>> {
    use crate::router::select_profile_from_chunks;
    use contextra_types::{ContextChunk, DocId};

    let profile_0 = SlmProfile::new(
        "profile-0",
        "http://localhost/0",
        vec![10],
        TokenBudget::new(1000, 100),
        0.1,
    );

    let profile_1 = SlmProfile::new(
        "profile-1",
        "http://localhost/1",
        vec![10],
        TokenBudget::new(1000, 100),
        0.1,
    );

    let profile_2 = SlmProfile::new(
        "profile-2",
        "http://localhost/2",
        vec![10],
        TokenBudget::new(1000, 100),
        0.1,
    );

    let profiles = vec![profile_0, profile_1, profile_2];
    let chunks = vec![(
        ContextChunk {
            doc_id: DocId::new(1),
            content: "identical score chunk".to_string(),
            relevance: 0.5,
            token_count: 5,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        },
        Some(10),
    )];

    for _ in 0..100 {
        let selected_idx = select_profile_from_chunks(&profiles, &chunks)?;
        // Lower profile index (0) must always win tie-breaks
        assert_eq!(selected_idx, 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_route_1_and_50_profiles() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    let vec_data = vec![1.0, 0.0, 0.0, 0.0];
    let key = "entity_1";
    collection
        .insert(key, &vec_data, Some(json!({"text": "sample text"})))
        .await
        .unwrap(); // unwrap

    let eid = EntityId::from_key(key).unwrap(); // unwrap
    let tx = db.allocate_tx().unwrap(); // unwrap
    let comm_key = format!("__graph:community:{}", eid.inner()).into_bytes();
    db.inner_storage()
        .put(tx, &comm_key, &serde_json::to_vec(&100u64).unwrap()) // unwrap
        .await
        .unwrap(); // unwrap
    db.inner_storage().commit(tx).await.unwrap(); // unwrap

    let profiles_50: Vec<_> = (0..50)
        .map(|i| {
            SlmProfile::new(
                format!("profile-{}", i),
                format!("http://localhost:8000/mcp/{}", i),
                vec![100],
                TokenBudget::new(1000, 100),
                0.0,
            )
        })
        .collect();

    let router = try_create_test_router(collection, profiles_50, None).unwrap(); // unwrap
    let decision = router.route(&vec_data, "sample text").await.unwrap(); // unwrap
    assert_eq!(decision.profile.name, "profile-0");

    let update_res = router.try_update_profiles(vec![SlmProfile::new(
        "profile-single",
        "http://localhost/single",
        vec![100],
        TokenBudget::new(1000, 100),
        0.0,
    )]);
    assert!(update_res.is_ok());

    let decision_single = router.route(&vec_data, "sample text").await.unwrap(); // unwrap
    assert_eq!(decision_single.profile.name, "profile-single");
}

#[tokio::test]
async fn test_router_engine_profiles_accessor() {
    let dir = tempfile::tempdir().unwrap(); // unwrap
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap(); // unwrap
    let collection = db.collection("default").await.unwrap(); // unwrap

    let profile = SlmProfile::new(
        "p-acc",
        "http://localhost/mcp",
        vec![1],
        TokenBudget::new(100, 10),
        0.1,
    );
    let router = create_test_router(collection, vec![profile.clone()], None);
    let profiles = router.profiles();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].name, "p-acc");
}

#[test]
fn test_select_profile_from_chunks_empty_chunks() {
    use crate::router::select_profile_from_chunks;
    let profile = SlmProfile::new(
        "p-empty",
        "http://localhost/mcp",
        vec![1],
        TokenBudget::new(100, 10),
        0.1,
    );
    let res = select_profile_from_chunks(&[profile], &[]);
    assert!(
        matches!(res, Err(ContextraError::NotFound(msg)) if msg.contains("Keine gültigen Chunks"))
    );
}

#[test]
fn test_select_profile_max_score_meets_threshold_when_aggregated_does_not() {
    use crate::router::select_profile_from_chunks;
    use contextra_types::{ContextChunk, DocId};

    // Profile requires min_relevance_score = 0.8
    let profile = SlmProfile::new(
        "p-max-score",
        "http://localhost/mcp",
        vec![10],
        TokenBudget::new(1000, 100),
        0.8,
    );

    let chunk_pos = ContextChunk {
        doc_id: DocId::new(2),
        content: "pos".to_string(),
        relevance: 0.8,
        token_count: 5,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };
    // Aggregated score = 0.8 * 1.2 = 0.96 >= 0.8
    let chunks = vec![(chunk_pos, Some(10))];
    let idx = select_profile_from_chunks(&[profile], &chunks).unwrap(); // unwrap
    assert_eq!(idx, 0);
}
