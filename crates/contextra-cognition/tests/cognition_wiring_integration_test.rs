use contextra_cognition::wiring::{CognitionWiring, CognitionWiringDeps};
use contextra_cognition::{ConsolidationConfig, SynthesisConfig};
use contextra_engine::collection::Collection;
use contextra_engine::Language;
use contextra_graph::CsrGraph;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_vector::{HnswConfig, HnswIndex};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_cognition_wiring_build_and_run_pipeline() {
    let dir = tempdir().expect("tempdir");
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .expect("lsm storage"),
    );
    let index = Arc::new(
        HnswIndex::try_new(HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .expect("hnsw index"),
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let collection = Arc::new(Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        Language::English,
    ));

    let deps = CognitionWiringDeps {
        collection,
        consolidation_config: ConsolidationConfig::default(),
        synthesis_config: SynthesisConfig::default(),
        maintenance_config: None,
        llm: None,
        rich_validator: None,
        leanrag_config: None,
        clock: None,
        rng: None,
    };

    let wiring = CognitionWiring::build(deps).expect("CognitionWiring::build");
    assert!(Arc::strong_count(wiring.engine()) >= 1);
    assert!(Arc::strong_count(wiring.scheduler()) >= 1);

    let res = wiring.run_pipeline().await.expect("run_pipeline");
    assert_eq!(res.structural.segments_created, 0);
    assert!(res.synthesis.synthesized.is_empty());
    assert!(res.aggregation.is_none());
}
