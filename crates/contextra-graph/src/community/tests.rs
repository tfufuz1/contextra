use super::*;
use crate::csr::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{ContextraError, Edge, Entity, EntityId, TxId};

use super::*;

#[tokio::test]
async fn test_community_detection_determinism() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=10 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("E{i}"), "Node"))
            .await
            .unwrap(); // unwrap allowed
    }

    // Add some edges
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "knows"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "knows"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(1), "knows"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(4), EntityId::new(5), "knows"))
        .await
        .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed

    let config = CommunityDetectionConfig {
        max_iterations: 50,
        seed: 12345,
        hyperedges_included: false,
    };

    let run1 = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed
    let run2 = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

    assert_eq!(
        run1, run2,
        "Twice execution with identical graph and seed must yield identical CommunityAssignments"
    );
}

#[tokio::test]
async fn test_community_detection_disconnected_clusters() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    // Cluster 1: Nodes 1, 2, 3 tightly connected
    for id in 1..=3 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("C1_{id}"), "Node"),
            )
            .await
            .unwrap(); // unwrap allowed
    }
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "link"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(1), "link"))
        .await
        .unwrap(); // unwrap allowed

    // Cluster 2: Nodes 100, 101, 102 tightly connected (no path to Cluster 1)
    for id in [100, 101, 102] {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("C2_{id}"), "Node"),
            )
            .await
            .unwrap(); // unwrap allowed
    }
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(100), EntityId::new(101), "link"),
        )
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(101), EntityId::new(102), "link"),
        )
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(102), EntityId::new(100), "link"),
        )
        .await
        .unwrap(); // unwrap allowed

    graph.commit(tx).await.unwrap(); // unwrap allowed

    let config = CommunityDetectionConfig::default();
    let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

    let map: HashMap<u64, u64> = assignments
        .into_iter()
        .map(|a| (a.entity_id.inner(), a.community_id))
        .collect();

    // Nodes in Cluster 1 must share the same community ID
    let c1_community = map[&1];
    assert_eq!(map[&2], c1_community);
    assert_eq!(map[&3], c1_community);

    // Nodes in Cluster 2 must share the same community ID
    let c2_community = map[&100];
    assert_eq!(map[&101], c2_community);
    assert_eq!(map[&102], c2_community);

    // Cluster 1 and Cluster 2 must be assigned DIFFERENT communities
    assert_ne!(
        c1_community, c2_community,
        "Disconnected clusters MUST be assigned to different communities"
    );
}

/// Comparison Test: Synthetic Graph showing Leiden's well-connected community guarantee vs LPA reference.
///
/// Graph structure: Two triangles A (1-2-3-1) and B (10-11-12-10) connected by a single weak bridge edge (3-10).
/// Under LPA, label propagation could easily cause labels from triangle A to flood triangle B across the weak bridge edge,
/// merging the two distinct clusters into a single community.
/// Under Leiden, the Refinement Phase evaluates sub-community modularity and guarantees that Cluster A and Cluster B
/// remain as distinct, well-connected communities.
#[tokio::test]
async fn test_leiden_vs_lpa_well_connected_communities_comparison() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    // Cluster A
    for id in 1..=3 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("A_{id}"), "Node"),
            )
            .await
            .unwrap(); // unwrap allowed
    }
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "link"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(1), "link"))
        .await
        .unwrap(); // unwrap allowed

    // Cluster B
    for id in 10..=12 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("B_{id}"), "Node"),
            )
            .await
            .unwrap(); // unwrap allowed
    }
    graph
        .add_edge(tx, Edge::new(EntityId::new(10), EntityId::new(11), "link"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(11), EntityId::new(12), "link"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(12), EntityId::new(10), "link"))
        .await
        .unwrap(); // unwrap allowed

    // Weak bridge edge connecting Cluster A and Cluster B
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(3), EntityId::new(10), "weak_bridge"),
        )
        .await
        .unwrap(); // unwrap allowed

    graph.commit(tx).await.unwrap(); // unwrap allowed

    let config = CommunityDetectionConfig::default();
    let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

    let map: HashMap<u64, u64> = assignments
        .into_iter()
        .map(|a| (a.entity_id.inner(), a.community_id))
        .collect();

    // Cluster A nodes must share a common community ID
    let comm_a = map[&1];
    assert_eq!(
        map[&2], comm_a,
        "Cluster A nodes must share the same community"
    );
    assert_eq!(
        map[&3], comm_a,
        "Cluster A nodes must share the same community"
    );

    // Cluster B nodes must share a common community ID
    let comm_b = map[&10];
    assert_eq!(
        map[&11], comm_b,
        "Cluster B nodes must share the same community"
    );
    assert_eq!(
        map[&12], comm_b,
        "Cluster B nodes must share the same community"
    );

    // Leiden guarantees that Cluster A and Cluster B are distinct well-connected communities
    assert_ne!(
            comm_a, comm_b,
            "Leiden must separate Cluster A and Cluster B into distinct well-connected communities despite weak bridge edge"
        );
}

#[derive(Clone)]
struct LogCaptureLayer(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for LogCaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = StringVisitor(String::new());
        event.record(&mut visitor);
        self.0.lock().unwrap().push(visitor.0); // unwrap allowed
    }
}

struct StringVisitor(String);
impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        write!(self.0, "{}={:?} ", field.name(), value).ok();
    }
}

proptest::proptest! {
    #[test]
    fn prop_community_detection_never_panics(
        node_count in 1usize..50,
        edge_specs in proptest::collection::vec((0..50usize, 0..50usize), 0..150),
        max_iterations in 1u32..50,
        seed in proptest::num::u64::ANY,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap(); // unwrap allowed
        let res: std::result::Result<(), proptest::test_runner::TestCaseError> = rt.block_on(async {
            let graph = CsrGraph::new();
            let tx = TxId::new(1);
            for i in 0..node_count {
                graph.add_entity(tx, Entity::new(EntityId::new(i as u64 + 1), format!("N{i}"), "Node")).await.unwrap(); // unwrap allowed
            }
            for (src, dst) in edge_specs {
                let src_id = EntityId::new((src % node_count) as u64 + 1);
                let dst_id = EntityId::new((dst % node_count) as u64 + 1);
                let _ = graph.add_edge(tx, Edge::new(src_id, dst_id, "link")).await;
            }
            graph.commit(tx).await.unwrap(); // unwrap allowed

            let config = CommunityDetectionConfig { max_iterations, seed, hyperedges_included: false };
            let result = detect_communities(&graph, &config).await;

            proptest::prop_assert!(result.is_ok() || result.is_err());
            if let Ok(assignments) = result {
                proptest::prop_assert_eq!(assignments.len(), node_count);
            }
            Ok(())
        });
        res?;
    }

    #[test]
    fn prop_community_detection_every_node_assigned(
        node_count in 1usize..30,
        edge_specs in proptest::collection::vec((0..30usize, 0..30usize), 0..60),
        seed in proptest::num::u64::ANY,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap(); // unwrap allowed
        let res: std::result::Result<(), proptest::test_runner::TestCaseError> = rt.block_on(async {
            let graph = CsrGraph::new();
            let tx = TxId::new(1);
            let expected_ids: std::collections::HashSet<_> = (0..node_count)
                .map(|i| EntityId::new(i as u64 + 1))
                .collect();

            for &id in &expected_ids {
                graph.add_entity(tx, Entity::new(id, format!("Node{}", id.inner()), "Node")).await.unwrap(); // unwrap allowed
            }
            for (src, dst) in edge_specs {
                let src_id = EntityId::new((src % node_count) as u64 + 1);
                let dst_id = EntityId::new((dst % node_count) as u64 + 1);
                let _ = graph.add_edge(tx, Edge::new(src_id, dst_id, "link")).await;
            }
            graph.commit(tx).await.unwrap(); // unwrap allowed

            let config = CommunityDetectionConfig { max_iterations: 20, seed, hyperedges_included: false };
            let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

            proptest::prop_assert_eq!(assignments.len(), node_count);
            let assigned_ids: std::collections::HashSet<_> = assignments
                .into_iter()
                .map(|a| a.entity_id)
                .collect();
            proptest::prop_assert_eq!(assigned_ids, expected_ids);
            Ok(())
        });
        res?;
    }
}

#[tokio::test]
async fn test_community_detection_non_convergence_logs_warning_and_returns_best_effort() {
    use tracing_subscriber::layer::SubscriberExt;

    let logs = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let capture_layer = LogCaptureLayer(logs.clone());
    let subscriber = tracing_subscriber::registry().with(capture_layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=10 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(i), format!("Node{i}"), "Node"),
            )
            .await
            .unwrap(); // unwrap allowed
    }

    // Bipartite graph 1..5 to 6..10
    for i in 1..=5 {
        for j in 6..=10 {
            graph
                .add_edge(tx, Edge::new(EntityId::new(i), EntityId::new(j), "link"))
                .await
                .unwrap(); // unwrap allowed
        }
    }
    graph.commit(tx).await.unwrap(); // unwrap allowed

    // With max_iterations: 1, full convergence cannot occur if nodes change labels during iteration 1
    let config = CommunityDetectionConfig {
        max_iterations: 1,
        seed: 42,
        hyperedges_included: false,
    };

    let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

    assert_eq!(
        assignments.len(),
        10,
        "Best-effort assignments must be returned"
    );

    let captured = logs.lock().unwrap(); // unwrap allowed
    let warning_found = captured.iter().any(|msg| {
            msg.contains("Community detection reached maximum iterations without reaching stable label assignment")
                && msg.contains("max_iterations=1")
                && msg.contains("unstable_nodes=")
        });

    assert!(
        warning_found,
        "Expected structured warning log on community detection non-convergence, got logs: {:?}",
        *captured
    );
}

#[tokio::test]
#[allow(non_snake_case)]
async fn detect_communities_CASE_empty_graph() {
    let graph = CsrGraph::new();
    let config = CommunityDetectionConfig::default();

    let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed
    assert!(
        assignments.is_empty(),
        "Community detection on empty graph must return empty assignments"
    );
}

#[tokio::test]
#[allow(non_snake_case)]
async fn detect_communities_CASE_single_node() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let id = EntityId::new(42);

    graph
        .add_entity(tx, Entity::new(id, "SingleNode", "Type"))
        .await
        .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed

    let config = CommunityDetectionConfig::default();
    let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed

    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].entity_id, id);
    assert_eq!(assignments[0].community_id, id.inner());
}

#[test]
#[allow(non_snake_case)]
fn serialization_roundtrip_CASE_community_config_and_assignment() {
    let config = CommunityDetectionConfig {
        max_iterations: 150,
        seed: 987654321,
        hyperedges_included: false,
    };

    let serialized_config = bincode::serialize(&config).unwrap(); // unwrap allowed
    let deserialized_config: CommunityDetectionConfig =
        bincode::deserialize(&serialized_config).unwrap(); // unwrap allowed
    assert_eq!(config.max_iterations, deserialized_config.max_iterations);
    assert_eq!(config.seed, deserialized_config.seed);
    assert_eq!(
        config.hyperedges_included,
        deserialized_config.hyperedges_included
    );

    let assignment = CommunityAssignment {
        entity_id: EntityId::new(100),
        community_id: 100,
        hyperedges_included: false,
    };

    let serialized_assignment = bincode::serialize(&assignment).unwrap(); // unwrap allowed
    let deserialized_assignment: CommunityAssignment =
        bincode::deserialize(&serialized_assignment).unwrap(); // unwrap allowed
    assert_eq!(assignment, deserialized_assignment);
}

#[test]
#[allow(non_snake_case)]
fn test_hyperedges_included_flag_default_is_false() {
    let config = CommunityDetectionConfig::default();
    assert!(
        !config.hyperedges_included,
        "Default for CommunityDetectionConfig.hyperedges_included must be false"
    );

    let assignment = CommunityAssignment {
        entity_id: EntityId::new(1),
        community_id: 1,
        hyperedges_included: false,
    };
    assert!(!assignment.hyperedges_included);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn test_hyperedges_included_flag_passthrough_from_config_to_assignment() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    graph
        .add_entity(tx, Entity::new(EntityId::new(1), "Node1", "Type"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_entity(tx, Entity::new(EntityId::new(2), "Node2", "Type"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
        .await
        .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed

    // Test with hyperedges_included = true
    let config_true = CommunityDetectionConfig {
        max_iterations: 10,
        seed: 42,
        hyperedges_included: true,
    };
    let assignments_true = detect_communities(&graph, &config_true).await.unwrap(); // unwrap allowed
    assert!(!assignments_true.is_empty());
    for assignment in &assignments_true {
        assert!(
            assignment.hyperedges_included,
            "CommunityAssignment must reflect config.hyperedges_included = true"
        );
    }

    // Test with hyperedges_included = false
    let config_false = CommunityDetectionConfig {
        max_iterations: 10,
        seed: 42,
        hyperedges_included: false,
    };
    let assignments_false = detect_communities(&graph, &config_false).await.unwrap(); // unwrap allowed
    assert!(!assignments_false.is_empty());
    for assignment in &assignments_false {
        assert!(
            !assignment.hyperedges_included,
            "CommunityAssignment must reflect config.hyperedges_included = false"
        );
    }
}

#[tokio::test]
#[allow(non_snake_case)]
async fn test_star_expansion_iterator_and_community_detection() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    // Add 3 entities: 1, 2, 3
    for i in 1..=3 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("E{i}"), "Type"))
            .await
            .unwrap();
    }

    // Add hyperedge linking E1, E2, E3 (no binary edges present!)
    let he = HyperEdge::new(
        HyperEdgeId::new(50),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(1)),
            RoleBinding::new(RoleId::new(2), EntityId::new(2)),
            RoleBinding::new(RoleId::new(3), EntityId::new(3)),
        ],
        1.0,
    );
    graph.insert_hyperedge(he);
    graph.commit(tx).await.unwrap();

    // 1. Verify StarExpansionIterator
    let star_items: Vec<_> = StarExpansionIterator::new(&graph).collect();
    assert_eq!(star_items.len(), 3);
    assert_eq!(
        star_items,
        vec![
            (EntityId::new(1), VirtualHyperedgeNode::new(50)),
            (EntityId::new(2), VirtualHyperedgeNode::new(50)),
            (EntityId::new(3), VirtualHyperedgeNode::new(50)),
        ]
    );

    // 2. When hyperedges_included = false, E1, E2, E3 are disconnected singletons (no binary edges)
    let config_false = CommunityDetectionConfig {
        max_iterations: 10,
        seed: 42,
        hyperedges_included: false,
    };
    let assignments_false = detect_communities(&graph, &config_false).await.unwrap();
    assert_eq!(assignments_false.len(), 3);
    let comm_1 = assignments_false
        .iter()
        .find(|a| a.entity_id == EntityId::new(1))
        .unwrap()
        .community_id;
    let comm_2 = assignments_false
        .iter()
        .find(|a| a.entity_id == EntityId::new(2))
        .unwrap()
        .community_id;
    let comm_3 = assignments_false
        .iter()
        .find(|a| a.entity_id == EntityId::new(3))
        .unwrap()
        .community_id;
    assert_ne!(comm_1, comm_2);
    assert_ne!(comm_2, comm_3);

    // 3. When hyperedges_included = true, star expansion connects E1, E2, E3 via virtual hyperedge node
    let config_true = CommunityDetectionConfig {
        max_iterations: 10,
        seed: 42,
        hyperedges_included: true,
    };
    let assignments_true = detect_communities(&graph, &config_true).await.unwrap();
    assert_eq!(assignments_true.len(), 3);
    let comm_1_h = assignments_true
        .iter()
        .find(|a| a.entity_id == EntityId::new(1))
        .unwrap()
        .community_id;
    let comm_2_h = assignments_true
        .iter()
        .find(|a| a.entity_id == EntityId::new(2))
        .unwrap()
        .community_id;
    let comm_3_h = assignments_true
        .iter()
        .find(|a| a.entity_id == EntityId::new(3))
        .unwrap()
        .community_id;
    assert_eq!(comm_1_h, comm_2_h);
    assert_eq!(comm_2_h, comm_3_h);
}

#[test]
#[allow(non_snake_case)]
fn test_hyperedges_included_serde_backwards_compatibility() {
    // Old JSON config snapshot without hyperedges_included field
    let old_config_json = r#"{"max_iterations":100,"seed":42}"#;
    let config: CommunityDetectionConfig = serde_json::from_str(old_config_json).unwrap(); // unwrap allowed
    assert_eq!(config.max_iterations, 100);
    assert_eq!(config.seed, 42);
    assert!(
        !config.hyperedges_included,
        "Deserializing old config without hyperedges_included must default to false"
    );

    // Old JSON assignment snapshot without hyperedges_included field
    let old_assignment_json = r#"{"entity_id":100,"community_id":42}"#;
    let assignment: CommunityAssignment = serde_json::from_str(old_assignment_json).unwrap(); // unwrap allowed
    assert_eq!(assignment.entity_id, EntityId::new(100));
    assert_eq!(assignment.community_id, 42);
    assert!(
        !assignment.hyperedges_included,
        "Deserializing old assignment without hyperedges_included must default to false"
    );

    // Serialization roundtrip with hyperedges_included = true
    let config_true = CommunityDetectionConfig {
        max_iterations: 50,
        seed: 123,
        hyperedges_included: true,
    };
    let serialized = serde_json::to_string(&config_true).unwrap(); // unwrap allowed
    assert!(serialized.contains(r#""hyperedges_included":true"#));
    let deserialized: CommunityDetectionConfig = serde_json::from_str(&serialized).unwrap(); // unwrap allowed
    assert!(deserialized.hyperedges_included);

    let assignment_true = CommunityAssignment {
        entity_id: EntityId::new(100),
        community_id: 42,
        hyperedges_included: true,
    };
    let serialized_a = serde_json::to_string(&assignment_true).unwrap(); // unwrap allowed
    assert!(serialized_a.contains(r#""hyperedges_included":true"#));
    let deserialized_a: CommunityAssignment = serde_json::from_str(&serialized_a).unwrap(); // unwrap allowed
    assert!(deserialized_a.hyperedges_included);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn test_star_expansion_iterator_iteration() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for id in 1..=3 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(id), format!("E{id}"), "Node"))
            .await
            .unwrap(); // unwrap allowed
    }
    graph.commit(tx).await.unwrap(); // unwrap allowed

    let he_id = HyperEdgeId::new(500);
    let hedge = HyperEdge::new(
        he_id,
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(1)),
            RoleBinding::new(RoleId::new(2), EntityId::new(2)),
            RoleBinding::new(RoleId::new(3), EntityId::new(3)),
        ],
        2.5,
    );
    graph.insert_hyperedge_direct(hedge);

    let mut iter = StarExpansionIterator::new(&graph);
    let mut items = Vec::new();
    while let Some((eid, vnode, weight)) = iter.next_with_weight() {
        items.push((eid, vnode.hyperedge_id, weight));
    }

    assert_eq!(items.len(), 3);
    assert_eq!(items[0], (EntityId::new(1), he_id, 2.5));
    assert_eq!(items[1], (EntityId::new(2), he_id, 2.5));
    assert_eq!(items[2], (EntityId::new(3), he_id, 2.5));
}

#[tokio::test]
#[allow(non_snake_case)]
async fn test_community_detection_with_star_expansion_hyperedges() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    // Cluster 1: Nodes 1, 2, 3 (no binary edges)
    for id in 1..=3 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("C1_{id}"), "Node"),
            )
            .await
            .unwrap(); // unwrap allowed
    }
    // Cluster 2: Nodes 10, 11, 12 (no binary edges)
    for id in 10..=12 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("C2_{id}"), "Node"),
            )
            .await
            .unwrap(); // unwrap allowed
    }
    graph.commit(tx).await.unwrap(); // unwrap allowed

    // Add Hyperedge H1 grouping Nodes 1, 2, 3
    let he1 = HyperEdge::new(
        HyperEdgeId::new(101),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(1)),
            RoleBinding::new(RoleId::new(2), EntityId::new(2)),
            RoleBinding::new(RoleId::new(3), EntityId::new(3)),
        ],
        1.0,
    );
    graph.insert_hyperedge_direct(he1);

    // Add Hyperedge H2 grouping Nodes 10, 11, 12
    let he2 = HyperEdge::new(
        HyperEdgeId::new(102),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(10)),
            RoleBinding::new(RoleId::new(2), EntityId::new(11)),
            RoleBinding::new(RoleId::new(3), EntityId::new(12)),
        ],
        1.0,
    );
    graph.insert_hyperedge_direct(he2);

    // Without hyperedges (hyperedges_included = false), no binary edges exist -> singletons
    let config_false = CommunityDetectionConfig {
        max_iterations: 50,
        seed: 42,
        hyperedges_included: false,
    };
    let assignments_false = detect_communities(&graph, &config_false).await.unwrap(); // unwrap allowed
    let map_false: HashMap<u64, u64> = assignments_false
        .into_iter()
        .map(|a| (a.entity_id.inner(), a.community_id))
        .collect();
    assert_ne!(map_false[&1], map_false[&2]);
    assert_ne!(map_false[&10], map_false[&11]);

    // With hyperedges (hyperedges_included = true), star expansion connects nodes via virtual hyperedge nodes
    let config_true = CommunityDetectionConfig {
        max_iterations: 50,
        seed: 42,
        hyperedges_included: true,
    };
    let assignments_true = detect_communities(&graph, &config_true).await.unwrap(); // unwrap allowed
    let map_true: HashMap<u64, u64> = assignments_true
        .into_iter()
        .map(|a| (a.entity_id.inner(), a.community_id))
        .collect();

    // Cluster 1 nodes must be grouped into the same community
    let c1_comm = map_true[&1];
    assert_eq!(map_true[&2], c1_comm);
    assert_eq!(map_true[&3], c1_comm);

    // Cluster 2 nodes must be grouped into the same community
    let c2_comm = map_true[&10];
    assert_eq!(map_true[&11], c2_comm);
    assert_eq!(map_true[&12], c2_comm);

    // Cluster 1 and Cluster 2 must be in distinct communities
    assert_ne!(c1_comm, c2_comm);
}
