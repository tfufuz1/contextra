// FILE-CONTEXT
// ZWECK: Integrationstests fuer die Verdrahtung von AutoExtraction in Collection::insert_text_only.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::collection::crud::{AutoExtractionConfig, EntityExtractionConfig};
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
async fn test_auto_extraction_disabled_no_llm_calls() {
    let tmp = TempDir::new().expect("temp dir");
    let config = contextra_engine::ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        ..Default::default()
    };
    let db = contextra_engine::Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");

    let mock_llm = Arc::new(MockLlmGenerator::ok("[]"));
    let auto_cfg = AutoExtractionConfig {
        enabled: false,
        entity_config: EntityExtractionConfig::default(),
    };

    let col = db.collection("default").await.expect("collection");
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 }))
        .await
        .expect("set_embedder");
    col.set_auto_extraction(mock_llm.clone(), auto_cfg);

    col.insert_text_only("doc-1", "Alice works at Acme Inc.", None)
        .await
        .expect("insert_text_only should succeed");

    assert_eq!(
        mock_llm.call_count.load(Ordering::SeqCst),
        0,
        "When auto extraction is disabled, 0 LLM calls must occur"
    );

    let doc = col.get("doc-1").await.expect("get doc");
    assert!(doc.is_some(), "Document must exist");
}

#[tokio::test]
async fn test_auto_extraction_enabled_creates_edges() {
    let tmp = TempDir::new().expect("temp dir");
    let config = contextra_engine::ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        ..Default::default()
    };
    let db = contextra_engine::Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");

    let mock_json = r#"[
        {"subject": "Alice", "predicate": "works_at", "object": "Acme", "confidence": 0.9, "source_span": null}
    ]"#;
    let mock_llm = Arc::new(MockLlmGenerator::ok(mock_json));
    let auto_cfg = AutoExtractionConfig {
        enabled: true,
        entity_config: EntityExtractionConfig {
            enabled: true,
            max_llm_calls_per_cycle: 10,
            min_confidence: 0.5,
        },
    };

    let col = db.collection("default").await.expect("collection");
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 }))
        .await
        .expect("set_embedder");
    col.set_auto_extraction(mock_llm.clone(), auto_cfg);

    col.insert_text_only("doc-1", "Alice works at Acme.", None)
        .await
        .expect("insert_text_only should succeed");

    #[cfg(feature = "entity-extraction")]
    {
        assert_eq!(
            mock_llm.call_count.load(Ordering::SeqCst),
            1,
            "When auto extraction is enabled, LLM should be called once"
        );

        // Verify created relation in graph index
        let alice_id = contextra_types::EntityId::from_key("Alice").expect("Alice entity id");
        let acme_id = contextra_types::EntityId::from_key("Acme").expect("Acme entity id");
        let neighbors = col
            .graph_index()
            .neighbors(alice_id)
            .await
            .expect("neighbors");
        assert!(
            neighbors.contains(&acme_id),
            "Graph should contain extracted edge from Alice to Acme"
        );
    }
}

#[tokio::test]
async fn test_auto_extraction_low_confidence_ignored() {
    let tmp = TempDir::new().expect("temp dir");
    let config = contextra_engine::ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        ..Default::default()
    };
    let db = contextra_engine::Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");

    let mock_json = r#"[
        {"subject": "Bob", "predicate": "likes", "object": "Pizza", "confidence": 0.3, "source_span": null}
    ]"#;
    let mock_llm = Arc::new(MockLlmGenerator::ok(mock_json));
    let auto_cfg = AutoExtractionConfig {
        enabled: true,
        entity_config: EntityExtractionConfig {
            enabled: true,
            max_llm_calls_per_cycle: 10,
            min_confidence: 0.5,
        },
    };

    let col = db.collection("default").await.expect("collection");
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 }))
        .await
        .expect("set_embedder");
    col.set_auto_extraction(mock_llm.clone(), auto_cfg);

    col.insert_text_only("doc-2", "Bob likes Pizza.", None)
        .await
        .expect("insert_text_only should succeed");

    #[cfg(feature = "entity-extraction")]
    {
        assert_eq!(
            mock_llm.call_count.load(Ordering::SeqCst),
            1,
            "LLM generator was called"
        );

        let bob_id = contextra_types::EntityId::from_key("Bob").expect("Bob entity id");
        let neighbors = col
            .graph_index()
            .neighbors(bob_id)
            .await
            .expect("neighbors");
        assert!(
            neighbors.is_empty(),
            "No edges should be created for low-confidence triples"
        );
    }
}

#[tokio::test]
async fn test_auto_extraction_llm_error_best_effort() {
    let tmp = TempDir::new().expect("temp dir");
    let config = contextra_engine::ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        ..Default::default()
    };
    let db = contextra_engine::Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");

    let mock_llm = Arc::new(MockLlmGenerator::err("Simulated LLM service timeout"));
    let auto_cfg = AutoExtractionConfig {
        enabled: true,
        entity_config: EntityExtractionConfig::default(),
    };

    let col = db.collection("default").await.expect("collection");
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 }))
        .await
        .expect("set_embedder");
    col.set_auto_extraction(mock_llm.clone(), auto_cfg);

    let res = col
        .insert_text_only("doc-3", "Charlie studies computer science.", None)
        .await;

    assert!(
        res.is_ok(),
        "Document insertion must succeed even if LLM extraction fails"
    );

    let doc = col.get("doc-3").await.expect("get doc");
    assert!(doc.is_some(), "Document must be stored");
}

#[tokio::test]
async fn test_auto_extraction_default_config_creates_edges_in_graph_index() {
    let tmp = TempDir::new().expect("temp dir");
    let config = contextra_engine::ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        ..Default::default()
    };
    let db = contextra_engine::Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");

    let mock_json = r#"[
        {"subject": "Carol", "predicate": "manages", "object": "ProjectX", "confidence": 0.95, "source_span": null}
    ]"#;
    let mock_llm = Arc::new(MockLlmGenerator::ok(mock_json));

    // Uses AutoExtractionConfig::default() which defaults to enabled: true (unless opt-out feature set)
    let auto_cfg = AutoExtractionConfig::default();

    let col = db.collection("default").await.expect("collection");
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 }))
        .await
        .expect("set_embedder");
    col.set_auto_extraction(mock_llm.clone(), auto_cfg);

    col.insert_text_only("doc-default-cfg", "Carol manages ProjectX.", None)
        .await
        .expect("insert_text_only should succeed");

    #[cfg(feature = "entity-extraction")]
    {
        if cfg!(not(feature = "auto-extraction-opt-out")) {
            assert_eq!(
                mock_llm.call_count.load(Ordering::SeqCst),
                1,
                "AutoExtractionConfig::default() should trigger LLM call"
            );

            let carol_id = contextra_types::EntityId::from_key("Carol").expect("Carol entity id");
            let proj_id = contextra_types::EntityId::from_key("ProjectX").expect("ProjectX entity id");
            let neighbors = col
                .graph_index()
                .neighbors(carol_id)
                .await
                .expect("neighbors");
            assert!(
                neighbors.contains(&proj_id),
                "Graph index must contain auto-extracted relationship between Carol and ProjectX"
            );
        } else {
            assert_eq!(
                mock_llm.call_count.load(Ordering::SeqCst),
                0,
                "When opt-out feature is active, default config must not trigger LLM"
            );
        }
    }
}
