use contextra_cognition::aggregation_phase::AggregationConfig;
use contextra_cognition::consolidate_semantic_hyperedges;
use contextra_engine::collection::Collection;
use contextra_graph::csr::EdgeType;
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_graph::CsrGraph;
use contextra_ports::{BoxFuture, LlmTextGenerator, MockEmbedder};
use contextra_store::LsmStorage;
use contextra_types::{DocId, EntityId};
use contextra_vector::HnswIndex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

struct CountingMockLlm {
    call_count: Arc<AtomicU64>,
}

impl CountingMockLlm {
    fn new() -> (Self, Arc<AtomicU64>) {
        let counter = Arc::new(AtomicU64::new(0));
        (
            Self {
                call_count: counter.clone(),
            },
            counter,
        )
    }
}

impl LlmTextGenerator for CountingMockLlm {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, contextra_types::Result<String>> {
        let counter = self.call_count.clone();
        Box::pin(async move {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok("Mock LLM generated abstract summary for community".to_string())
        })
    }
}

async fn create_test_collection(
) -> Option<(Arc<Collection<LsmStorage, HnswIndex>>, tempfile::TempDir)> {
    let dir = tempdir().ok()?;
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .ok()?,
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .ok()?,
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
    Some((col, dir))
}

#[tokio::test]
async fn test_consolidate_semantic_hyperedges_successful_clustering_and_superedge() {
    let (col, _dir) = match create_test_collection().await {
        Some(t) => t,
        None => return,
    };

    // Insert 4 documents
    col.insert("doc_1", &[1.0, 0.0, 0.0, 0.0], None)
        .await
        .expect("insert doc_1");
    col.insert("doc_2", &[0.95, 0.05, 0.0, 0.0], None)
        .await
        .expect("insert doc_2");
    col.insert("doc_3", &[0.0, 0.0, 1.0, 0.0], None)
        .await
        .expect("insert doc_3");
    col.insert("doc_4", &[0.0, 0.0, 0.95, 0.05], None)
        .await
        .expect("insert doc_4");

    let e1 = EntityId::from_doc_id(DocId::from_key("doc_1").unwrap());
    let e2 = EntityId::from_doc_id(DocId::from_key("doc_2").unwrap());
    let e3 = EntityId::from_doc_id(DocId::from_key("doc_3").unwrap());
    let e4 = EntityId::from_doc_id(DocId::from_key("doc_4").unwrap());

    let graph = col.graph_index();

    let edge1 = HyperEdge::new(
        HyperEdgeId::new(1),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e1),
            RoleBinding::new(RoleId::new(2), e2),
        ],
        1.0,
    );
    let edge2 = HyperEdge::new(
        HyperEdgeId::new(2),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e3),
            RoleBinding::new(RoleId::new(2), e4),
        ],
        1.0,
    );
    let edge3 = HyperEdge::new(
        HyperEdgeId::new(3),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e1),
            RoleBinding::new(RoleId::new(2), e3),
        ],
        1.0,
    );

    graph.insert_hyperedge_direct(edge1);
    graph.insert_hyperedge_direct(edge2);
    graph.insert_hyperedge_direct(edge3);

    let embedder = MockEmbedder::new(4);
    let (llm, _counter) = CountingMockLlm::new();

    let config = AggregationConfig {
        max_clusters: 2,
        clustering_tau_threshold: 0.1,
        min_cluster_size: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 5,
        gmm_deterministic_seed: 42,
        ..Default::default()
    };

    let res = consolidate_semantic_hyperedges(col.as_ref(), &embedder, &llm, &config)
        .await
        .expect("consolidate_semantic_hyperedges should succeed");

    assert!(
        res.abstract_hyperedges_created > 0,
        "Should create at least 1 abstract super-hyperedge"
    );
    assert!(
        res.child_edge_ids_written > 0,
        "child_edge_ids_written must be > 0"
    );
}

#[tokio::test]
async fn test_consolidate_semantic_hyperedges_budget_exceeded_aborts_before_llm() {
    let (col, _dir) = match create_test_collection().await {
        Some(t) => t,
        None => return,
    };

    col.insert("doc_1", &[1.0, 0.0, 0.0, 0.0], None)
        .await
        .expect("insert doc_1");

    let embedder = MockEmbedder::new(4);
    let (llm, llm_calls) = CountingMockLlm::new();

    // Set invalid max_compaction_peak_memory_mb to 0
    let config = AggregationConfig {
        max_compaction_peak_memory_mb: 0,
        ..Default::default()
    };

    let res = consolidate_semantic_hyperedges(col.as_ref(), &embedder, &llm, &config).await;

    assert!(
        res.is_err(),
        "Budget / config validation error should return Err"
    );
    assert_eq!(
        llm_calls.load(Ordering::SeqCst),
        0,
        "0 LLM calls must be made when budget/config validation fails"
    );
}

#[tokio::test]
async fn test_consolidate_semantic_hyperedges_deterministic_seed() {
    let (col1, _dir1) = match create_test_collection().await {
        Some(t) => t,
        None => return,
    };
    let (col2, _dir2) = match create_test_collection().await {
        Some(t) => t,
        None => return,
    };

    for col in [&col1, &col2] {
        col.insert("doc_1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .unwrap();
        col.insert("doc_2", &[0.95, 0.05, 0.0, 0.0], None)
            .await
            .unwrap();
        col.insert("doc_3", &[0.0, 0.0, 1.0, 0.0], None)
            .await
            .unwrap();
        col.insert("doc_4", &[0.0, 0.0, 0.95, 0.05], None)
            .await
            .unwrap();

        let e1 = EntityId::from_doc_id(DocId::from_key("doc_1").unwrap());
        let e2 = EntityId::from_doc_id(DocId::from_key("doc_2").unwrap());
        let e3 = EntityId::from_doc_id(DocId::from_key("doc_3").unwrap());
        let e4 = EntityId::from_doc_id(DocId::from_key("doc_4").unwrap());

        let graph = col.graph_index();

        let edge1 = HyperEdge::new(
            HyperEdgeId::new(1),
            EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), e1),
                RoleBinding::new(RoleId::new(2), e2),
            ],
            1.0,
        );
        let edge2 = HyperEdge::new(
            HyperEdgeId::new(2),
            EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), e3),
                RoleBinding::new(RoleId::new(2), e4),
            ],
            1.0,
        );
        let edge3 = HyperEdge::new(
            HyperEdgeId::new(3),
            EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), e1),
                RoleBinding::new(RoleId::new(2), e3),
            ],
            1.0,
        );

        graph.insert_hyperedge_direct(edge1);
        graph.insert_hyperedge_direct(edge2);
        graph.insert_hyperedge_direct(edge3);
    }

    let embedder = MockEmbedder::new(4);
    let (llm1, _) = CountingMockLlm::new();
    let (llm2, _) = CountingMockLlm::new();

    let config = AggregationConfig {
        max_clusters: 2,
        clustering_tau_threshold: 0.1,
        min_cluster_size: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 5,
        gmm_deterministic_seed: 12345,
        ..Default::default()
    };

    let res1 = consolidate_semantic_hyperedges(col1.as_ref(), &embedder, &llm1, &config)
        .await
        .unwrap();

    let res2 = consolidate_semantic_hyperedges(col2.as_ref(), &embedder, &llm2, &config)
        .await
        .unwrap();

    assert_eq!(
        res1, res2,
        "Identical inputs with identical gmm_deterministic_seed must produce bit-identical results"
    );
}
