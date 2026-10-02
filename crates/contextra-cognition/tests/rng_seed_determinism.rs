use contextra_cognition::aggregation_phase::{
    run_aggregation_pass, AggregationConfig, AggregationEdge, AggregationNode, SuperEdgeDraft,
    SuperEdgeSink,
};
use contextra_cognition::context_compaction::{CompactionStrategy, ContextCompactor};
use contextra_cognition::memory_consolidation::{
    run_consolidation_pass, CommunityStabilityTracker, ConsolidationConfig,
};
use contextra_graph::HyperEdgeId;
use contextra_ports::{BoxFuture, LlmTextGenerator, SeededRng};
use contextra_types::{ContextChunk, DocId, EntityId, Result, TokenBudget};
use std::sync::Arc;

struct TestMockLlm;

impl LlmTextGenerator for TestMockLlm {
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move {
            Ok(format!(
                "Deterministic LLM summary for prompt len {}",
                prompt.len()
            ))
        })
    }
}

struct TestMockSink {
    tombstones: Vec<HyperEdgeId>,
    super_edges: Vec<SuperEdgeDraft>,
}

impl TestMockSink {
    fn new() -> Self {
        Self {
            tombstones: Vec::new(),
            super_edges: Vec::new(),
        }
    }
}

impl SuperEdgeSink for TestMockSink {
    fn tombstone_edge(&mut self, id: HyperEdgeId) -> Result<()> {
        self.tombstones.push(id);
        Ok(())
    }

    fn write_super_edge(&mut self, draft: SuperEdgeDraft) -> Result<HyperEdgeId> {
        let id = HyperEdgeId(self.super_edges.len() as u64 + 100);
        self.super_edges.push(draft);
        Ok(id)
    }

    fn commit(&mut self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_consolidation_pass_determinism() {
    let emb1 = vec![1.0, 0.0, 0.0, 0.0];
    let emb2 = vec![0.99, 0.01, 0.0, 0.0];
    let emb3 = vec![0.0, 1.0, 0.0, 0.0];

    let turns = vec![
        (DocId::new(1), emb1.clone()),
        (DocId::new(2), emb2.clone()),
        (DocId::new(3), emb3.clone()),
    ];

    let config = ConsolidationConfig::default();

    let res1 = run_consolidation_pass(&turns, &config);
    let res2 = run_consolidation_pass(&turns, &config);

    assert_eq!(
        res1, res2,
        "run_consolidation_pass must be strictly deterministic across two runs"
    );
}

#[test]
fn test_context_compactor_seed_determinism() {
    let budget = TokenBudget::new(100, 0);
    let rng1 = Arc::new(SeededRng::new(12345));
    let rng2 = Arc::new(SeededRng::new(12345));

    let compactor1 =
        ContextCompactor::new(budget.clone(), CompactionStrategy::Truncate).with_rng(rng1);
    let compactor2 = ContextCompactor::new(budget, CompactionStrategy::Truncate).with_rng(rng2);

    let chunks = vec![
        ContextChunk {
            doc_id: DocId::new(1),
            content: "First chunk text".to_string(),
            relevance: 0.9,
            token_count: 50,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        },
        ContextChunk {
            doc_id: DocId::new(2),
            content: "Second chunk text".to_string(),
            relevance: 0.8,
            token_count: 60,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        },
    ];

    let out1 = compactor1.compact(chunks.clone());
    let out2 = compactor2.compact(chunks);

    assert_eq!(out1.retained_chunks.len(), out2.retained_chunks.len());
    for (c1, c2) in out1.retained_chunks.iter().zip(out2.retained_chunks.iter()) {
        assert_eq!(c1.doc_id, c2.doc_id);
        assert_eq!(c1.content, c2.content);
    }
    assert_eq!(out1.tokens_used, out2.tokens_used);
}

#[tokio::test]
async fn test_aggregation_pass_seed_determinism() {
    let node1 = AggregationNode {
        entity: EntityId::new(1),
        embedding: vec![1.0, 0.1, 0.0, 0.0],
        type_id: 1,
    };
    let node2 = AggregationNode {
        entity: EntityId::new(2),
        embedding: vec![0.9, 0.2, 0.0, 0.0],
        type_id: 1,
    };
    let node3 = AggregationNode {
        entity: EntityId::new(3),
        embedding: vec![0.0, 0.0, 1.0, 0.1],
        type_id: 2,
    };
    let node4 = AggregationNode {
        entity: EntityId::new(4),
        embedding: vec![0.0, 0.0, 0.9, 0.2],
        type_id: 2,
    };

    let nodes = vec![node1, node2, node3, node4];

    let edge1 = AggregationEdge {
        id: HyperEdgeId(10),
        predicate_type: 100,
        participants: vec![EntityId::new(1), EntityId::new(2)],
    };
    let edge2 = AggregationEdge {
        id: HyperEdgeId(11),
        predicate_type: 100,
        participants: vec![EntityId::new(3), EntityId::new(4)],
    };

    let edges = vec![edge1, edge2];

    let config = AggregationConfig {
        gmm_deterministic_seed: 42,
        stability_cycles_required: 1,
        min_cluster_size: 2,
        ..Default::default()
    };

    let llm = TestMockLlm;

    let mut tracker1 = CommunityStabilityTracker::new();
    let mut sink1 = TestMockSink::new();

    let (res1, alphas1) =
        run_aggregation_pass(&nodes, &edges, &config, &llm, &mut tracker1, &mut sink1)
            .await
            .expect("run_aggregation_pass run 1");

    let mut tracker2 = CommunityStabilityTracker::new();
    let mut sink2 = TestMockSink::new();

    let (res2, alphas2) =
        run_aggregation_pass(&nodes, &edges, &config, &llm, &mut tracker2, &mut sink2)
            .await
            .expect("run_aggregation_pass run 2");

    assert_eq!(res1, res2, "Aggregation Phase Results must be identical");
    assert_eq!(alphas1, alphas2, "Alpha Nodes must be identical");
    assert_eq!(sink1.tombstones, sink2.tombstones);
    assert_eq!(sink1.super_edges, sink2.super_edges);
}
