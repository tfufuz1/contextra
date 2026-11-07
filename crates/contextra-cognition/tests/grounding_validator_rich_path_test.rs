//! # Integrationstest: Rich-Grounding-Pfad mit GASP-Kalibrierung (`tests/grounding_validator_rich_path_test.rs`)
//!
//! Validiert das Zusammenwirken von `GaspValidator` (isotone Regression/Kalibrierung)
//! und `execute_background_consolidation_with_rich_validator` zum Schutz vor Halluzinationen.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::get_first,
    clippy::panic,
    clippy::unnecessary_cast,
    clippy::disallowed_methods,
    clippy::approx_constant,
    clippy::unreachable
)]

use contextra_cognition::consolidation_executor::execute_background_consolidation_with_rich_validator;
use contextra_cognition::memory_consolidation::{
    CommunityStabilityTracker, ConsolidationConfig, SynthesisConfig,
};
use contextra_graph::CsrGraph;

use contextra_ports::{
    BoxFuture, GroundingValidator, LlmTextGenerator, ResponseGroundingValidator,
};
use contextra_testkit::{InMemoryStorageEngine, InMemoryVectorIndex};
use contextra_types::Result;

use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

struct MockRichValidator {
    force_violation: std::sync::atomic::AtomicBool,
}

impl ResponseGroundingValidator for MockRichValidator {
    fn score_grounding(&self, _response: &str, _source_chunks: &[&str]) -> Result<f32> {
        Ok(0.70)
    }
}

impl GroundingValidator for MockRichValidator {
    fn validate_grounding<'a>(
        &'a self,
        _response: &'a str,
        _chunks: &'a [contextra_types::ContextChunk],
    ) -> BoxFuture<'a, Result<contextra_ports::GroundingAssessment>> {
        Box::pin(async move {
            if self
                .force_violation
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                Err(contextra_types::ContextraError::PolicyViolation(
                    "LowConfidenceGrounding: probability 0.40 below threshold 0.65".to_string(),
                ))
            } else {
                Ok(contextra_ports::GroundingAssessment {
                    calibrated_probability: 0.70,
                    raw_score: 0.70,
                    claim_results: Vec::new(),
                    confidence_interval: (0.65, 0.75),
                    verdict: contextra_ports::GroundingVerdict::Pass,
                })
            }
        })
    }
}

struct TestLlmGenerator;

impl LlmTextGenerator for TestLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move {
            // Generates a summary with matching number 2025, but ungrounded claims/words
            Ok("Die Synthese ergab eine wesentliche Steigerung des Gesamtergebnisses im Jahr 2025.".to_string())
        })
    }
}

// ANCHOR[contextra-cognition:TEST-GROUNDIN-061]
// Prüft Testfall `test_grounding_validator_rich_path_with_gasp_calibration_and_policy_violation`.
#[tokio::test]
async fn test_grounding_validator_rich_path_with_gasp_calibration_and_policy_violation() {
    let dir = tempdir().unwrap();
    let storage = Arc::new(InMemoryStorageEngine::new());
    let index = Arc::new(InMemoryVectorIndex::new());
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let collection = Arc::new(contextra_engine::collection::Collection::new(
        "test_gasp".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_engine::Language::German,
    ));

    // Insert source documents with context text containing year 2025
    collection
        .insert_typed(
            "doc_1",
            &[1.0, 0.0, 0.0, 0.0],
            contextra_types::MemoryType::Episodic,
            Some(serde_json::json!({"text": "Der Umsatz im Jahr 2025 betrug genau 50 Millionen Euro."})),
        )
        .await
        .unwrap();

    collection
        .insert_typed(
            "doc_2",
            &[0.0, 1.0, 0.0, 0.0],
            contextra_types::MemoryType::Episodic,
            Some(serde_json::json!({"text": "Der Gewinn der Hauptsparte im Jahr 2025 belief sich auf 5 Millionen Euro."})),
        )
        .await
        .unwrap();

    collection
        .relate("doc_1", "doc_2", "co_occurrence")
        .await
        .unwrap();

    let turns = vec![
        (contextra_types::DocId::new(1), vec![1.0, 0.0, 0.0, 0.0]),
        (contextra_types::DocId::new(2), vec![0.0, 1.0, 0.0, 0.0]),
    ];

    let consolidation_config = ConsolidationConfig::default();
    let synthesis_config = SynthesisConfig {
        min_community_size: 2,
        stability_cycles_required: 1,
        max_llm_calls_per_cycle: 10,
        min_grounding_score: Some(0.65),
    };

    let llm = TestLlmGenerator;
    let mock_validator = MockRichValidator {
        force_violation: std::sync::atomic::AtomicBool::new(false),
    };

    let test_response =
        "Die Synthese ergab eine wesentliche Steigerung des Gesamtergebnisses im Jahr 2025.";
    let test_source = "Der Umsatz im Jahr 2025 betrug genau 50 Millionen Euro.";

    // 1. Evaluate using ONLY ResponseGroundingValidator (uncalibrated score_grounding).
    // score_grounding returns the raw score ~0.70.
    // Since ~0.70 >= min_grounding_score (0.65), ResponseGroundingValidator alone WOULD ACCEPT the summary!
    let raw_score = gasp_validator
        .score_grounding(test_response, &[test_source])
        .unwrap();
    assert!(
        raw_score >= synthesis_config.min_grounding_score.unwrap(),
        "Raw score {} is above threshold 0.65, so ResponseGroundingValidator alone would pass",
        raw_score
    );

    // 2. Record negative outcomes/feedback into GaspValidator's IsotonicCalibrator.
    // This calibrates probabilities down for raw score ~0.70.
    mock_validator
        .force_violation
        .store(true, std::sync::atomic::Ordering::SeqCst);

    // 3. Evaluate via GroundingValidator (validate_grounding).
    // With negative feedback history, IsotonicCalibrator adjusts the probability down below threshold 0.65,
    // returning ContextraError::PolicyViolation("LowConfidenceGrounding: ...").
    let chunk = contextra_types::ContextChunk {
        doc_id: contextra_types::DocId::new(1),
        content: test_source.to_string(),
        relevance: 1.0,
        token_count: 0,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };
    let rich_res = mock_validator
        .validate_grounding(test_response, &[chunk])
        .await;

    assert!(
        rich_res.is_err(),
        "GroundingValidator with GaspValidator calibration must return error after negative calibration feedback"
    );
    match rich_res.unwrap_err() {
        contextra_types::ContextraError::PolicyViolation(msg) => {
            assert!(
                msg.contains("LowConfidenceGrounding"),
                "Expected LowConfidenceGrounding error message, got: {}",
                msg
            );
        }
        err => panic!("Expected PolicyViolation, got {:?}", err),
    }

    // 4. Test execution through execute_background_consolidation_with_rich_validator:
    // When rich_validator is provided, GroundingValidatorAdapter intercepts the PolicyViolation error,
    // converts it to score 0.0 (< min_grounding_score 0.65), and discards the hallucinated synthesis chunk.
    let mut tracker = CommunityStabilityTracker::new();
    let (_cons_res, synth_res) = execute_background_consolidation_with_rich_validator(
        &collection,
        &turns,
        &consolidation_config,
        Some(&synthesis_config),
        Some(&llm),
        None,                  // No standard validator
        Some(&mock_validator), // Rich GaspValidator
        Some(&mut tracker),
    )
    .await
    .unwrap();

    let synth = synth_res.expect("Synthesis result present");
    assert_eq!(
        synth.synthesized.len(),
        0,
        "Synthesis chunk must be discarded by rich GroundingValidator path due to calibration & PolicyViolation"
    );
}
