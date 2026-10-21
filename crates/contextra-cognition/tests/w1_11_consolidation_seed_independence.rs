// FILE-CONTEXT
// ZWECK: Determinismus- & Seed-Unabhängigkeitstest für Consolidation / Aggregation (W1-11 Stage 2).

use contextra_cognition::aggregation_phase::{
    run_aggregation_pass, AggregationConfig, AggregationEdge, AggregationNode, SuperEdgeDraft, SuperEdgeSink,
};
use contextra_graph::HyperEdgeId;
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_types::{EntityId, Result};

#[derive(Default)]
struct DummySink {
    tombstones: Vec<HyperEdgeId>,
    super_edges: Vec<SuperEdgeDraft>,
}

impl SuperEdgeSink for DummySink {
    fn tombstone_edge(&mut self, id: HyperEdgeId) -> Result<()> {
        self.tombstones.push(id);
        Ok(())
    }

    fn write_super_edge(&mut self, draft: SuperEdgeDraft) -> Result<HyperEdgeId> {
        self.super_edges.push(draft);
        Ok(HyperEdgeId(self.super_edges.len() as u64))
    }

    fn commit(&mut self) -> Result<()> {
        Ok(())
    }
}

struct DummyLlm;

impl LlmTextGenerator for DummyLlm {
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move { Ok(format!("Summary for: {}", prompt)) })
    }
}

#[tokio::test]
async fn consolidation_output_independent_of_hash_seed() {
    let node1 = AggregationNode {
        entity: EntityId::new(10),
        embedding: vec![1.0, 0.0, 0.0, 0.0],
        type_id: 1,
    };
    let node2 = AggregationNode {
        entity: EntityId::new(20),
        embedding: vec![0.9, 0.1, 0.0, 0.0],
        type_id: 1,
    };
    let node3 = AggregationNode {
        entity: EntityId::new(30),
        embedding: vec![0.0, 0.0, 1.0, 0.0],
        type_id: 2,
    };

    let edge1 = AggregationEdge {
        id: HyperEdgeId(100),
        predicate_type: 1,
        participants: vec![node1.entity, node2.entity],
    };
    let edge2 = AggregationEdge {
        id: HyperEdgeId(200),
        predicate_type: 1,
        participants: vec![node2.entity, node3.entity],
    };

    let cfg = AggregationConfig {
        gmm_deterministic_seed: 42,
        min_cluster_size: 1,
        max_clusters: 2,
        stability_cycles_required: 1,
        ..Default::default()
    };

    let llm = DummyLlm;

    // Run 1: Order A
    let nodes_a = vec![node1.clone(), node2.clone(), node3.clone()];
    let edges_a = vec![edge1.clone(), edge2.clone()];
    let mut tracker1 = contextra_cognition::memory_consolidation::CommunityStabilityTracker::default();
    let mut sink1 = DummySink::default();

    let (res1, alpha1) = run_aggregation_pass(&nodes_a, &edges_a, &cfg, &llm, &mut tracker1, &mut sink1)
        .await
        .expect("pass 1");

    // Run 2: Order B (reversed / different insertion order)
    let nodes_b = vec![node3.clone(), node1.clone(), node2.clone()];
    let edges_b = vec![edge2.clone(), edge1.clone()];
    let mut tracker2 = contextra_cognition::memory_consolidation::CommunityStabilityTracker::default();
    let mut sink2 = DummySink::default();

    let (res2, alpha2) = run_aggregation_pass(&nodes_b, &edges_b, &cfg, &llm, &mut tracker2, &mut sink2)
        .await
        .expect("pass 2");

    assert_eq!(res1, res2);
    assert_eq!(alpha1, alpha2);
    assert_eq!(sink1.tombstones, sink2.tombstones);
    assert_eq!(sink1.super_edges, sink2.super_edges);
}
