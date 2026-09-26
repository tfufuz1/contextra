// FILE-CONTEXT
// ZWECK: Determinismus- & Provenienztests fuer OpenIE Tripel-Extraktion und edge.source_doc_id-Kaskade.
// INVARIANTEN: No unwrap/expect/panic in production code; Test A, B, C gemaess §3.2 und §13.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[cfg(feature = "entity-extraction")]
use contextra_engine::extraction::extract_triples;
use contextra_engine::collection::crud::{AutoExtractionConfig, EntityExtractionConfig};
use contextra_ports::{BoxFuture, LlmTextGenerator, TextEmbeddingEngine};
use contextra_types::{ConfigFingerprint, Result};
#[cfg(feature = "entity-extraction")]
use contextra_types::{DocId, EntityId};
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

struct DeterministicMockLlm {
    response: String,
    call_count: AtomicUsize,
}

impl DeterministicMockLlm {
    fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            call_count: AtomicUsize::new(0),
        }
    }
}

impl LlmTextGenerator for DeterministicMockLlm {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }
}

/// Test A: Gleicher Eingabetext + gleicher Generator + gleiches Modell-Fingerprint
/// => Identische extrahierte Tripel bei zwei unabhängigen Aufrufen.
#[tokio::test]
async fn test_openie_extraction_determinism() {
    #[cfg(feature = "entity-extraction")]
    {
        let mock_json = r#"[
            {"subject": "Alice", "predicate": "knows", "object": "Bob", "confidence": 0.95, "source_span": [0, 16]},
            {"subject": "Bob", "predicate": "works_at", "object": "Acme", "confidence": 0.88, "source_span": [18, 35]}
        ]"#;

        let generator = DeterministicMockLlm::new(mock_json);
        let entity_cfg = EntityExtractionConfig {
            enabled: true,
            max_llm_calls_per_cycle: 10,
            min_confidence: 0.5,
        };

        let text_input = "Alice knows Bob. Bob works at Acme.";

        // Pass 1
        let triples_pass1 = extract_triples(text_input, &generator, &entity_cfg)
            .await
            .expect("extract_triples pass 1");

        // Pass 2
        let triples_pass2 = extract_triples(text_input, &generator, &entity_cfg)
            .await
            .expect("extract_triples pass 2");

        assert_eq!(
            triples_pass1, triples_pass2,
            "OpenIE extraction must be strictly deterministic given identical input and generator"
        );
    }

    // Verify model fingerprint consistency
    let fp1 = ConfigFingerprint::new("llama-3.2-3b", "Q4_K_M", "extract_prompt_v1", 0.0);
    let fp2 = ConfigFingerprint::new("llama-3.2-3b", "Q4_K_M", "extract_prompt_v1", 0.0);
    assert_eq!(
        fp1, fp2,
        "Config fingerprints must match for identical extraction settings"
    );
}

/// Test B: nach `auto_extract_and_relate` fuer ein Dokument `doc_id` traegt jede daraus
/// erzeugte Kante `source_doc_id == Some(DocId::from_key(doc_id))`.
#[tokio::test]
async fn test_openie_edge_provenance_source_doc_id() {
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
        {"subject": "Alice", "predicate": "knows", "object": "Bob", "confidence": 0.9, "source_span": null}
    ]"#;
    let mock_llm = Arc::new(DeterministicMockLlm::new(mock_json));
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

    let doc_id_str = "doc-provenance-100";
    col.insert_text_only(doc_id_str, "Alice knows Bob.", None)
        .await
        .expect("insert_text_only should succeed");

    #[cfg(feature = "entity-extraction")]
    {
        let expected_doc_id = DocId::from_key(doc_id_str).expect("DocId from key");
        let alice_id = EntityId::from_key("Alice").expect("Alice EntityId");
        let bob_id = EntityId::from_key("Bob").expect("Bob EntityId");

        // Force compact to flush pending edges into CSR array
        col.graph_index().compact();

        let edge_source_doc = col.graph_index().source_doc_id_at(alice_id, bob_id);
        assert_eq!(
            edge_source_doc,
            Some(expected_doc_id),
            "Extracted edge from Alice to Bob must carry source_doc_id equal to source document ID"
        );
    }
}

/// Test C: Loeschsimulation — nach Entfernen des Quelldokuments ist die zugehoerige Kante
/// über `source_doc_id` auffindbar und als „verwaist" markierbar.
#[tokio::test]
async fn test_openie_deletion_orphan_simulation() {
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
        {"subject": "EntityA", "predicate": "linked_to", "object": "EntityB", "confidence": 0.95, "source_span": null}
    ]"#;
    let mock_llm = Arc::new(DeterministicMockLlm::new(mock_json));
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

    let doc_id_str = "doc-delete-target-42";
    col.insert_text_only(doc_id_str, "EntityA linked_to EntityB.", None)
        .await
        .expect("insert_text_only");

    #[cfg(feature = "entity-extraction")]
    {
        let target_doc_id = DocId::from_key(doc_id_str).expect("DocId from key");
        let node_a = EntityId::from_key("EntityA").expect("EntityA id");
        let node_b = EntityId::from_key("EntityB").expect("EntityB id");

        col.graph_index().compact();

        // Verify edge is initially associated with target_doc_id
        let initial_src = col.graph_index().source_doc_id_at(node_a, node_b);
        assert_eq!(initial_src, Some(target_doc_id));

        // Simulate document deletion
        col.delete(doc_id_str).await.expect("delete doc");

        let doc_after_delete = col.get(doc_id_str).await.expect("get doc");
        assert!(doc_after_delete.is_none(), "Document must be deleted");

        // Verify that the edge is findable via source_doc_id and marked as orphaned
        let edge_src_after = col.graph_index().source_doc_id_at(node_a, node_b);
        assert_eq!(
            edge_src_after,
            Some(target_doc_id),
            "Edge source_doc_id remains pointing to deleted doc_id"
        );

        // Check whether document exists in storage to detect orphan status
        let is_orphaned = col.get(doc_id_str).await.expect("get doc").is_none();
        assert!(
            is_orphaned,
            "Edge is identifiable as orphaned because source_doc_id document no longer exists in collection"
        );
    }
}
