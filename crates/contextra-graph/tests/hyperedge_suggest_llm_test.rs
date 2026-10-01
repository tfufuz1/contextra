use contextra_graph::{
    validate_candidates_with_llm, GraphMutationError, HyperEdgeCandidate,
};
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_types::{DocId, EntityId};
use std::sync::atomic::{AtomicUsize, Ordering};

struct MockLlmTextGenerator {
    response: String,
    call_count: AtomicUsize,
}

impl MockLlmTextGenerator {
    fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            call_count: AtomicUsize::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl LlmTextGenerator for MockLlmTextGenerator {
    fn generate<'a>(
        &'a self,
        _prompt: &'a str,
    ) -> BoxFuture<'a, contextra_ports::Result<String>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }
}

#[tokio::test]
async fn test_validate_candidates_llm_budget_guard() {
    let doc = DocId::from_key("doc-1").unwrap();
    let candidates = vec![
        HyperEdgeCandidate {
            participants: vec![EntityId::new(1), EntityId::new(2)],
            co_occurrence_count: 1,
            source_doc_ids: vec![doc],
        },
        HyperEdgeCandidate {
            participants: vec![EntityId::new(3), EntityId::new(4)],
            co_occurrence_count: 1,
            source_doc_ids: vec![doc],
        },
    ];

    let generator = MockLlmTextGenerator::new("YES: relates_to");
    let label_fn = |_| None;

    // Budget = 1, candidates = 2 -> exceeds max_llm_calls_per_cycle
    let result = validate_candidates_with_llm(&candidates, &label_fn, &generator, 1).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, GraphMutationError::Internal(ref msg) if msg.contains("LLM validation budget exceeded")),
        "Expected budget exceeded error, got: {err:?}"
    );
    assert_eq!(generator.calls(), 0, "No LLM calls should be made when budget guard triggers");
}

#[tokio::test]
async fn test_validate_candidates_llm_success() {
    let doc = DocId::from_key("doc-1").unwrap();
    let candidates = vec![
        HyperEdgeCandidate {
            participants: vec![EntityId::new(10), EntityId::new(20)],
            co_occurrence_count: 2,
            source_doc_ids: vec![doc],
        },
        HyperEdgeCandidate {
            participants: vec![EntityId::new(30), EntityId::new(40), EntityId::new(50)],
            co_occurrence_count: 3,
            source_doc_ids: vec![doc],
        },
    ];

    let generator = MockLlmTextGenerator::new("YES: interacts_with");
    let label_fn = |id: EntityId| Some(format!("Entity_{}", id.inner()));

    let result = validate_candidates_with_llm(&candidates, &label_fn, &generator, 5).await;

    assert!(result.is_ok(), "Validation should succeed within budget");
    let validated = result.unwrap();

    assert_eq!(validated.len(), 2);
    assert_eq!(generator.calls(), 2);

    for item in &validated {
        assert!(item.accepted, "Candidate should be accepted on YES response");
        assert_eq!(item.predicate, "interacts_with");
        assert!((item.llm_confidence - 0.9).abs() < f32::EPSILON);
    }
}

#[tokio::test]
async fn test_validate_candidates_llm_rejection() {
    let doc = DocId::from_key("doc-1").unwrap();
    let candidates = vec![HyperEdgeCandidate {
        participants: vec![EntityId::new(100), EntityId::new(200)],
        co_occurrence_count: 1,
        source_doc_ids: vec![doc],
    }];

    let generator = MockLlmTextGenerator::new("NO: unrelated entities");
    let label_fn = |_| None;

    let result = validate_candidates_with_llm(&candidates, &label_fn, &generator, 5).await;

    assert!(result.is_ok());
    let validated = result.unwrap();

    assert_eq!(validated.len(), 1);
    assert_eq!(generator.calls(), 1);

    assert!(!validated[0].accepted, "Candidate should be rejected on NO response");
    assert!(validated[0].predicate.is_empty());
    assert_eq!(validated[0].llm_confidence, 0.0);
}
