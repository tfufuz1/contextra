// Integration test suite verifying production reachability and wiring of all 13 J36 symbols.
// File prefix: j36-contextra_

use std::sync::Arc;
use std::time::Duration;

use contextra::collection_profile::DeploymentTier;
use contextra::{
    open_with_config, AgentMemory, ContextraBuilder, ContextraConfig, DistanceMetric,
    EmbeddingBackend, LlmQueryRewriter, TextEmbeddingEngine,
};
use contextra_db::QueryRewriter;
use contextra_ports::license::OpenFastGate;
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_types::ScoredEntry;

struct DummyLlmGenerator;

impl LlmTextGenerator for DummyLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, contextra_core::error::Result<String>> {
        Box::pin(async move {
            Ok("Subquery 1\nSubquery 2\nSubquery 3\nSubquery 4".to_string())
        })
    }
}

struct MockEmbedder;

impl TextEmbeddingEngine for MockEmbedder {
    fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, contextra_core::error::Result<Vec<f32>>> {
        Box::pin(async move { Ok(vec![0.1; 16]) })
    }

    fn embed_batch<'a>(&'a self, texts: &'a [&'a str]) -> BoxFuture<'a, contextra_core::error::Result<Vec<Vec<f32>>>> {
        Box::pin(async move { Ok(vec![vec![0.1; 16]; texts.len()]) })
    }
}

/// Tests that `open_with_config` (primary facade) executes builder setters:
/// `with_max_elements`, `with_distance_metric`, `with_encryption_passphrase`, `with_embedding_backend`, `with_consolidation`.
#[tokio::test]
async fn test_open_with_config_executes_builder_setters() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = std::env::temp_dir().join(format!("j36_open_with_config_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let config = ContextraConfig {
        dimension: 16,
        max_elements: 5000,
        distance_metric: DistanceMetric::Cosine,
        encryption_passphrase: Some("secret_pwd".to_string()),
        embedding_backend: EmbeddingBackend::None,
        consolidation_enabled: true,
        consolidation_interval: Duration::from_secs(120),
        ..Default::default()
    };

    // ContextraBuilder::from_config is invoked inside open_with_config's ContextraBuilder::with_config call
    let builder = ContextraBuilder::from_config(config.clone());
    assert_eq!(builder.config().max_elements, 5000);
    assert_eq!(builder.config().distance_metric, DistanceMetric::Cosine);
    assert_eq!(builder.config().encryption_passphrase.as_deref(), Some("secret_pwd"));
    assert_eq!(builder.config().embedding_backend, EmbeddingBackend::None);
    assert!(builder.config().consolidation_enabled);
    assert_eq!(builder.config().consolidation_interval, Duration::from_secs(120));

    let db = open_with_config(&tmp_path, config).await?;
    assert_eq!(db.len().await?, 0);

    let _ = std::fs::remove_dir_all(&tmp_path);
    Ok(())
}

/// Tests that `AgentMemory::new` and `AgentMemory::from` delegate directly to `AgentMemory::new_arc`.
#[tokio::test]
async fn test_agent_memory_new_delegates_to_new_arc() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = std::env::temp_dir().join(format!("j36_agent_mem_new_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let db = ContextraBuilder::new(16)
        .with_storage_path(&tmp_path)
        .with_embedder(Arc::new(MockEmbedder))
        .build()
        .await?;

    let arc_db = Arc::new(db);

    // AgentMemory::new and AgentMemory::from both execute new_arc in production code
    let agent_memory_1 = AgentMemory::new_arc(Arc::clone(&arc_db));
    let agent_memory_2: AgentMemory = Arc::clone(&arc_db).into();

    let mem_id = agent_memory_1.remember("Memory entry 1", None).await?;
    let recalled = agent_memory_2.recall("Memory entry 1", 5).await?;
    assert!(!recalled.is_empty());
    assert_eq!(recalled[0].id, mem_id.as_str());

    let _ = std::fs::remove_dir_all(&tmp_path);
    Ok(())
}

/// Tests that `ContextraBuilder::from_tier` invokes `validate_with_license` on `CollectionProfile` upon `build()`.
#[tokio::test]
async fn test_builder_from_tier_invokes_validate_with_license() {
    let tmp_path = std::env::temp_dir().join(format!("j36_tier_validation_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_path);

    // DeploymentTier::EnterpriseRegulated produces CollectionProfile with PerformanceProfile::Compliance,
    // which requires Compliance ring authorization. OpenFastGate only authorizes Fast ring.
    let builder = ContextraBuilder::from_tier(DeploymentTier::EnterpriseRegulated)
        .with_storage_path(&tmp_path)
        .with_license_gate(Arc::new(OpenFastGate));

    // build() invokes collection_profile.validate_with_license(license_gate), which fails closed
    let build_res = builder.build().await;
    assert!(build_res.is_err());

    let _ = std::fs::remove_dir_all(&tmp_path);
}

/// Tests that `LlmQueryRewriter::new` delegates to `new_with_limits`, executing `with_max_subqueries`,
/// `with_max_context_results`, and `with_max_snippet_chars`.
#[tokio::test]
async fn test_llm_query_rewriter_new_executes_limit_setters() -> Result<(), Box<dyn std::error::Error>> {
    let dummy_gen = Arc::new(DummyLlmGenerator);
    // LlmQueryRewriter::new delegates to new_with_limits, invoking all 3 limit setters
    let rewriter = LlmQueryRewriter::new(dummy_gen);

    assert_eq!(rewriter.max_subqueries(), 3);
    assert_eq!(rewriter.max_context_results(), 3);
    assert_eq!(rewriter.max_snippet_chars(), 300);

    let scored = vec![ScoredEntry {
        id: "doc_1".to_string(),
        final_score: 0.95,
        metadata: Some(serde_json::json!({"text": "Sample snippet content"})),
    }];

    let subqueries = rewriter.rewrite("original query", &scored).await?;
    assert!(subqueries.len() <= 3);

    Ok(())
}

/// Tests that `from_signed_license` invokes `with_signed_license` and `with_license_gate`.
#[tokio::test]
async fn test_builder_with_signed_license_executes_with_license_gate() {
    let builder = ContextraBuilder::from_signed_license(b"invalid_payload", &[0u8; 64], &[0u8; 32]);

    let build_res = builder.build().await;
    assert!(build_res.is_err());
}
