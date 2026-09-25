use contextra_cognition::aggregation_phase::{AggregationConfig, AggregationEdge, AggregationNode};
use contextra_cognition::consolidation_executor::execute_leanrag_aggregation_stage;
use contextra_cognition::memory_consolidation::CommunityStabilityTracker;
use contextra_engine::collection::Collection;
use contextra_graph::csr::EdgeType;
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_graph::CsrGraph;
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_store::LsmStorage;
use contextra_types::{EntityId, TxId};
use contextra_vector::HnswIndex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

struct TestMockLlm;

impl LlmTextGenerator for TestMockLlm {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, contextra_types::Result<String>> {
        Box::pin(async move { Ok("Mock abstract community summary".to_string()) })
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
async fn test_execute_leanrag_aggregation_stage_creates_superedge_in_csr_graph() {
    let (collection, _dir) = match create_test_collection().await {
        Some(tuple) => tuple,
        None => return,
    };
    let graph = collection.graph_index();

    // 1. Seed two raw hyperedges into CsrGraph
    let edge1 = HyperEdge::new(
        HyperEdgeId::new(1),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(10)),
            RoleBinding::new(RoleId::new(2), EntityId::new(20)),
        ],
        1.0,
    );
    let edge2 = HyperEdge::new(
        HyperEdgeId::new(2),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(30)),
            RoleBinding::new(RoleId::new(2), EntityId::new(40)),
        ],
        1.0,
    );
    let edge3 = HyperEdge::new(
        HyperEdgeId::new(3),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(10)),
            RoleBinding::new(RoleId::new(2), EntityId::new(30)),
        ],
        1.0,
    );

    graph.insert_hyperedge_direct(edge1);
    graph.insert_hyperedge_direct(edge2);
    graph.insert_hyperedge_direct(edge3);

    // 2. Prepare Aggregation input nodes & edges
    let nodes = vec![
        AggregationNode {
            entity: EntityId::new(10),
            embedding: vec![1.0, 0.0, 0.0, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(20),
            embedding: vec![0.95, 0.05, 0.0, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(30),
            embedding: vec![0.0, 0.0, 1.0, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(40),
            embedding: vec![0.0, 0.0, 0.95, 0.05],
            type_id: 1,
        },
    ];

    let edges = vec![
        AggregationEdge {
            id: HyperEdgeId::new(1),
            predicate_type: 100,
            participants: vec![EntityId::new(10), EntityId::new(20)],
        },
        AggregationEdge {
            id: HyperEdgeId::new(2),
            predicate_type: 100,
            participants: vec![EntityId::new(30), EntityId::new(40)],
        },
        AggregationEdge {
            id: HyperEdgeId::new(3),
            predicate_type: 100,
            participants: vec![EntityId::new(10), EntityId::new(30)],
        },
    ];

    let config = AggregationConfig {
        max_clusters: 2,
        clustering_tau_threshold: 0.1,
        min_cluster_size: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 5,
        ..Default::default()
    };

    let llm = TestMockLlm;
    let mut tracker = CommunityStabilityTracker::new();
    let wal_tx = TxId::new(500);

    // 3. Execute stage 3 aggregation
    let stage_res = execute_leanrag_aggregation_stage(
        collection.as_ref(),
        &nodes,
        &edges,
        &config,
        &llm,
        &mut tracker,
        wal_tx,
    )
    .await;

    assert!(
        stage_res.is_ok(),
        "execute_leanrag_aggregation_stage should succeed"
    );
    if let Ok((result, alpha_nodes)) = stage_res {
        assert!(
            result.abstract_hyperedges_created > 0,
            "Superedges should be created"
        );
        assert!(!alpha_nodes.is_empty(), "Alpha nodes should be synthesized");
    }

    // 4. Verify graph state: new superedge exists in CsrGraph
    let max_id = graph.max_hyperedge_id();
    assert!(max_id > 3, "New superedge should have ID > 3");

    let superedge = graph.get_hyperedge(HyperEdgeId::new(max_id));
    assert!(
        superedge.is_some(),
        "Superedge must be retrievable from graph"
    );
    if let Some(edge) = superedge {
        assert!(!edge.child_edge_ids.is_empty());
    }
}
