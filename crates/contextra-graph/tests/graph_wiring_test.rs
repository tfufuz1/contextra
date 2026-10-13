use contextra_graph::{
    CsrGraph, ExtractedRelation, GraphWiring, GraphWiringDependencies,
    RuleBasedEntityExtractor,
};
use contextra_ports::{BoxFuture, GraphIndex, LlmTextGenerator};
use contextra_types::{DocId, Entity, EntityId, PprConfig, TxId};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct MockLlmGenerator {
    call_count: AtomicUsize,
    response: String,
}

impl MockLlmGenerator {
    fn new(response: impl Into<String>) -> Self {
        Self {
            call_count: AtomicUsize::new(0),
            response: response.into(),
        }
    }

    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl LlmTextGenerator for MockLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, contextra_ports::Result<String>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }
}

#[tokio::test]
async fn test_graph_wiring_build_contract() {
    let graph = Arc::new(CsrGraph::new());
    let deps = GraphWiringDependencies {
        graph: graph.clone(),
        llm_generator: None,
        entity_extractor: None,
        apprh_selector: None,
    };

    let wiring = GraphWiring::build(deps).expect("build success");
    assert!(Arc::ptr_eq(wiring.graph(), &graph));
}

#[tokio::test]
async fn test_graph_wiring_ppr_pipeline_small_and_large_graph_determinism() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);

    // 1. Small graph PPR pipeline execution
    for id in 1..=10 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("E{id}"), "Entity"),
            )
            .await
            .unwrap();
    }
    GraphIndex::add_edge(
        graph.as_ref(),
        tx,
        contextra_types::Edge::new(EntityId::new(1), EntityId::new(2), "rel")
            .with_weight(1.0),
    )
    .await
    .unwrap();
    graph.commit(tx).await.unwrap();

    let deps = GraphWiringDependencies {
        graph: graph.clone(),
        llm_generator: None,
        entity_extractor: None,
        apprh_selector: None,
    };
    let wiring = GraphWiring::build(deps).unwrap();

    let config = PprConfig::default();
    let seeds = vec![EntityId::new(1)];

    let ppr1 = wiring
        .compute_ppr_pipeline(&seeds, &config)
        .await
        .expect("ppr small graph");
    let ppr2 = wiring
        .compute_ppr_pipeline(&seeds, &config)
        .await
        .expect("ppr small graph repeat");

    assert_eq!(ppr1, ppr2, "PPR results must be 100% bit-identically deterministic");

    // 2. Large graph PPR pipeline execution (triggers APPRH selector routing >= 100 nodes)
    let large_graph = Arc::new(CsrGraph::new());
    let tx_large = TxId::new(2);

    for id in 1..=150 {
        large_graph
            .add_entity(
                tx_large,
                Entity::new(EntityId::new(id), format!("Entity_{id}"), "Node"),
            )
            .await
            .unwrap();
    }
    for i in 1..150 {
        GraphIndex::add_edge(
            large_graph.as_ref(),
            tx_large,
            contextra_types::Edge::new(EntityId::new(i), EntityId::new(i + 1), "link")
                .with_weight(0.9),
        )
        .await
        .unwrap();
    }
    large_graph.commit(tx_large).await.unwrap();

    let large_deps = GraphWiringDependencies {
        graph: large_graph.clone(),
        llm_generator: None,
        entity_extractor: None,
        apprh_selector: None,
    };
    let large_wiring = GraphWiring::build(large_deps).unwrap();

    let ppr_large1 = large_wiring
        .compute_ppr_pipeline(&seeds, &config)
        .await
        .expect("ppr large graph");
    let ppr_large2 = large_wiring
        .compute_ppr_pipeline(&seeds, &config)
        .await
        .expect("ppr large graph repeat");

    assert_eq!(
        ppr_large1, ppr_large2,
        "APPRH PPR results on large graph must be 100% bit-identically deterministic"
    );
    assert!(!ppr_large1.is_empty());
}

#[tokio::test]
async fn test_graph_wiring_suggest_and_validate_hyperedges() {
    let graph = Arc::new(CsrGraph::new());
    let generator = Arc::new(MockLlmGenerator::new("YES: works_with"));

    let deps = GraphWiringDependencies {
        graph,
        llm_generator: Some(generator.clone()),
        entity_extractor: None,
        apprh_selector: None,
    };
    let wiring = GraphWiring::build(deps).unwrap();

    let doc1 = DocId::from_key("doc1").unwrap();
    let doc2 = DocId::from_key("doc2").unwrap();
    let e1 = EntityId::new(10);
    let e2 = EntityId::new(20);

    let entities_per_doc = vec![(doc1, vec![e1, e2]), (doc2, vec![e1, e2])];
    let label_fn = |id: EntityId| Some(format!("Label_{}", id.inner()));

    let validated = wiring
        .suggest_and_validate_hyperedges(&entities_per_doc, 2, 5, &label_fn)
        .await
        .expect("validation success");

    assert_eq!(validated.len(), 1);
    assert!(validated[0].accepted);
    assert_eq!(validated[0].predicate, "works_with");
    assert_eq!(generator.calls(), 1);
}

#[tokio::test]
async fn test_graph_wiring_suggest_and_validate_unconfigured_llm_port_returns_err() {
    let graph = Arc::new(CsrGraph::new());
    let deps = GraphWiringDependencies {
        graph,
        llm_generator: None,
        entity_extractor: None,
        apprh_selector: None,
    };
    let wiring = GraphWiring::build(deps).unwrap();

    let doc1 = DocId::from_key("doc1").unwrap();
    let entities_per_doc = vec![(doc1, vec![EntityId::new(1), EntityId::new(2)])];
    let label_fn = |_| None;

    let res = wiring
        .suggest_and_validate_hyperedges(&entities_per_doc, 1, 5, &label_fn)
        .await;

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.to_string().contains("No LLM text generator port configured"),
        "Expected error for unconfigured LLM port, got: {err}"
    );
}

#[tokio::test]
async fn test_graph_wiring_relation_extraction_and_ingestion() {
    let graph = Arc::new(CsrGraph::new());
    let extractor = Arc::new(RuleBasedEntityExtractor::new(0.5));

    let deps = GraphWiringDependencies {
        graph: graph.clone(),
        llm_generator: None,
        entity_extractor: Some(extractor),
        apprh_selector: None,
    };
    let wiring = GraphWiring::build(deps).unwrap();

    let sample_text = "Alice works for Contextra. Bob works at Contextra.";
    let doc_id = Some(DocId::from_key("doc_sample").unwrap());

    let hyperedges = wiring
        .extract_and_ingest_relations(sample_text, 0.7, doc_id)
        .expect("extract and ingest success");

    assert_eq!(hyperedges.len(), 2);
    assert_eq!(hyperedges[0].participants.len(), 2);

    // Also test direct ingestion of pre-extracted relations
    let manual_rel = ExtractedRelation::new("Charlie", "Contextra", "WORKS_AT", 0.9);
    let ingested = wiring
        .ingest_relations(&[manual_rel], 0.8, doc_id)
        .expect("ingest success");

    assert_eq!(ingested.len(), 1);
}
