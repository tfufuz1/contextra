// ZWECK: Integrationstest für den Konsolidierungs-Worker und die Validator-Konstruktoren (`start_worker`, `with_validator`, `with_rich_validator`).
// INVARIANTEN: P10 / Wiring Governance — Echte Ausführung des Konsolidierungs-Workers und Registrierung beider Validator-Typen.

use std::sync::Arc;
use std::time::Duration;

use contextra_cognition::consolidation_executor::ConsolidationEngine;
use contextra_cognition::{ConsolidationConfig, SynthesisConfig};
use contextra_engine::Collection;
use contextra_ports::{
    GroundingCalibrator, GroundingValidator, GroundingValidatorResult, GroundingValidatorSpec,
    ResponseGroundingValidator,
};
use contextra_store::LsmStorage;
use contextra_types::DocId;
use contextra_vector::hnsw::HnswIndex;
use tempfile::TempDir;

struct DummyResponseValidator;
impl ResponseGroundingValidator for DummyResponseValidator {
    fn validate_grounding(
        &self,
        _response: &str,
        _source_facts: &[String],
    ) -> contextra_types::Result<f32> {
        Ok(1.0)
    }
}

struct DummyRichValidator;
impl GroundingValidator for DummyRichValidator {
    fn spec(&self) -> GroundingValidatorSpec {
        GroundingValidatorSpec {
            name: "dummy_rich".into(),
            version: "1.0.0".into(),
            calibrator: GroundingCalibrator::Linear,
        }
    }

    fn validate(
        &self,
        _claim: &str,
        _premises: &[String],
    ) -> contextra_types::Result<GroundingValidatorResult> {
        Ok(GroundingValidatorResult {
            score: 1.0,
            is_policy_violation: false,
        })
    }
}

fn setup_test_collection() -> (Arc<Collection<LsmStorage, HnswIndex>>, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let storage = Arc::new(LsmStorage::open(dir.path()).expect("open storage"));
    let vector_index = Arc::new(HnswIndex::new(384, 16, 200));
    let collection = Arc::new(
        Collection::new("test_worker_wiring", storage, vector_index).expect("new collection"),
    );
    (collection, dir)
}

#[tokio::test]
async fn test_consolidation_engine_builder_and_start_worker() {
    let (collection, _dir) = setup_test_collection();
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let dummy_val: Arc<dyn ResponseGroundingValidator> = Arc::new(DummyResponseValidator);
    let dummy_rich_val: Arc<dyn GroundingValidator> = Arc::new(DummyRichValidator);

    // Verify builder methods on ConsolidationEngine
    let engine = ConsolidationEngine::new(
        collection.clone(),
        ConsolidationConfig::default(),
        SynthesisConfig::default(),
        Duration::from_millis(100),
        cancel_token.clone(),
    )
    .with_validator(dummy_val)
    .with_rich_validator(dummy_rich_val);

    assert!(engine.validator.is_some());
    assert!(engine.rich_validator.is_some());

    // Verify start_worker launcher
    let handle = ConsolidationEngine::start_worker(
        collection,
        ConsolidationConfig::default(),
        SynthesisConfig::default(),
        Duration::from_millis(50),
        cancel_token.clone(),
    );

    cancel_token.cancel();
    let res = handle.await;
    assert!(res.is_ok());
}
