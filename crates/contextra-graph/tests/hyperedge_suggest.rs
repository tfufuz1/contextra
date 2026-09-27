use contextra_graph::{
    compute_co_occurrence_candidates, validate_candidates_with_llm, GraphMutationError,
    HyperEdgeCandidate, MAX_RELATE_PARTICIPANTS,
};
use contextra_ports::{BoxFuture, TextGenerator};
use contextra_types::{DocId, EntityId};
use std::sync::atomic::{AtomicUsize, Ordering};

struct MockTextGenerator {
    response: String,
    call_count: AtomicUsize,
}

impl MockTextGenerator {
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

impl TextGenerator for MockTextGenerator {
    fn generate_text<'a>(
        &'a self,
        _prompt: &'a str,
    ) -> BoxFuture<'a, contextra_ports::Result<String>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }
}

#[test]
fn test_co_occurrence_counting_is_deterministic_on_input_permutation() {
    let doc1 = DocId::from_key("doc-1").unwrap();
    let doc2 = DocId::from_key("doc-2").unwrap();
    let doc3 = DocId::from_key("doc-3").unwrap();

    let e1 = EntityId::new(10);
    let e2 = EntityId::new(20);
    let e3 = EntityId::new(30);

    // Document set order 1
    let docs_order1 = vec![
        (doc1, vec![e1, e2, e3]),
        (doc2, vec![e2, e1]),
        (doc3, vec![e3, e2, e1]),
    ];

    // Document set order 2 (permuted documents and internal entity lists)
    let docs_order2 = vec![
        (doc3, vec![e1, e3, e2]),
        (doc1, vec![e3, e1, e2]),
        (doc2, vec![e1, e2]),
    ];

    let candidates1 = compute_co_occurrence_candidates(&docs_order1, 1);
    let candidates2 = compute_co_occurrence_candidates(&docs_order2, 1);

    assert_eq!(candidates1, candidates2);
    assert_eq!(candidates1.len(), 2);

    let mut expected_docs_1_3 = vec![doc1, doc3];
    expected_docs_1_3.sort_unstable_by_key(|d| d.inner());

    // Candidate 0: [10, 20] occurs in doc2 (count 1)
    assert_eq!(candidates1[0].participants, vec![e1, e2]);
    assert_eq!(candidates1[0].co_occurrence_count, 1);
    assert_eq!(candidates1[0].source_doc_ids, vec![doc2]);

    // Candidate 1: [10, 20, 30] occurs in doc1, doc3 (count 2)
    assert_eq!(candidates1[1].participants, vec![e1, e2, e3]);
    assert_eq!(candidates1[1].co_occurrence_count, 2);
    assert_eq!(candidates1[1].source_doc_ids, expected_docs_1_3);
}

#[test]
fn test_combinations_below_min_co_occurrence_filtered_out() {
    let doc1 = DocId::from_key("doc-1").unwrap();
    let doc2 = DocId::from_key("doc-2").unwrap();

    let e1 = EntityId::new(100);
    let e2 = EntityId::new(200);

    let docs = vec![(doc1, vec![e1, e2]), (doc2, vec![e1, e2])];

    // min_co_occurrence = 2 -> [e1, e2] appears in both docs
    let candidates = compute_co_occurrence_candidates(&docs, 2);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].participants, vec![e1, e2]);
    assert_eq!(candidates[0].co_occurrence_count, 2);

    // min_co_occurrence = 3 -> filtered out
    let filtered_candidates = compute_co_occurrence_candidates(&docs, 3);
    assert!(filtered_candidates.is_empty());
}

#[test]
fn test_combinations_exceeding_max_relate_participants_discarded_not_truncated() {
    let doc1 = DocId::from_key("doc-huge").unwrap();
    let doc2 = DocId::from_key("doc-normal").unwrap();

    let huge_entity_set: Vec<EntityId> = (1..=(MAX_RELATE_PARTICIPANTS + 5))
        .map(|i| EntityId::new(i as u64))
        .collect();

    let normal_entity_set: Vec<EntityId> = (1..=5).map(|i| EntityId::new(i as u64)).collect();

    let docs = vec![
        (doc1, huge_entity_set.clone()),
        (doc2, normal_entity_set.clone()),
    ];

    let candidates = compute_co_occurrence_candidates(&docs, 1);

    // huge_entity_set has MAX_RELATE_PARTICIPANTS + 5 (>64) items and must be discarded, not truncated!
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].participants, normal_entity_set);
    assert_eq!(candidates[0].source_doc_ids, vec![doc2]);
}

#[tokio::test]
async fn test_validate_candidates_exceeding_budget_returns_err() {
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

    let generator = MockTextGenerator::new("YES: relates_to");
    let label_fn = |_| None;

    // Budget = 1, candidates = 2 -> exceeds max_llm_calls_per_cycle
    let result = validate_candidates_with_llm(&candidates, &label_fn, &generator, 1).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, GraphMutationError::Internal(ref msg) if msg.contains("LLM validation budget exceeded"))
    );
    assert_eq!(generator.calls(), 0); // No LLM calls executed due to budget check
}

#[tokio::test]
async fn test_mock_llm_rejection_results_in_accepted_false_without_error() {
    let doc = DocId::from_key("doc-1").unwrap();
    let candidates = vec![
        HyperEdgeCandidate {
            participants: vec![EntityId::new(10), EntityId::new(20)],
            co_occurrence_count: 2,
            source_doc_ids: vec![doc],
        },
        HyperEdgeCandidate {
            participants: vec![EntityId::new(30), EntityId::new(40)],
            co_occurrence_count: 2,
            source_doc_ids: vec![doc],
        },
    ];

    let generator = MockTextGenerator::new("NO: this relationship is not meaningful");
    let label_fn = |id: EntityId| Some(format!("EntityName_{}", id.inner()));

    let result = validate_candidates_with_llm(&candidates, &label_fn, &generator, 10).await;

    assert!(result.is_ok());
    let validated = result.unwrap();

    assert_eq!(validated.len(), 2);
    assert_eq!(generator.calls(), 2);

    for item in &validated {
        assert!(!item.accepted);
        assert_eq!(item.llm_confidence, 0.0);
        assert!(item.predicate.is_empty());
    }
}
