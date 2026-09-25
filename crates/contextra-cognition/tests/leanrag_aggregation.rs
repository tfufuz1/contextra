//! Integrationstests für LeanRAG Aggregation Phase (Consolidation Stage 3).

use contextra_cognition::aggregation_phase::{
    check_compaction_budget, run_aggregation_pass, AggregationConfig, AggregationEdge,
    AggregationNode, SuperEdgeDraft, SuperEdgeSink,
};
use contextra_cognition::memory_consolidation::CommunityStabilityTracker;
use contextra_cognition::synthesis_phase::SegmentSynthesisResult;
use contextra_graph::HyperEdgeId;
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_types::{ContextraError, EntityId, Result};

struct MockLlm {
    fail_on_prompt: Option<String>,
}

impl LlmTextGenerator for MockLlm {
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move {
            if let Some(ref fail) = self.fail_on_prompt {
                if prompt.contains(fail) {
                    return Err(ContextraError::Internal("Simulated LLM Failure".into()));
                }
            }
            Ok(format!("Synthesized summary for: {}", prompt))
        })
    }
}

struct MockSink {
    tombstoned: Vec<HyperEdgeId>,
    written_super_edges: Vec<SuperEdgeDraft>,
    committed: bool,
    fail_on_write: bool,
}

impl MockSink {
    fn new() -> Self {
        Self {
            tombstoned: Vec::new(),
            written_super_edges: Vec::new(),
            committed: false,
            fail_on_write: false,
        }
    }
}

impl SuperEdgeSink for MockSink {
    fn tombstone_edge(&mut self, id: HyperEdgeId) -> Result<()> {
        if self.fail_on_write {
            return Err(ContextraError::Internal("Sink write failed".into()));
        }
        self.tombstoned.push(id);
        Ok(())
    }

    fn write_super_edge(&mut self, draft: SuperEdgeDraft) -> Result<HyperEdgeId> {
        if self.fail_on_write {
            return Err(ContextraError::Internal("Sink write failed".into()));
        }
        let id = HyperEdgeId::new((self.written_super_edges.len() + 1000) as u64);
        self.written_super_edges.push(draft);
        Ok(id)
    }

    fn commit(&mut self) -> Result<()> {
        self.committed = true;
        Ok(())
    }
}

#[tokio::test]
async fn test_gmm_determinism_and_permutation_invariance() {
    // 6 nodes in 2 distinct 2D clusters
    let nodes_a = vec![
        AggregationNode {
            entity: EntityId::new(1),
            embedding: vec![10.0, 10.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(2),
            embedding: vec![10.1, 10.1],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(3),
            embedding: vec![10.2, 9.9],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(4),
            embedding: vec![-10.0, -10.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(5),
            embedding: vec![-10.1, -10.1],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(6),
            embedding: vec![-9.9, -10.2],
            type_id: 1,
        },
    ];

    // Permuted order
    let mut nodes_b = nodes_a.clone();
    nodes_b.reverse();

    let edges = vec![];
    let cfg = AggregationConfig {
        min_cluster_size: 2,
        max_clusters: 2,
        stability_cycles_required: 1,
        ..Default::default()
    };

    let llm = MockLlm {
        fail_on_prompt: None,
    };

    let mut tracker1 = CommunityStabilityTracker::new();
    let mut sink1 = MockSink::new();

    let (res1, alpha1) =
        run_aggregation_pass(&nodes_a, &edges, &cfg, &llm, &mut tracker1, &mut sink1)
            .await
            .unwrap();

    let mut tracker2 = CommunityStabilityTracker::new();
    let mut sink2 = MockSink::new();

    let (res2, alpha2) =
        run_aggregation_pass(&nodes_b, &edges, &cfg, &llm, &mut tracker2, &mut sink2)
            .await
            .unwrap();

    assert_eq!(res1, res2);
    assert_eq!(alpha1, alpha2);
}

#[tokio::test]
async fn test_clustering_two_distinct_blobs() {
    let nodes = vec![
        AggregationNode {
            entity: EntityId::new(1),
            embedding: vec![5.0, 5.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(2),
            embedding: vec![5.1, 5.2],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(3),
            embedding: vec![-5.0, -5.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(4),
            embedding: vec![-5.2, -5.1],
            type_id: 1,
        },
    ];

    let cfg = AggregationConfig {
        min_cluster_size: 2,
        max_clusters: 2,
        stability_cycles_required: 1,
        ..Default::default()
    };

    let llm = MockLlm {
        fail_on_prompt: None,
    };
    let mut tracker = CommunityStabilityTracker::new();
    let mut sink = MockSink::new();

    let (_res, alphas) = run_aggregation_pass(&nodes, &[], &cfg, &llm, &mut tracker, &mut sink)
        .await
        .unwrap();

    assert_eq!(
        alphas.len(),
        2,
        "Expected 2 alpha nodes for 2 distinct blobs"
    );
    assert!(sink.committed);
}

#[tokio::test]
async fn test_type_denoising_tombstones_inconsistent_edge() {
    let nodes = vec![
        AggregationNode {
            entity: EntityId::new(1),
            embedding: vec![1.0, 0.0],
            type_id: 10,
        },
        AggregationNode {
            entity: EntityId::new(2),
            embedding: vec![1.1, 0.0],
            type_id: 10,
        },
        AggregationNode {
            entity: EntityId::new(3),
            embedding: vec![0.0, 1.0],
            type_id: 99, // Rare type
        },
    ];

    // Predicate type 100 used 9 times between 10 and 10, 1 time between 10 and 99
    let mut edges = Vec::new();
    for i in 1..=9 {
        edges.push(AggregationEdge {
            id: HyperEdgeId::new(i),
            predicate_type: 100,
            participants: vec![EntityId::new(1), EntityId::new(2)],
        });
    }
    // Inconsistent edge
    edges.push(AggregationEdge {
        id: HyperEdgeId::new(10),
        predicate_type: 100,
        participants: vec![EntityId::new(1), EntityId::new(3)],
    });

    let cfg = AggregationConfig {
        min_type_compat_score: 0.2, // 1/20 = 0.05 < 0.2 -> tombstone
        min_cluster_size: 1,
        stability_cycles_required: 1,
        ..Default::default()
    };

    let llm = MockLlm {
        fail_on_prompt: None,
    };
    let mut tracker = CommunityStabilityTracker::new();
    let mut sink = MockSink::new();

    let (res, _alphas) = run_aggregation_pass(&nodes, &edges, &cfg, &llm, &mut tracker, &mut sink)
        .await
        .unwrap();

    assert_eq!(res.raw_edges_tombstoned, 1);
    assert_eq!(sink.tombstoned, vec![HyperEdgeId::new(10)]);
}

#[tokio::test]
async fn test_llm_budgeting_and_per_cluster_error_recovery() {
    let nodes = vec![
        AggregationNode {
            entity: EntityId::new(1),
            embedding: vec![1.0, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(2),
            embedding: vec![1.1, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(3),
            embedding: vec![-1.0, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(4),
            embedding: vec![-1.1, 0.0],
            type_id: 1,
        },
    ];

    let cfg = AggregationConfig {
        min_cluster_size: 2,
        max_clusters: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 10,
        ..Default::default()
    };

    // LLM fails on cluster 0 (prompt contains "cluster 0")
    let llm = MockLlm {
        fail_on_prompt: Some("cluster 0".to_string()),
    };
    let mut tracker = CommunityStabilityTracker::new();
    let mut sink = MockSink::new();

    let (_res, alphas) = run_aggregation_pass(&nodes, &[], &cfg, &llm, &mut tracker, &mut sink)
        .await
        .expect("Pass should succeed even if one cluster synthesis fails");

    assert_eq!(alphas.len(), 1, "Cluster 1 synthesis should succeed");
    assert_eq!(alphas[0].cluster_id, 1);
}

#[test]
fn test_memory_budget_exceeded() {
    let cfg = AggregationConfig {
        max_compaction_peak_memory_mb: 512,
        ..Default::default()
    };

    // 600 MB peak bytes
    let peak_bytes = 600 * 1024 * 1024;
    let res = check_compaction_budget(peak_bytes, &cfg);

    match res {
        Err(ContextraError::MemoryBudgetExceeded { used_mb, limit_mb }) => {
            assert_eq!(used_mb, 600);
            assert_eq!(limit_mb, 512);
        }
        _ => panic!("Expected MemoryBudgetExceeded error"),
    }
}

#[tokio::test]
async fn test_super_edge_carries_correct_child_edge_ids() {
    // 2 clusters
    let nodes = vec![
        AggregationNode {
            entity: EntityId::new(1),
            embedding: vec![10.0, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(2),
            embedding: vec![10.1, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(3),
            embedding: vec![-10.0, 0.0],
            type_id: 1,
        },
        AggregationNode {
            entity: EntityId::new(4),
            embedding: vec![-10.1, 0.0],
            type_id: 1,
        },
    ];

    // Spanning edges between clusters
    let edges = vec![
        AggregationEdge {
            id: HyperEdgeId::new(50),
            predicate_type: 1,
            participants: vec![EntityId::new(1), EntityId::new(3)],
        },
        AggregationEdge {
            id: HyperEdgeId::new(51),
            predicate_type: 1,
            participants: vec![EntityId::new(2), EntityId::new(4)],
        },
    ];

    let cfg = AggregationConfig {
        min_cluster_size: 2,
        max_clusters: 2,
        clustering_tau_threshold: 0.1,
        stability_cycles_required: 1,
        ..Default::default()
    };

    let llm = MockLlm {
        fail_on_prompt: None,
    };
    let mut tracker = CommunityStabilityTracker::new();
    let mut sink = MockSink::new();

    let (res, _alphas) = run_aggregation_pass(&nodes, &edges, &cfg, &llm, &mut tracker, &mut sink)
        .await
        .unwrap();

    assert_eq!(res.abstract_hyperedges_created, 1);
    assert_eq!(res.child_edge_ids_written, 2);
    assert_eq!(sink.written_super_edges.len(), 1);
    assert_eq!(
        sink.written_super_edges[0].child_edge_ids,
        vec![HyperEdgeId::new(50), HyperEdgeId::new(51)]
    );
}

#[tokio::test]
async fn test_no_commit_on_sink_error() {
    // Non-finite embedding to trigger immediate error
    let invalid_nodes = vec![AggregationNode {
        entity: EntityId::new(1),
        embedding: vec![f32::NAN, 0.0],
        type_id: 1,
    }];

    let cfg = AggregationConfig::default();
    let llm = MockLlm {
        fail_on_prompt: None,
    };
    let mut tracker = CommunityStabilityTracker::new();
    let mut sink = MockSink::new();

    let res = run_aggregation_pass(&invalid_nodes, &[], &cfg, &llm, &mut tracker, &mut sink).await;

    assert!(res.is_err());
    assert!(!sink.committed, "Sink commit must NOT be called on error");
}

#[test]
fn test_type_renaming_and_deprecation() {
    // Verify SegmentSynthesisResult in synthesis_phase
    let res = SegmentSynthesisResult {
        synthesized_chunks: vec![],
        skipped_segments: 0,
    };
    assert_eq!(res.skipped_segments, 0);

    // Verify root re-export SynthesisPhaseResult in memory_consolidation remains intact
    let root_res = contextra_cognition::memory_consolidation::SynthesisPhaseResult {
        synthesized: vec![],
        deferred_community_hashes: vec![],
    };
    assert!(root_res.synthesized.is_empty());
}
