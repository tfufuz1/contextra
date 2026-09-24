use contextra_cognition::aggregation_phase::AggregationConfig;
use contextra_cognition::consolidation_executor::ConsolidationEngine;
use contextra_cognition::leanrag_input::build_leanrag_inputs;
use contextra_cognition::memory_consolidation::{ConsolidationConfig, SynthesisConfig};
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_types::{DocId, EntityId};
use contextra_engine::collection::Collection;
use contextra_graph::csr::EdgeType;
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_graph::CsrGraph;
use contextra_store::LsmStorage;
use contextra_vector::HnswIndex;
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

struct TestMockLlm {
    should_fail: AtomicBool,
    call_count: AtomicU64,
}

impl TestMockLlm {
    fn new() -> Self {
        Self {
            should_fail: AtomicBool::new(false),
            call_count: AtomicU64::new(0),
        }
    }
}

impl LlmTextGenerator for TestMockLlm {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, contextra_types::Result<String>> {
        Box::pin(async move {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if self.should_fail.load(Ordering::SeqCst) {
                Err(contextra_types::ContextraError::Internal(
                    "Simulated LLM Fault Injection Failure".to_string(),
                ))
            } else {
                Ok("Mock abstract community summary".to_string())
            }
        })
    }
}

async fn create_test_collection() -> (Arc<Collection<LsmStorage, HnswIndex>>, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .expect("LsmStorage"),
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .expect("HnswIndex"),
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Arc::new(Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_engine::Language::English,
    ));
    (col, dir)
}

async fn setup_test_graph_data(col: &Collection<LsmStorage, HnswIndex>) {
    col.insert(
        "doc_10",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "Cluster 1 item A"})),
    )
    .await
    .expect("insert doc_10");

    col.insert(
        "doc_20",
        &[0.95, 0.05, 0.0, 0.0],
        Some(json!({"text": "Cluster 1 item B"})),
    )
    .await
    .expect("insert doc_20");

    col.insert(
        "doc_30",
        &[0.0, 0.0, 1.0, 0.0],
        Some(json!({"text": "Cluster 2 item A"})),
    )
    .await
    .expect("insert doc_30");

    col.insert(
        "doc_40",
        &[0.0, 0.0, 0.95, 0.05],
        Some(json!({"text": "Cluster 2 item B"})),
    )
    .await
    .expect("insert doc_40");

    let e10 = EntityId::from_doc_id(DocId::from_key("doc_10").unwrap());
    let e20 = EntityId::from_doc_id(DocId::from_key("doc_20").unwrap());
    let e30 = EntityId::from_doc_id(DocId::from_key("doc_30").unwrap());
    let e40 = EntityId::from_doc_id(DocId::from_key("doc_40").unwrap());

    let graph = col.graph_index();

    let edge1 = HyperEdge::new(
        HyperEdgeId::new(1),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e10),
            RoleBinding::new(RoleId::new(2), e20),
        ],
        1.0,
    );
    let edge2 = HyperEdge::new(
        HyperEdgeId::new(2),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e30),
            RoleBinding::new(RoleId::new(2), e40),
        ],
        1.0,
    );
    let edge3 = HyperEdge::new(
        HyperEdgeId::new(3),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e10),
            RoleBinding::new(RoleId::new(2), e30),
        ],
        1.0,
    );
    let edge4 = HyperEdge::new(
        HyperEdgeId::new(4),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e20),
            RoleBinding::new(RoleId::new(2), e40),
        ],
        1.0,
    );

    graph.insert_hyperedge_direct(edge1);
    graph.insert_hyperedge_direct(edge2);
    graph.insert_hyperedge_direct(edge3);
    graph.insert_hyperedge_direct(edge4);
}

/// Test (a): Default Engine (ohne with_leanrag) ruft Stage 3 nicht auf.
#[tokio::test]
async fn test_default_engine_skips_leanrag_stage() {
    let (col, _dir) = create_test_collection().await;
    setup_test_graph_data(&col).await;

    let llm = Arc::new(TestMockLlm::new());
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let engine = ConsolidationEngine::new(
        col.clone(),
        ConsolidationConfig {
            near_duplicate_cosine_threshold: 0.999,
            ..Default::default()
        },
        SynthesisConfig::default(),
        Duration::from_secs(60),
        cancel_token,
    )
    .with_llm(llm.clone());

    let initial_call_count = llm.call_count.load(Ordering::SeqCst);
    let res = engine.run_cycle().await;
    assert!(res.is_ok());

    let graph = col.graph_index();
    assert_eq!(
        graph.max_hyperedge_id(),
        4,
        "Graph max_hyperedge_id must remain 4 when LeanRAG stage is not opted-in"
    );
    let post_call_count = llm.call_count.load(Ordering::SeqCst);
    assert_eq!(
        initial_call_count, post_call_count,
        "LLM should not be called for LeanRAG stage 3 in default engine"
    );
}

/// Test (b): Engine mit with_leanrag + LLM führt Stage 3 aus und erzeugt Superkante.
#[tokio::test]
async fn test_opt_in_leanrag_stage_creates_superedge() {
    let (col, _dir) = create_test_collection().await;
    setup_test_graph_data(&col).await;

    let llm = Arc::new(TestMockLlm::new());
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let agg_cfg = AggregationConfig {
        max_clusters: 2,
        clustering_tau_threshold: 0.1,
        min_cluster_size: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 5,
        ..Default::default()
    };

    let engine = ConsolidationEngine::new(
        col.clone(),
        ConsolidationConfig {
            near_duplicate_cosine_threshold: 0.999,
            ..Default::default()
        },
        SynthesisConfig::default(),
        Duration::from_secs(60),
        cancel_token,
    )
    .with_llm(llm.clone())
    .with_leanrag(agg_cfg);

    let res = engine.run_cycle().await;
    assert!(res.is_ok(), "run_cycle should succeed with opt-in leanrag");

    let graph = col.graph_index();
    let max_id = graph.max_hyperedge_id();
    assert!(
        max_id > 4,
        "Superedge should be created with hyperedge ID > 4"
    );

    let superedge = graph.get_hyperedge(HyperEdgeId::new(max_id));
    assert!(superedge.is_some(), "Superedge should exist in CSR graph");
    if let Some(edge) = superedge {
        assert!(
            !edge.child_edge_ids.is_empty(),
            "Superedge child_edge_ids should not be empty"
        );
    }
}

/// Test (c): with_leanrag ohne LLM führt keine Stage 3 aus und gibt keinen Fehler zurück.
#[tokio::test]
async fn test_opt_in_leanrag_without_llm_skips_stage() {
    let (col, _dir) = create_test_collection().await;
    setup_test_graph_data(&col).await;

    let cancel_token = tokio_util::sync::CancellationToken::new();

    let agg_cfg = AggregationConfig {
        max_clusters: 2,
        clustering_tau_threshold: 0.1,
        min_cluster_size: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 5,
        ..Default::default()
    };

    let engine = ConsolidationEngine::new(
        col.clone(),
        ConsolidationConfig {
            near_duplicate_cosine_threshold: 0.999,
            ..Default::default()
        },
        SynthesisConfig::default(),
        Duration::from_secs(60),
        cancel_token,
    )
    .with_leanrag(agg_cfg);

    let res = engine.run_cycle().await;
    assert!(res.is_ok());

    let graph = col.graph_index();
    assert_eq!(
        graph.max_hyperedge_id(),
        4,
        "Graph max_hyperedge_id should stay 4 without LLM"
    );
}

/// Test (d): build_leanrag_inputs ist deterministisch und respektiert max_nodes.
#[tokio::test]
async fn test_build_leanrag_inputs_determinism_and_max_nodes() {
    let (col, _dir) = create_test_collection().await;
    setup_test_graph_data(&col).await;

    let d10 = DocId::from_key("doc_10").unwrap();
    let d20 = DocId::from_key("doc_20").unwrap();
    let d30 = DocId::from_key("doc_30").unwrap();
    let d40 = DocId::from_key("doc_40").unwrap();

    let turns = vec![
        (d40, vec![0.0, 0.0, 0.95, 0.05]),
        (d10, vec![1.0, 0.0, 0.0, 0.0]),
        (d30, vec![0.0, 0.0, 1.0, 0.0]),
        (d20, vec![0.95, 0.05, 0.0, 0.0]),
    ];

    let inputs1 = build_leanrag_inputs(col.as_ref(), &turns, 10);
    let inputs2 = build_leanrag_inputs(col.as_ref(), &turns, 10);

    assert_eq!(inputs1, inputs2, "build_leanrag_inputs must be deterministic");
    assert_eq!(inputs1.nodes.len(), 4);
    assert_eq!(inputs1.edges.len(), 4);

    // Test max_nodes bounding
    let inputs_capped = build_leanrag_inputs(col.as_ref(), &turns, 2);
    assert_eq!(inputs_capped.nodes.len(), 2);
}

/// Test (e): Stage-Fehler (LLM schlägt fehl) -> run_cycle liefert weiterhin Ok.
#[tokio::test]
async fn test_stage_error_does_not_fail_run_cycle() {
    let (col, _dir) = create_test_collection().await;
    setup_test_graph_data(&col).await;

    let llm = Arc::new(TestMockLlm::new());
    llm.should_fail.store(true, Ordering::SeqCst);

    let cancel_token = tokio_util::sync::CancellationToken::new();

    let agg_cfg = AggregationConfig {
        max_clusters: 2,
        clustering_tau_threshold: 0.1,
        min_cluster_size: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 5,
        ..Default::default()
    };

    let engine = ConsolidationEngine::new(
        col.clone(),
        ConsolidationConfig {
            near_duplicate_cosine_threshold: 0.999,
            ..Default::default()
        },
        SynthesisConfig::default(),
        Duration::from_secs(60),
        cancel_token,
    )
    .with_llm(llm.clone())
    .with_leanrag(agg_cfg);

    let res = engine.run_cycle().await;
    assert!(
        res.is_ok(),
        "run_cycle must succeed even if Stage 3 LLM fails"
    );
}

/// Test (f): Speicherbudget 0 MB wird von validate() abgelehnt / Stage übersprungen, kein Panic.
#[tokio::test]
async fn test_invalid_config_or_zero_budget_skipped_gracefully() {
    let (col, _dir) = create_test_collection().await;
    setup_test_graph_data(&col).await;

    let llm = Arc::new(TestMockLlm::new());
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let invalid_cfg = AggregationConfig {
        max_compaction_peak_memory_mb: 0,
        ..Default::default()
    };

    let engine = ConsolidationEngine::new(
        col.clone(),
        ConsolidationConfig {
            near_duplicate_cosine_threshold: 0.999,
            ..Default::default()
        },
        SynthesisConfig::default(),
        Duration::from_secs(60),
        cancel_token,
    )
    .with_llm(llm.clone())
    .with_leanrag(invalid_cfg);

    let res = engine.run_cycle().await;
    assert!(
        res.is_ok(),
        "run_cycle must handle zero memory budget config gracefully"
    );

    let graph = col.graph_index();
    assert_eq!(
        graph.max_hyperedge_id(),
        4,
        "No superedge should be created when config validation fails"
    );
}
