// FILE-CONTEXT
// ZWECK: Integration tests verifying public API wiring and reachability of all 13 symbols in contextra facade (J36-closure).
// INVARIANTEN: Zero panic in tests; explicit test verification of builder methods, collection profile validation, query rewriter limits, and agent memory arc constructors.

use contextra::agent_memory::AgentMemory;
use contextra::builder::ContextraBuilder;
use contextra::collection_profile::{CollectionProfile, DeploymentTier};
use contextra::performance_profile::PerformanceProfile;
use contextra::query_rewriter::LlmQueryRewriter;
use contextra_core::DistanceMetric;
use contextra_db::EmbeddingBackend;
use contextra_ports::license::OpenFastGate;
use contextra_ports::{BoxFuture, LlmTextGenerator};
use std::sync::Arc;
use std::time::Duration;

struct DummyLlmGenerator;

impl LlmTextGenerator for DummyLlmGenerator {
    fn generate<'a>(
        &'a self,
        _prompt: &'a str,
    ) -> BoxFuture<'a, Result<String, contextra_core::error::ContextraError>> {
        Box::pin(async move { Ok("subquery 1\nsubquery 2".to_string()) })
    }
}

#[tokio::test]
async fn test_agent_memory_new_arc_reachability() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = std::env::temp_dir().join(format!("contextra_j36_mem_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let engine = ContextraBuilder::new(4)
        .with_storage_path(&tmp_path)
        .build()
        .await?;

    let arc_engine = Arc::new(engine);
    let agent_mem = AgentMemory::new_arc(Arc::clone(&arc_engine));

    assert_eq!(agent_mem.engine().len().await?, 0);

    let _ = std::fs::remove_dir_all(&tmp_path);
    Ok(())
}

#[test]
fn test_builder_all_configuration_symbols_reachability() {
    let gate: Arc<dyn contextra_ports::license::LicenseGate> = Arc::new(OpenFastGate);
    let builder = ContextraBuilder::new(128)
        .with_max_elements(10_000)
        .with_distance_metric(DistanceMetric::Cosine)
        .with_encryption_passphrase("passphrase-123")
        .with_embedding_backend(EmbeddingBackend::None)
        .with_consolidation(true, Duration::from_secs(120))
        .with_license_gate(Arc::clone(&gate))
        .with_performance_profile(PerformanceProfile::BareMetal)
        .with_signed_license(b"invalid_payload", &[0u8; 64], &[0u8; 32]);

    assert_eq!(builder.config().dimension, 128);
    assert_eq!(builder.config().max_elements, 10_000);
    assert_eq!(builder.config().distance_metric, DistanceMetric::Cosine);
    assert_eq!(
        builder.config().encryption_passphrase,
        Some("passphrase-123".to_string())
    );
    assert_eq!(builder.config().embedding_backend, EmbeddingBackend::None);
    assert!(builder.config().consolidation_enabled);
    assert_eq!(
        builder.config().consolidation_interval,
        Duration::from_secs(120)
    );
    assert_eq!(
        builder.performance_profile(),
        Some(PerformanceProfile::BareMetal)
    );
}

#[test]
fn test_collection_profile_validate_with_license_reachability() {
    let profile: CollectionProfile = DeploymentTier::EdgeMinimal.resolve();
    let gate = OpenFastGate;

    // Direct invocation of validate_with_license
    let result = profile.validate_with_license(&gate);
    assert!(
        result.is_ok(),
        "EdgeMinimal profile validation with OpenFastGate should succeed"
    );
}

#[test]
fn test_query_rewriter_limits_symbols_reachability() {
    let generator: Arc<dyn LlmTextGenerator> = Arc::new(DummyLlmGenerator);

    let rewriter = LlmQueryRewriter::new(Arc::clone(&generator))
        .with_max_subqueries(5)
        .with_max_context_results(4)
        .with_max_snippet_chars(150);

    assert_eq!(rewriter.max_subqueries(), 5);
    assert_eq!(rewriter.max_context_results(), 4);
    assert_eq!(rewriter.max_snippet_chars(), 150);

    let rewriter_explicit = LlmQueryRewriter::new_with_limits(generator, 10, 8, 500);
    assert_eq!(rewriter_explicit.max_subqueries(), 10);
    assert_eq!(rewriter_explicit.max_context_results(), 8);
    assert_eq!(rewriter_explicit.max_snippet_chars(), 500);
}
