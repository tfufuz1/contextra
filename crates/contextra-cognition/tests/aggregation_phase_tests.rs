//! # Vertragstests: Aggregationsphase / LeanRAG Stage 3 (`tests/aggregation_phase_tests.rs`)
//!
//! Validiert Leereingaben, Einzelelemente, Grenz- und Schwellenwerte,
//! bit-exakten Seed-Determinismus und Permutationsinvarianz sowie
//! Property-ähnliche Invarianzprüfungen mit `contextra_ports::SeededRng`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::get_first,
    clippy::panic,
    clippy::disallowed_methods,
    clippy::cast_precision_loss
)]

use contextra_cognition::aggregation_phase::{
    check_compaction_budget, compute_entity_community_hash, run_aggregation_pass,
    AggregationConfig, AggregationEdge, AggregationNode, SuperEdgeDraft, SuperEdgeSink,
};
use contextra_cognition::memory_consolidation::CommunityStabilityTracker;
use contextra_graph::HyperEdgeId;
use contextra_ports::{BoxFuture, LlmTextGenerator, Rng, SeededRng};
use contextra_types::{ContextraError, EntityId, Result};

struct TestMockLlm;

impl LlmTextGenerator for TestMockLlm {
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move { Ok(format!("Abstract concept summary ({})", prompt.len())) })
    }
}

struct TestMockSink {
    tombstones: Vec<HyperEdgeId>,
    super_edges: Vec<SuperEdgeDraft>,
    committed: bool,
}

impl TestMockSink {
    fn new() -> Self {
        Self {
            tombstones: Vec::new(),
            super_edges: Vec::new(),
            committed: false,
        }
    }
}

impl SuperEdgeSink for TestMockSink {
    fn tombstone_edge(&mut self, id: HyperEdgeId) -> Result<()> {
        self.tombstones.push(id);
        Ok(())
    }

    fn write_super_edge(&mut self, draft: SuperEdgeDraft) -> Result<HyperEdgeId> {
        let id = HyperEdgeId(self.super_edges.len() as u64 + 1);
        self.super_edges.push(draft);
        Ok(id)
    }

    fn commit(&mut self) -> Result<()> {
        self.committed = true;
        Ok(())
    }
}

#[tokio::test]
async fn test_empty_input_behavior() {
    let cfg = AggregationConfig::default();
    let llm = TestMockLlm;
    let mut tracker = CommunityStabilityTracker::new();
    let mut sink = TestMockSink::new();

    let (res, alphas) = run_aggregation_pass(&[], &[], &cfg, &llm, &mut tracker, &mut sink)
        .await
        .expect("empty run_aggregation_pass must succeed");

    assert_eq!(res.raw_edges_tombstoned, 0);
    assert_eq!(res.abstract_hyperedges_created, 0);
    assert_eq!(res.peak_memory_used_mb, 0);
    assert_eq!(res.child_edge_ids_written, 0);
    assert!(alphas.is_empty());
    assert!(sink.committed, "Sink must commit even on empty input");
}

#[tokio::test]
async fn test_single_node_element_behavior() {
    let node = AggregationNode {
        entity: EntityId::new(42),
        embedding: vec![1.0, 2.0, 3.0],
        type_id: 1,
    };
    let cfg = AggregationConfig {
        min_cluster_size: 1,
        stability_cycles_required: 1,
        ..Default::default()
    };
    let llm = TestMockLlm;
    let mut tracker = CommunityStabilityTracker::new();
    let mut sink = TestMockSink::new();

    let (res, alphas) = run_aggregation_pass(&[node], &[], &cfg, &llm, &mut tracker, &mut sink)
        .await
        .expect("single node run_aggregation_pass must succeed");

    assert_eq!(res.raw_edges_tombstoned, 0);
    assert_eq!(res.abstract_hyperedges_created, 0);
    assert_eq!(alphas.len(), 1);
    assert_eq!(alphas[0].members, vec![EntityId::new(42)]);
    assert!(sink.committed);
}

#[test]
fn test_configuration_exact_boundary_thresholds() {
    // Valid boundary config
    let mut cfg = AggregationConfig {
        max_compaction_peak_memory_mb: 1,
        clustering_tau_threshold: 0.0,
        min_type_compat_score: 0.0,
        max_clusters: 1,
        min_cluster_size: 1,
        em_max_iterations: 1,
        em_tolerance: 1e-6,
        ..Default::default()
    };
    assert!(cfg.validate().is_ok());

    cfg.clustering_tau_threshold = 1.0;
    cfg.min_type_compat_score = 1.0;
    assert!(cfg.validate().is_ok());

    // Exact violation boundaries
    let mut invalid = cfg.clone();
    invalid.max_compaction_peak_memory_mb = 0;
    assert!(matches!(
        invalid.validate(),
        Err(ContextraError::InvalidInput(_))
    ));

    let mut invalid = cfg.clone();
    invalid.clustering_tau_threshold = -0.0001;
    assert!(matches!(
        invalid.validate(),
        Err(ContextraError::InvalidInput(_))
    ));

    let mut invalid = cfg.clone();
    invalid.clustering_tau_threshold = 1.0001;
    assert!(matches!(
        invalid.validate(),
        Err(ContextraError::InvalidInput(_))
    ));

    let mut invalid = cfg.clone();
    invalid.min_type_compat_score = -0.0001;
    assert!(matches!(
        invalid.validate(),
        Err(ContextraError::InvalidInput(_))
    ));

    let mut invalid = cfg.clone();
    invalid.min_type_compat_score = 1.0001;
    assert!(matches!(
        invalid.validate(),
        Err(ContextraError::InvalidInput(_))
    ));

    let mut invalid = cfg.clone();
    invalid.em_tolerance = 0.0;
    assert!(matches!(
        invalid.validate(),
        Err(ContextraError::InvalidInput(_))
    ));
}

#[test]
fn test_compaction_memory_budget_exact_boundary() {
    let cfg = AggregationConfig {
        max_compaction_peak_memory_mb: 10,
        ..Default::default()
    };

    // Exactly 10 MB in bytes (10 * 1024 * 1024)
    let exact_bytes = 10 * 1024 * 1024;
    assert!(check_compaction_budget(exact_bytes, &cfg).is_ok());

    // 10 MB + 1 byte -> div_ceil yields 11 MB -> MemoryBudgetExceeded
    let exceeded_bytes = exact_bytes + 1;
    let res = check_compaction_budget(exceeded_bytes, &cfg);
    assert!(matches!(
        res,
        Err(ContextraError::MemoryBudgetExceeded {
            used_mb: 11,
            limit_mb: 10
        })
    ));
}

#[tokio::test]
async fn test_identical_input_and_seed_yields_identical_output() {
    let nodes = vec![
        AggregationNode {
            entity: EntityId::new(10),
            embedding: vec![1.0, 0.5, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(20),
            embedding: vec![0.9, 0.6, 0.1],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(30),
            embedding: vec![-1.0, -0.5, 0.0],
            type_id: 2,
        },
        AggregationNode {
            entity: EntityId::new(40),
            embedding: vec![-0.9, -0.6, -0.1],
            type_id: 2,
        },
    ];

    let edges = vec![
        AggregationEdge {
            id: HyperEdgeId::new(1),
            predicate_type: 5,
            participants: vec![EntityId::new(10), EntityId::new(20)],
        },
        AggregationEdge {
            id: HyperEdgeId::new(2),
            predicate_type: 5,
            participants: vec![EntityId::new(30), EntityId::new(40)],
        },
        AggregationEdge {
            id: HyperEdgeId::new(3),
            predicate_type: 5,
            participants: vec![EntityId::new(10), EntityId::new(30)],
        },
    ];

    let cfg = AggregationConfig {
        gmm_deterministic_seed: 98765,
        min_cluster_size: 2,
        max_clusters: 2,
        clustering_tau_threshold: 0.1,
        stability_cycles_required: 1,
        ..Default::default()
    };

    let llm = TestMockLlm;

    let mut tracker1 = CommunityStabilityTracker::new();
    let mut sink1 = TestMockSink::new();
    let (res1, alphas1) =
        run_aggregation_pass(&nodes, &edges, &cfg, &llm, &mut tracker1, &mut sink1)
            .await
            .unwrap();

    let mut tracker2 = CommunityStabilityTracker::new();
    let mut sink2 = TestMockSink::new();
    let (res2, alphas2) =
        run_aggregation_pass(&nodes, &edges, &cfg, &llm, &mut tracker2, &mut sink2)
            .await
            .unwrap();

    assert_eq!(res1, res2);
    assert_eq!(alphas1, alphas2);
    assert_eq!(sink1.tombstones, sink2.tombstones);
    assert_eq!(sink1.super_edges, sink2.super_edges);
}

#[tokio::test]
async fn test_permutation_invariance_of_nodes_and_edges() {
    let node1 = AggregationNode {
        entity: EntityId::new(1),
        embedding: vec![10.0, 10.0],
        type_id: 1,
    };
    let node2 = AggregationNode {
        entity: EntityId::new(2),
        embedding: vec![10.2, 9.8],
        type_id: 1,
    };
    let node3 = AggregationNode {
        entity: EntityId::new(3),
        embedding: vec![-10.0, -10.0],
        type_id: 2,
    };
    let node4 = AggregationNode {
        entity: EntityId::new(4),
        embedding: vec![-9.8, -10.2],
        type_id: 2,
    };

    let edge1 = AggregationEdge {
        id: HyperEdgeId::new(101),
        predicate_type: 7,
        participants: vec![EntityId::new(1), EntityId::new(2)],
    };
    let edge2 = AggregationEdge {
        id: HyperEdgeId::new(102),
        predicate_type: 7,
        participants: vec![EntityId::new(3), EntityId::new(4)],
    };
    let edge3 = AggregationEdge {
        id: HyperEdgeId::new(103),
        predicate_type: 7,
        participants: vec![EntityId::new(1), EntityId::new(3)],
    };

    let nodes_original = vec![node1.clone(), node2.clone(), node3.clone(), node4.clone()];
    let edges_original = vec![edge1.clone(), edge2.clone(), edge3.clone()];

    // Shuffled / reversed permutation
    let nodes_permuted = vec![node3, node1, node4, node2];
    let edges_permuted = vec![edge3, edge1, edge2];

    let cfg = AggregationConfig {
        gmm_deterministic_seed: 1234,
        min_cluster_size: 2,
        max_clusters: 2,
        clustering_tau_threshold: 0.1,
        stability_cycles_required: 1,
        ..Default::default()
    };
    let llm = TestMockLlm;

    let mut tracker1 = CommunityStabilityTracker::new();
    let mut sink1 = TestMockSink::new();
    let (res1, alphas1) = run_aggregation_pass(
        &nodes_original,
        &edges_original,
        &cfg,
        &llm,
        &mut tracker1,
        &mut sink1,
    )
    .await
    .unwrap();

    let mut tracker2 = CommunityStabilityTracker::new();
    let mut sink2 = TestMockSink::new();
    let (res2, alphas2) = run_aggregation_pass(
        &nodes_permuted,
        &edges_permuted,
        &cfg,
        &llm,
        &mut tracker2,
        &mut sink2,
    )
    .await
    .unwrap();

    // Permutation invariance due to canonical input sorting by entity and edge ID
    assert_eq!(res1, res2);
    assert_eq!(alphas1, alphas2);
    assert_eq!(sink1.tombstones, sink2.tombstones);
    assert_eq!(sink1.super_edges, sink2.super_edges);
}

#[tokio::test]
async fn test_property_like_invariance_with_seeded_rng() {
    let rng = SeededRng::new(0xCAFE_BABE_DEAD_BEEF);
    let cfg = AggregationConfig {
        min_cluster_size: 2,
        max_clusters: 3,
        stability_cycles_required: 1,
        ..Default::default()
    };
    let llm = TestMockLlm;

    // Run 5 randomized rounds using SeededRng
    for round in 0..5 {
        let node_count = (rng.next_u64() % 8 + 3) as usize; // 3 to 10 nodes
        let mut nodes = Vec::with_capacity(node_count);
        for i in 0..node_count {
            let mut rand_bytes = [0u8; 8];
            rng.fill_bytes(&mut rand_bytes);
            let val1 = (u32::from_le_bytes(rand_bytes[0..4].try_into().unwrap()) as f32)
                / (u32::MAX as f32);
            let val2 = (u32::from_le_bytes(rand_bytes[4..8].try_into().unwrap()) as f32)
                / (u32::MAX as f32);

            nodes.push(AggregationNode {
                entity: EntityId::new((i as u64) + 1),
                embedding: vec![val1, val2],
                type_id: (i % 2) as u32,
            });
        }

        let mut tracker1 = CommunityStabilityTracker::new();
        let mut sink1 = TestMockSink::new();
        let run1 = run_aggregation_pass(&nodes, &[], &cfg, &llm, &mut tracker1, &mut sink1).await;
        assert!(run1.is_ok(), "Round {} run 1 failed: {:?}", round, run1);

        let mut tracker2 = CommunityStabilityTracker::new();
        let mut sink2 = TestMockSink::new();
        let run2 = run_aggregation_pass(&nodes, &[], &cfg, &llm, &mut tracker2, &mut sink2).await;
        assert!(run2.is_ok(), "Round {} run 2 failed: {:?}", round, run2);

        let (res1, alphas1) = run1.unwrap();
        let (res2, alphas2) = run2.unwrap();
        assert_eq!(res1, res2);
        assert_eq!(alphas1, alphas2);
    }
}

#[test]
fn test_entity_community_hash_permutation_invariance() {
    let members_a = vec![EntityId::new(10), EntityId::new(20), EntityId::new(30)];
    let members_b = vec![EntityId::new(30), EntityId::new(10), EntityId::new(20)];

    let hash_a = compute_entity_community_hash(&members_a);
    let hash_b = compute_entity_community_hash(&members_b);

    assert_eq!(
        hash_a, hash_b,
        "Hash must be permutation invariant over entity order"
    );
}
