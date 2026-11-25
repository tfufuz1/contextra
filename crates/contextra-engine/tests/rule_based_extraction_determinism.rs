#![cfg(not(loom))]
// FILE-CONTEXT
// ZWECK: Determinismus- & Provenienztests fuer RuleBasedExtractor (v17 §6.3, §10.5, P28).
// INVARIANTEN: No unwrap/expect/panic in production code; 100% Provenienz belegt; 100 Identische Laeufe.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::collection::crud::{
    AutoExtractionConfig, AutoExtractionMode, EntityExtractionConfig,
};
#[cfg(feature = "entity-extraction")]
use contextra_engine::extraction::{
    extract_triples_for_config, ExtractorMode, RuleBasedExtractor, MAX_EXTRACTION_TEXT_LENGTH,
};
use contextra_ports::{BoxFuture, TextEmbeddingEngine};
use contextra_types::Result;
#[cfg(feature = "entity-extraction")]
use contextra_types::{DocId, EntityId};
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

/// Test 1: Determinismus & Stabile Sortierung über 100 Läufe mit identischer Eingabe.
#[tokio::test]
async fn test_rule_based_extraction_100_runs_determinism() {
    #[cfg(feature = "entity-extraction")]
    {
        let text = "Alice arbeitet bei Acme GmbH. Bob wohnt in Berlin! Carol leitet TeamX; Dave gruendete StartupY.";
        let config = EntityExtractionConfig {
            enabled: true,
            max_llm_calls_per_cycle: 10,
            min_confidence: 0.5,
        };

        let first_run = extract_triples_for_config(text, None, &config)
            .await
            .expect("first run");

        assert!(!first_run.is_empty(), "First run should produce triples");

        for i in 2..=100 {
            let current_run = extract_triples_for_config(text, None, &config)
                .await
                .expect(&format!("run {i}"));

            assert_eq!(
                first_run, current_run,
                "Run {i} output differed from first run. Output must be strictly deterministic across 100 runs."
            );
        }
    }
}

/// Test 2: Provenienz — 100 % der aus der RuleBased-Extraktion stammenden Kanten tragen `source_doc_id`.
#[tokio::test]
async fn test_rule_based_extraction_provenance_100_percent() {
    let tmp = TempDir::new().expect("temp dir");
    let config = contextra_engine::ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        ..Default::default()
    };
    let db = contextra_engine::Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");

    let auto_cfg = AutoExtractionConfig {
        mode: AutoExtractionMode::Enabled,
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
    col.set_auto_extraction_config(auto_cfg);

    let doc_id_str = "rule-doc-provenance-200";
    let text_content = "Alice arbeitet bei Acme. Bob wohnt in Berlin. Carol leitet TeamX.";

    col.insert_text_only(doc_id_str, text_content, None)
        .await
        .expect("insert_text_only");

    #[cfg(feature = "entity-extraction")]
    {
        col.graph_index().compact();

        let expected_doc_id = DocId::from_key(doc_id_str).expect("DocId");

        // Verify edge 1: Alice -> Acme
        let alice_id = EntityId::from_key("Alice").expect("Alice EntityId");
        let acme_id = EntityId::from_key("Acme").expect("Acme EntityId");
        let edge1_doc = col.graph_index().source_doc_id_at(alice_id, acme_id);
        assert_eq!(
            edge1_doc,
            Some(expected_doc_id),
            "100% Provenance assertion failed for Alice -> Acme edge"
        );

        // Verify edge 2: Bob -> Berlin
        let bob_id = EntityId::from_key("Bob").expect("Bob EntityId");
        let berlin_id = EntityId::from_key("Berlin").expect("Berlin EntityId");
        let edge2_doc = col.graph_index().source_doc_id_at(bob_id, berlin_id);
        assert_eq!(
            edge2_doc,
            Some(expected_doc_id),
            "100% Provenance assertion failed for Bob -> Berlin edge"
        );

        // Verify edge 3: Carol -> TeamX
        let carol_id = EntityId::from_key("Carol").expect("Carol EntityId");
        let team_id = EntityId::from_key("TeamX").expect("TeamX EntityId");
        let edge3_doc = col.graph_index().source_doc_id_at(carol_id, team_id);
        assert_eq!(
            edge3_doc,
            Some(expected_doc_id),
            "100% Provenance assertion failed for Carol -> TeamX edge"
        );
    }
}

/// Test 3: Edge cases — Leerer Text, Unicode NFC vs. NFD, überlange Texte, nur Satzzeichen.
#[tokio::test]
async fn test_rule_based_extraction_edge_cases() {
    #[cfg(feature = "entity-extraction")]
    {
        let extractor = RuleBasedExtractor::new();
        let config = EntityExtractionConfig {
            enabled: true,
            max_llm_calls_per_cycle: 10,
            min_confidence: 0.5,
        };

        // Edge case 1: Leerer Text
        let empty_res = extractor.extract("", &config).expect("empty text");
        assert!(empty_res.is_empty(), "Empty text must yield 0 triples");

        let whitespace_res = extractor
            .extract("   \n\t   ", &config)
            .expect("whitespace text");
        assert!(
            whitespace_res.is_empty(),
            "Whitespace-only text must yield 0 triples"
        );

        // Edge case 2: Nur Satzzeichen
        let punc_res = extractor
            .extract("... !?? ;;; ---", &config)
            .expect("punctuation text");
        assert!(
            punc_res.is_empty(),
            "Punctuation-only text must yield 0 triples"
        );

        // Edge case 3: Unicode NFC vs. NFD Normalisierung (pre-composed vs decomposed u-umlaut)
        let text_nfc = "München wohnt in Bayern.";
        let text_nfd = "Mu\u{0308}nchen wohnt in Bayern.";

        let res_nfc = extractor.extract(text_nfc, &config).expect("NFC text");
        let res_nfd = extractor.extract(text_nfd, &config).expect("NFD text");

        assert_eq!(
            res_nfc.len(),
            res_nfd.len(),
            "NFC and NFD input should yield matching triple count"
        );

        // Edge case 4: Überlanger Text überschreitet Obergrenze
        let overlong_text = "a".repeat(MAX_EXTRACTION_TEXT_LENGTH + 1);
        let overlong_err = extractor.extract(&overlong_text, &config);
        assert!(
            overlong_err.is_err(),
            "Text exceeding MAX_EXTRACTION_TEXT_LENGTH must return an error"
        );
    }
}
