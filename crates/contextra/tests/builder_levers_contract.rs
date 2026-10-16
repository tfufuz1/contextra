// FILE-CONTEXT
// ZWECK: Vertragstests für alle Builder- und QueryRewriter-Hebel (Ring 4 / Facade).
// INVARIANTEN: Zero panic; Hebel-Werte erreichen Konfiguration; deterministisches Überschreiben; INV-PERF-PROFILE-1.

use std::sync::Arc;
use std::time::Duration;

use contextra::performance_profile::PerformanceProfile;
use contextra::query_rewriter::LlmQueryRewriter;
use contextra::ContextraBuilder;
use contextra_core::error::ContextraError;
use contextra_core::DistanceMetric;
use contextra_db::{ContextraConfig, EmbeddingBackend};
use contextra_ports::license::{FeatureRing, LicenseGate, OpenFastGate};
use contextra_ports::{BoxFuture, LlmTextGenerator};

struct DummyLlmGenerator;
impl LlmTextGenerator for DummyLlmGenerator {
    fn generate<'a>(
        &'a self,
        _prompt: &'a str,
    ) -> BoxFuture<'a, contextra_core::error::Result<String>> {
        Box::pin(async { Ok("subquery 1\nsubquery 2".to_string()) })
    }
}

struct CustomLicenseGate;
impl LicenseGate for CustomLicenseGate {
    fn check_ring(&self, _ring: FeatureRing) -> Result<(), contextra_ports::license::LicenseError> {
        Ok(())
    }
}

#[test]
fn test_builder_max_elements_lever() {
    let builder = ContextraBuilder::new(128).with_max_elements(100);
    assert_eq!(builder.config().max_elements, 100);

    let builder = builder.with_max_elements(200);
    assert_eq!(builder.config().max_elements, 200);
}

#[test]
fn test_builder_distance_metric_lever() {
    let builder = ContextraBuilder::new(128).with_distance_metric(DistanceMetric::DotProduct);
    assert_eq!(builder.config().distance_metric, DistanceMetric::DotProduct);

    let builder = builder.with_distance_metric(DistanceMetric::Euclidean);
    assert_eq!(builder.config().distance_metric, DistanceMetric::Euclidean);
}

#[test]
fn test_builder_encryption_passphrase_lever() {
    let builder = ContextraBuilder::new(128).with_encryption_passphrase("pass1");
    assert_eq!(
        builder.config().encryption_passphrase,
        Some("pass1".to_string())
    );

    let builder = builder.with_encryption_passphrase("pass2");
    assert_eq!(
        builder.config().encryption_passphrase,
        Some("pass2".to_string())
    );
}

#[test]
fn test_builder_embedding_backend_lever() {
    let backend1 = EmbeddingBackend::Onnx {
        model_name: "model_a".to_string(),
        cache_dir: None,
    };
    let builder = ContextraBuilder::new(128).with_embedding_backend(backend1.clone());
    assert_eq!(builder.config().embedding_backend, backend1);

    let backend2 = EmbeddingBackend::Onnx {
        model_name: "model_b".to_string(),
        cache_dir: None,
    };
    let builder = builder.with_embedding_backend(backend2.clone());
    assert_eq!(builder.config().embedding_backend, backend2);
}

#[test]
fn test_builder_consolidation_lever() {
    let builder = ContextraBuilder::new(128).with_consolidation(true, Duration::from_secs(60));
    assert!(builder.config().consolidation_enabled);
    assert_eq!(
        builder.config().consolidation_interval,
        Duration::from_secs(60)
    );

    let builder = builder.with_consolidation(false, Duration::from_secs(120));
    assert!(!builder.config().consolidation_enabled);
    assert_eq!(
        builder.config().consolidation_interval,
        Duration::from_secs(120)
    );
}

#[test]
fn test_builder_license_gate_lever() {
    let gate1: Arc<dyn LicenseGate> = Arc::new(OpenFastGate);
    let builder = ContextraBuilder::new(128).with_license_gate(gate1);

    let gate2: Arc<dyn LicenseGate> = Arc::new(CustomLicenseGate);
    let builder = builder.with_license_gate(gate2);
    assert_eq!(
        builder.license_gate().check_ring(FeatureRing::Sovereign),
        Ok(())
    );
}

#[test]
fn test_builder_signed_license_lever() {
    let dummy_payload = b"invalid payload";
    let dummy_sig = [0u8; 64];
    let dummy_key = [0u8; 32];

    let builder =
        ContextraBuilder::new(128).with_signed_license(dummy_payload, &dummy_sig, &dummy_key);
    // Invalid signature should record error on builder
    let gate2: Arc<dyn LicenseGate> = Arc::new(OpenFastGate);
    let builder = builder.with_license_gate(gate2);
    // Overwritten gate resets signed_license_error
    assert_eq!(builder.license_gate().check_ring(FeatureRing::Fast), Ok(()));
}

#[test]
fn test_builder_performance_profile_lever() {
    let builder =
        ContextraBuilder::new(128).with_performance_profile(PerformanceProfile::Compliance);
    assert_eq!(
        builder.performance_profile(),
        Some(PerformanceProfile::Compliance)
    );

    let builder = builder.with_performance_profile(PerformanceProfile::BareMetal);
    assert_eq!(
        builder.performance_profile(),
        Some(PerformanceProfile::BareMetal)
    );
}

#[test]
fn test_rewriter_max_subqueries_lever() {
    let gen = Arc::new(DummyLlmGenerator);
    let rewriter = LlmQueryRewriter::new(gen).with_max_subqueries(5);
    assert_eq!(rewriter.max_subqueries(), 5);

    let rewriter = rewriter.with_max_subqueries(10);
    assert_eq!(rewriter.max_subqueries(), 10);
}

#[test]
fn test_rewriter_max_context_results_lever() {
    let gen = Arc::new(DummyLlmGenerator);
    let rewriter = LlmQueryRewriter::new(gen).with_max_context_results(4);
    assert_eq!(rewriter.max_context_results(), 4);

    let rewriter = rewriter.with_max_context_results(8);
    assert_eq!(rewriter.max_context_results(), 8);
}

#[test]
fn test_rewriter_max_snippet_chars_lever() {
    let gen = Arc::new(DummyLlmGenerator);
    let rewriter = LlmQueryRewriter::new(gen).with_max_snippet_chars(500);
    assert_eq!(rewriter.max_snippet_chars(), 500);

    let rewriter = rewriter.with_max_snippet_chars(1000);
    assert_eq!(rewriter.max_snippet_chars(), 1000);
}

#[tokio::test]
async fn test_performance_profile_deletion_proof_conflict_returns_error() {
    let config = ContextraConfig {
        deletion_proof_active: true,
        ..Default::default()
    };

    // BareMetal profile has deletion_proof_active = false
    let res = ContextraBuilder::new(128)
        .with_config(config.clone())
        .with_performance_profile(PerformanceProfile::BareMetal)
        .build()
        .await;

    if let Err(ContextraError::PolicyViolation(msg)) = res {
        assert!(
            msg.contains("Explicit ContextraConfig conflicts with PerformanceProfile settings"),
            "Unexpected error message: {msg}"
        );
    } else {
        assert!(
            matches!(res, Err(ContextraError::PolicyViolation(_))),
            "Expected PolicyViolation error"
        );
    }

    // Balanced profile also has deletion_proof_active = false
    let res = ContextraBuilder::new(128)
        .with_config(config)
        .with_performance_profile(PerformanceProfile::Balanced)
        .build()
        .await;

    if let Err(ContextraError::PolicyViolation(msg)) = res {
        assert!(
            msg.contains("Explicit ContextraConfig conflicts with PerformanceProfile settings"),
            "Unexpected error message: {msg}"
        );
    } else {
        assert!(
            matches!(res, Err(ContextraError::PolicyViolation(_))),
            "Expected PolicyViolation error"
        );
    }
}

#[test]
fn test_builder_encryption_passphrase_debug_redacted() {
    let secret = "super_secret_cleartext_passphrase_99";
    let builder = ContextraBuilder::new(128).with_encryption_passphrase(secret);
    let debug_output = format!("{builder:?}");

    assert!(
        !debug_output.contains(secret),
        "Cleartext passphrase leaked in Debug output!"
    );
    assert!(
        debug_output.contains("***"),
        "Redacted passphrase marker '***' missing in Debug output!"
    );
}
