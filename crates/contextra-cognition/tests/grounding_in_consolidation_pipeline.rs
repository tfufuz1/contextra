use contextra_cognition::consolidation_executor::execute_background_consolidation_with_rich_validator;
use contextra_cognition::memory_consolidation::{
    compute_community_hash, run_structural_synthesis_pass, CommunityStabilityTracker,
    ConsolidationConfig, SynthesisConfig,
};
use contextra_graph::CsrGraph;
use contextra_ports::{
    BoxFuture, GroundingAssessment, GroundingValidator, LlmTextGenerator,
    ResponseGroundingValidator,
};
use contextra_store::LsmStorage;
use contextra_types::{ContextChunk, DocId, Result};
use contextra_vector::HnswIndex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

struct TestMockLlm;

impl LlmTextGenerator for TestMockLlm {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move { Ok("Synthesized memory text for testing.".to_string()) })
    }
}

struct TestMockValidator {
    score: f32,
}

impl ResponseGroundingValidator for TestMockValidator {
    fn score_grounding(&self, _response: &str, _sources: &[&str]) -> Result<f32> {
        Ok(self.score)
    }
}

struct TestMockRichValidator {
    score: f32,
}

impl GroundingValidator for TestMockRichValidator {
    fn validate_grounding<'a>(
        &'a self,
        _response: &'a str,
        _chunks: &'a [ContextChunk],
    ) -> BoxFuture<'a, Result<GroundingAssessment>> {
        let score = self.score;
        Box::pin(async move {
            if score < 0.6 {
                Err(contextra_types::ContextraError::PolicyViolation(
                    "LowConfidenceGrounding: score below threshold".to_string(),
                ))
            } else {
                Ok(GroundingAssessment {
                    score,
                    is_grounded: true,
                    reason: Some("Validated".to_string()),
                })
            }
        })
    }
}

#[tokio::test]
async fn test_grounding_validation_pass_and_fail_modes() {
    let llm = TestMockLlm;

    let config = SynthesisConfig {
        min_community_size: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 10,
        min_grounding_score: Some(0.70),
    };

    let members = vec![DocId::new(10), DocId::new(20)];
    let comm_hash = compute_community_hash(&members);
    let stable_communities = vec![(comm_hash, members)];

    let mut source_texts = std::collections::HashMap::new();
    source_texts.insert(DocId::new(10), "Source text 1".to_string());
    source_texts.insert(DocId::new(20), "Source text 2".to_string());

    // 1. Validator passing (score 0.90 >= threshold 0.70)
    let validator_pass = TestMockValidator { score: 0.90 };
    let res_pass = run_structural_synthesis_pass(
        &stable_communities,
        &source_texts,
        &llm,
        &config,
        Some(&validator_pass),
    )
    .await
    .expect("synthesis pass should succeed");

    assert_eq!(
        res_pass.synthesized.len(),
        1,
        "Passing grounding score must synthesize 1 chunk"
    );

    // 2. Validator failing (score 0.40 < threshold 0.70)
    let validator_fail = TestMockValidator { score: 0.40 };
    let res_fail = run_structural_synthesis_pass(
        &stable_communities,
        &source_texts,
        &llm,
        &config,
        Some(&validator_fail),
    )
    .await
    .expect("synthesis pass should succeed");

    assert_eq!(
        res_fail.synthesized.len(),
        0,
        "Failing grounding score must discard synthesis chunk"
    );

    // 3. Validator missing when min_grounding_score is set
    let res_missing_validator =
        run_structural_synthesis_pass(&stable_communities, &source_texts, &llm, &config, None)
            .await
            .expect("synthesis pass should succeed");

    assert_eq!(
        res_missing_validator.synthesized.len(),
        0,
        "Missing validator when min_grounding_score is set must discard ungrounded synthesis chunk"
    );
}

#[tokio::test]
async fn test_consolidation_pipeline_rich_validator_wiring() {
    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let collection = Arc::new(contextra_engine::collection::Collection::new(
        "test_grounding_pipeline".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_engine::Language::English,
    ));

    collection
        .insert_typed(
            "doc_10",
            &[1.0, 0.0, 0.0, 0.0],
            contextra_types::MemoryType::Episodic,
            Some(serde_json::json!({"text": "Grounded source 1"})),
        )
        .await
        .unwrap();

    collection
        .insert_typed(
            "doc_20",
            &[0.0, 1.0, 0.0, 0.0],
            contextra_types::MemoryType::Episodic,
            Some(serde_json::json!({"text": "Grounded source 2"})),
        )
        .await
        .unwrap();

    collection
        .relate("doc_10", "doc_20", "co_occurrence")
        .await
        .unwrap();

    let turns = vec![
        (DocId::new(1), vec![1.0, 0.0, 0.0, 0.0]),
        (DocId::new(2), vec![0.0, 1.0, 0.0, 0.0]),
    ];

    let consolidation_config = ConsolidationConfig::default();
    let synthesis_config = SynthesisConfig {
        min_community_size: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 10,
        min_grounding_score: Some(0.65),
    };

    let llm = TestMockLlm;
    let rich_validator_failing = TestMockRichValidator { score: 0.30 };

    let mut tracker = CommunityStabilityTracker::new();

    let (_cons_res, synth_res) = execute_background_consolidation_with_rich_validator(
        &collection,
        &turns,
        &consolidation_config,
        Some(&synthesis_config),
        Some(&llm),
        None,
        Some(&rich_validator_failing),
        Some(&mut tracker),
    )
    .await
    .unwrap();

    let synth = synth_res.expect("Synthesis result expected");
    assert_eq!(
        synth.synthesized.len(),
        0,
        "Rich GroundingValidator returning PolicyViolation/LowScore must cause synthesis chunk rejection"
    );
}
