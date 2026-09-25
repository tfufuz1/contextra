// FILE-CONTEXT
// ZWECK: Unit-Tests für OpenIE Entitätsextraktion.
// STAND: 2026-09-07

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::open_ie::extract_triples;
use super::types::{EntityExtractionConfig, ExtractedTriple};
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_types::{ContextraError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};

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

#[tokio::test]
async fn test_extract_triples_success() {
    let mock_json = r#"[
        {"subject": "Alice", "predicate": "works_at", "object": "Acme", "confidence": 0.9, "source_span": [0, 20]},
        {"subject": "Alice", "predicate": "knows", "object": "Bob", "confidence": 0.4, "source_span": null}
    ]"#;

    let generator = MockLlmGenerator::new(mock_json);
    let config = EntityExtractionConfig {
        enabled: true,
        max_llm_calls_per_cycle: 10,
        min_confidence: 0.5,
    };

    let triples = extract_triples("Alice works at Acme.", &generator, &config)
        .await
        .unwrap();

    assert_eq!(triples.len(), 1);
    assert_eq!(
        triples[0],
        ExtractedTriple {
            subject: "Alice".to_string(),
            predicate: "works_at".to_string(),
            object: "Acme".to_string(),
            confidence: 0.9,
            source_span: Some((0, 20)),
        }
    );
    assert_eq!(generator.call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_extract_triples_budget_limit_exceeded() {
    let generator = MockLlmGenerator::new("[]");
    let config = EntityExtractionConfig {
        enabled: true,
        max_llm_calls_per_cycle: 2,
        min_confidence: 0.5,
    };

    let text = "Sentence one. Sentence two. Sentence three. Sentence four.";
    let res = extract_triples(text, &generator, &config).await;

    assert!(res.is_err());
    match res.unwrap_err() {
        ContextraError::LimitExceeded { limit, context } => {
            assert_eq!(limit, 2);
            assert!(context.contains("exceeds max_llm_calls_per_cycle limit"));
        }
        err => panic!("Expected LimitExceeded error, got {err:?}"),
    }
    assert_eq!(generator.call_count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_extract_triples_invalid_json_returns_error() {
    let broken_json = "This is not valid JSON at all!";
    let generator = MockLlmGenerator::new(broken_json);
    let config = EntityExtractionConfig::default();

    let res = extract_triples("Some text here.", &generator, &config).await;

    assert!(res.is_err());
    match res.unwrap_err() {
        ContextraError::ParseError(msg) => {
            assert!(msg.contains("Failed to parse LLM entity extraction response as JSON"));
        }
        err => panic!("Expected ParseError, got {err:?}"),
    }
}
