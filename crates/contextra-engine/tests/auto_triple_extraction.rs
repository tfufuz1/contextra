use contextra_engine::collection::crud::AutoExtractionConfig;
use contextra_engine::{Contextra, ContextraConfig};
use contextra_ports::{BoxFuture, LlmTextGenerator, TextEmbeddingEngine};
use contextra_types::{ContextraError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

struct FakeEmbedder {
    dim: usize,
}

impl TextEmbeddingEngine for FakeEmbedder {
    fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>> {
        let dim = self.dim;
        Box::pin(async move { Ok(vec![0.1; dim]) })
    }
}

struct MockLlmGenerator {
    response: std::result::Result<String, String>,
    call_count: AtomicUsize,
}

impl MockLlmGenerator {
    fn ok(response: impl Into<String>) -> Self {
        Self {
            response: Ok(response.into()),
            call_count: AtomicUsize::new(0),
        }
    }

    fn err(msg: impl Into<String>) -> Self {
        Self {
            response: Err(msg.into()),
            call_count: AtomicUsize::new(0),
        }
    }
}

impl LlmTextGenerator for MockLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        match &self.response {
            Ok(resp) => {
                let r = resp.clone();
                Box::pin(async move { Ok(r) })
            }
            Err(e) => {
                let err_msg = e.clone();
                Box::pin(async move { Err(ContextraError::Internal(err_msg)) })
            }
        }
    }
}

#[tokio::test]
async fn test_auto_triple_extraction_default_config_creates_graph_edges() -> Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        ..Default::default()
    };
    let db = Contextra::open_with_config(tmp.path(), config).await?;

    let mock_json = r#"[
        {"subject": "Berlin", "predicate": "capital_of", "object": "Germany", "confidence": 0.95, "source_span": null}
    ]"#;
    let mock_llm = Arc::new(MockLlmGenerator::ok(mock_json));

    let auto_cfg = AutoExtractionConfig::default();

    let col = db.collection("auto_triple_col").await?;
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 })).await?;
    col.set_auto_extraction(mock_llm.clone(), auto_cfg);

    // 1. Insert document text
    col.insert_text_only("doc_berlin_1", "Berlin is the capital city of Germany.", None)
        .await?;

    #[cfg(feature = "entity-extraction")]
    {
        if cfg!(not(feature = "auto-extraction-opt-out")) {
            assert_eq!(
                mock_llm.call_count.load(Ordering::SeqCst),
                1,
                "AutoExtractionConfig::default() should execute LLM generation once"
            );

            let berlin_id = contextra_types::EntityId::from_key("Berlin")?;
            let germany_id = contextra_types::EntityId::from_key("Germany")?;
            let neighbors = col.graph_index().neighbors(berlin_id).await?;

            assert!(
                neighbors.contains(&germany_id),
                "Graph index must contain auto-extracted relationship edge between Berlin and Germany"
            );
        }
    }

    // 2. Error path test: LLM service timeout / error must NOT fail document insertion (best-effort resilience)
    let failing_llm = Arc::new(MockLlmGenerator::err("LLM service timeout"));
    let col2 = db.collection("auto_triple_fail_col").await?;
    col2.set_embedder(Arc::new(FakeEmbedder { dim: 4 })).await?;
    col2.set_auto_extraction(failing_llm.clone(), AutoExtractionConfig::default());

    let res = col2
        .insert_text_only("doc_resilient_1", "Resilient document text.", None)
        .await;

    assert!(
        res.is_ok(),
        "Document insertion must succeed even if auto-extraction LLM service fails"
    );

    let doc = col2.get("doc_resilient_1").await?;
    assert!(
        doc.is_some(),
        "Inserted document must exist in storage despite LLM extraction failure"
    );

    db.close().await?;
    Ok(())
}
