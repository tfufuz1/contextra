// FILE-CONTEXT
// ZWECK: Integrationstests fuer AutoExtractionMode Laufzeit-Verhalten und Profil-Wechsel.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::collection::crud::{
    AutoExtractionConfig, AutoExtractionMode, EntityExtractionConfig,
};
use contextra_ports::{BoxFuture, LlmTextGenerator, TextEmbeddingEngine};
#[cfg(feature = "entity-extraction")]
use contextra_types::{DocId, EntityId};
use contextra_types::Result;
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
    response: String,
    call_count: AtomicUsize,
}

impl MockLlmGenerator {
    fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            call_count: AtomicUsize::new(0),
        }
    }
}

impl LlmTextGenerator for MockLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }
}

/// Test 1: Disabled schreibt keine Triples/Entities (Provenance leer)
#[tokio::test]
async fn test_auto_extraction_disabled_mode_creates_no_triples_and_provenance_empty() {
    let tmp = TempDir::new().expect("temp dir");
    let db = contextra_engine::Contextra::open_with_config(
        tmp.path(),
        contextra_engine::ContextraConfig {
            dimension: 4,
            max_elements: 10_000,
            ..Default::default()
        },
    )
    .await
    .expect("open db");

    let mock_json = r#"[
        {"subject": "Alice", "predicate": "knows", "object": "Bob", "confidence": 0.95, "source_span": null}
    ]"#;
    let mock_llm = Arc::new(MockLlmGenerator::new(mock_json));

    let disabled_cfg = AutoExtractionConfig {
        mode: AutoExtractionMode::Disabled,
        enabled: true, // mode: Disabled overrides enabled: true
        entity_config: EntityExtractionConfig::default(),
    };

    let col = db.collection("default").await.expect("collection");
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 }))
        .await
        .expect("set_embedder");
    col.set_auto_extraction(mock_llm.clone(), disabled_cfg);

    col.insert_text_only("doc-disabled", "Alice knows Bob.", None)
        .await
        .expect("insert_text_only");

    assert_eq!(
        mock_llm.call_count.load(Ordering::SeqCst),
        0,
        "Disabled AutoExtractionMode must perform 0 LLM calls"
    );

    #[cfg(feature = "entity-extraction")]
    {
        let alice_id = EntityId::from_key("Alice").expect("Alice entity id");
        let neighbors = col.graph_index().neighbors(alice_id).await.expect("neighbors");
        assert!(
            neighbors.is_empty(),
            "No graph edges or triples should exist when auto extraction is disabled"
        );

        let bob_id = EntityId::from_key("Bob").expect("Bob entity id");
        let edge_doc = col.graph_index().source_doc_id_at(alice_id, bob_id);
        assert_eq!(
            edge_doc, None,
            "Provenance must be empty for disabled auto extraction"
        );
    }
}

/// Test 2: Enabled mode extracts triples and sets provenance
#[tokio::test]
async fn test_auto_extraction_enabled_mode_creates_triples() {
    let tmp = TempDir::new().expect("temp dir");
    let db = contextra_engine::Contextra::open_with_config(
        tmp.path(),
        contextra_engine::ContextraConfig {
            dimension: 4,
            max_elements: 10_000,
            ..Default::default()
        },
    )
    .await
    .expect("open db");

    let mock_json = r#"[
        {"subject": "Carol", "predicate": "leads", "object": "TeamX", "confidence": 0.95, "source_span": null}
    ]"#;
    let mock_llm = Arc::new(MockLlmGenerator::new(mock_json));

    let enabled_cfg = AutoExtractionConfig {
        mode: AutoExtractionMode::Enabled,
        enabled: true,
        entity_config: EntityExtractionConfig::default(),
    };

    let col = db.collection("default").await.expect("collection");
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 }))
        .await
        .expect("set_embedder");
    col.set_auto_extraction(mock_llm.clone(), enabled_cfg);

    let doc_key = "doc-enabled-1";
    col.insert_text_only(doc_key, "Carol leads TeamX.", None)
        .await
        .expect("insert_text_only");

    #[cfg(feature = "entity-extraction")]
    {
        if cfg!(not(feature = "auto-extraction-opt-out")) {
            assert_eq!(
                mock_llm.call_count.load(Ordering::SeqCst),
                1,
                "LLM must be called once when enabled"
            );

            col.graph_index().compact();

            let carol_id = EntityId::from_key("Carol").expect("Carol EntityId");
            let team_id = EntityId::from_key("TeamX").expect("TeamX EntityId");
            let expected_doc_id = DocId::from_key(doc_key).expect("DocId");

            let edge_doc = col.graph_index().source_doc_id_at(carol_id, team_id);
            assert_eq!(
                edge_doc,
                Some(expected_doc_id),
                "Extracted edge must carry document provenance"
            );
        }
    }
}

/// Test 3: Profil-Wechsel wirkt nur auf neue Inserts
#[tokio::test]
async fn test_auto_extraction_profile_switch_affects_only_new_inserts() {
    let tmp = TempDir::new().expect("temp dir");
    let db = contextra_engine::Contextra::open_with_config(
        tmp.path(),
        contextra_engine::ContextraConfig {
            dimension: 4,
            max_elements: 10_000,
            ..Default::default()
        },
    )
    .await
    .expect("open db");

    let mock_json_1 = r#"[
        {"subject": "Eve", "predicate": "knows", "object": "Frank", "confidence": 0.95, "source_span": null}
    ]"#;
    let mock_llm_1 = Arc::new(MockLlmGenerator::new(mock_json_1));

    let col = db.collection("default").await.expect("collection");
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 }))
        .await
        .expect("set_embedder");

    // Phase 1: Enabled Mode
    let enabled_cfg = AutoExtractionConfig {
        mode: AutoExtractionMode::Enabled,
        enabled: true,
        entity_config: EntityExtractionConfig::default(),
    };
    col.set_auto_extraction(mock_llm_1.clone(), enabled_cfg);

    col.insert_text_only("doc-phase1", "Eve knows Frank.", None)
        .await
        .expect("insert phase 1");

    // Phase 2: Switch Profile to Disabled Mode
    let mock_json_2 = r#"[
        {"subject": "Grace", "predicate": "knows", "object": "Heidi", "confidence": 0.95, "source_span": null}
    ]"#;
    let mock_llm_2 = Arc::new(MockLlmGenerator::new(mock_json_2));

    let disabled_cfg = AutoExtractionConfig {
        mode: AutoExtractionMode::Disabled,
        enabled: true,
        entity_config: EntityExtractionConfig::default(),
    };
    col.set_auto_extraction(mock_llm_2.clone(), disabled_cfg);

    col.insert_text_only("doc-phase2", "Grace knows Heidi.", None)
        .await
        .expect("insert phase 2");

    assert_eq!(
        mock_llm_2.call_count.load(Ordering::SeqCst),
        0,
        "Mock LLM 2 should not be called after profile switch to Disabled"
    );

    #[cfg(feature = "entity-extraction")]
    {
        if cfg!(not(feature = "auto-extraction-opt-out")) {
            col.graph_index().compact();

            // Verify Phase 1 triple still exists
            let eve_id = EntityId::from_key("Eve").expect("Eve EntityId");
            let frank_id = EntityId::from_key("Frank").expect("Frank EntityId");
            let phase1_doc = DocId::from_key("doc-phase1").expect("DocId");
            assert_eq!(
                col.graph_index().source_doc_id_at(eve_id, frank_id),
                Some(phase1_doc),
                "Existing triples from Phase 1 must remain intact after profile switch"
            );

            // Verify Phase 2 triple was NOT created
            let grace_id = EntityId::from_key("Grace").expect("Grace EntityId");
            let heidi_id = EntityId::from_key("Heidi").expect("Heidi EntityId");
            assert_eq!(
                col.graph_index().source_doc_id_at(grace_id, heidi_id),
                None,
                "New insert in Phase 2 under Disabled mode must not create new triples"
            );
        }
    }
}
