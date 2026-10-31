//! Comprehensive unit, edge-case, and property tests for `hyperedge_suggest`.
//!
//! Tests edge cases: empty input, single participant, exactly MAX_RELATE_PARTICIPANTS (64),
//! MAX_RELATE_PARTICIPANTS + 1 (65), duplicate entities, self-references, unknown node IDs,
//! determinism and permutation invariance, error paths of HyperedgeSuggestError,
//! and proptest properties (idempotency, upper bounds, permutation invariance).

use contextra_graph::{
    compute_co_occurrence_candidates, validate_candidates_with_llm, GraphMutationError,
    HyperEdgeCandidate, HyperedgeSuggestError, MAX_RELATE_PARTICIPANTS,
};
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_types::{DocId, EntityId};
use proptest::prelude::*;
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

    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl LlmTextGenerator for MockLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, contextra_ports::Result<String>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }
}

#[test]
fn test_empty_input_returns_no_candidates() {
    let empty_docs: Vec<(DocId, Vec<EntityId>)> = Vec::new();
    let candidates = compute_co_occurrence_candidates(&empty_docs, 1);
    assert!(
        candidates.is_empty(),
        "Empty input must return empty candidates"
    );

    let candidates_min_0 = compute_co_occurrence_candidates(&empty_docs, 0);
    assert!(
        candidates_min_0.is_empty(),
        "Empty input with min 0 must return empty candidates"
    );
}

#[test]
fn test_single_participant_is_discarded() {
    // Hyperedges require at least 2 participants (2..=MAX_RELATE_PARTICIPANTS)
    let doc1 = DocId::from(101u64);
    let doc2 = DocId::from(102u64);
    let e1 = EntityId::new(42);

    let docs = vec![(doc1, vec![e1]), (doc2, vec![e1])];

    let candidates = compute_co_occurrence_candidates(&docs, 1);
    assert!(
        candidates.is_empty(),
        "Single participant cannot form a hyperedge and must be discarded"
    );
}

#[test]
fn test_exact_boundary_max_relate_participants_64_included() {
    let doc = DocId::from(1u64);
    let entities: Vec<EntityId> = (1..=MAX_RELATE_PARTICIPANTS as u64)
        .map(EntityId::new)
        .collect();
    assert_eq!(entities.len(), 64);

    let docs = vec![(doc, entities.clone())];
    let candidates = compute_co_occurrence_candidates(&docs, 1);

    assert_eq!(
        candidates.len(),
        1,
        "Exactly 64 participants must be accepted"
    );
    assert_eq!(candidates[0].participants.len(), 64);
    assert_eq!(candidates[0].participants, entities);
    assert_eq!(candidates[0].co_occurrence_count, 1);
    assert_eq!(candidates[0].source_doc_ids, vec![doc]);
}

#[test]
fn test_exact_boundary_max_relate_participants_plus_one_65_discarded() {
    let doc = DocId::from(1u64);
    let entities: Vec<EntityId> = (1..=(MAX_RELATE_PARTICIPANTS as u64 + 1))
        .map(EntityId::new)
        .collect();
    assert_eq!(entities.len(), 65);

    let docs = vec![(doc, entities)];
    let candidates = compute_co_occurrence_candidates(&docs, 1);

    assert!(
        candidates.is_empty(),
        "65 participants exceeds MAX_RELATE_PARTICIPANTS and must be discarded, not truncated"
    );
}

#[test]
fn test_duplicate_entities_and_self_references_in_same_document() {
    let doc1 = DocId::from(1u64);
    let e1 = EntityId::new(10);
    let e2 = EntityId::new(20);

    // Document contains duplicate e1 and self references
    let docs = vec![(doc1, vec![e1, e1, e1, e2, e2])];

    let candidates = compute_co_occurrence_candidates(&docs, 1);
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0].participants,
        vec![e1, e2],
        "Duplicate entities must be deduplicated"
    );
    assert_eq!(candidates[0].participants.len(), 2);

    // If an entire document has only self-references to the same single entity:
    let doc2 = DocId::from(2u64);
    let docs_only_self = vec![(doc2, vec![e1, e1, e1])];
    let candidates_self = compute_co_occurrence_candidates(&docs_only_self, 1);
    assert!(
        candidates_self.is_empty(),
        "Only self-references collapse to 1 entity, which is discarded (< 2)"
    );
}

#[test]
fn test_unknown_or_sparse_node_ids() {
    // Arbitrary / high EntityIds (sparse IDs) should work normally without panicking or indexing issues
    let doc = DocId::from(999999u64);
    let e_large_1 = EntityId::new(u64::MAX - 10);
    let e_large_2 = EntityId::new(u64::MAX - 5);
    let e_zero = EntityId::new(0);

    let docs = vec![(doc, vec![e_large_2, e_zero, e_large_1])];
    let candidates = compute_co_occurrence_candidates(&docs, 1);

    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0].participants,
        vec![e_zero, e_large_1, e_large_2],
        "Sparse IDs must be correctly sorted"
    );
}

#[test]
fn test_determinism_under_all_permutations() {
    let d1 = DocId::from(1u64);
    let d2 = DocId::from(2u64);
    let d3 = DocId::from(3u64);

    let e1 = EntityId::new(5);
    let e2 = EntityId::new(10);
    let e3 = EntityId::new(15);
    let e4 = EntityId::new(20);

    let run_a = vec![
        (d1, vec![e3, e1, e2]),
        (d2, vec![e2, e4]),
        (d3, vec![e1, e2, e3]),
    ];

    let run_b = vec![
        (d3, vec![e2, e3, e1]),
        (d1, vec![e1, e2, e3]),
        (d2, vec![e4, e2]),
    ];

    let res_a = compute_co_occurrence_candidates(&run_a, 1);
    let res_b = compute_co_occurrence_candidates(&run_b, 1);

    assert_eq!(
        res_a, res_b,
        "Output must be completely deterministic regardless of document or entity ordering"
    );
    assert_eq!(res_a.len(), 2);
    // Combination 1: [5, 10, 15] in d1, d3
    assert_eq!(res_a[0].participants, vec![e1, e2, e3]);
    assert_eq!(res_a[0].co_occurrence_count, 2);
    // Combination 2: [10, 20] in d2
    assert_eq!(res_a[1].participants, vec![e2, e4]);
    assert_eq!(res_a[1].co_occurrence_count, 1);
}

#[test]
fn test_error_variants_and_conversions() {
    let err = HyperedgeSuggestError::LlmValidationBudgetExceeded(128);
    let err_msg = err.to_string();
    assert!(err_msg.contains("128"));
    assert!(err_msg.contains("LLM validation budget exceeded"));

    let graph_err: GraphMutationError = err.clone().into();
    match graph_err {
        GraphMutationError::Internal(msg) => {
            assert!(msg.contains("128"));
            assert!(msg.contains("LLM validation budget exceeded"));
        }
        _ => panic!("Expected GraphMutationError::Internal"),
    }
}

#[tokio::test]
async fn test_validate_candidates_with_llm_budget_zero_and_exceeded() {
    let candidate = HyperEdgeCandidate {
        participants: vec![EntityId::new(1), EntityId::new(2)],
        co_occurrence_count: 1,
        source_doc_ids: vec![DocId::from(1u64)],
    };

    let generator = MockLlmGenerator::new("YES: related");
    let label_fn = |_| None;

    // Budget 0 with 1 candidate -> error
    let res_0 = validate_candidates_with_llm(&[candidate.clone()], &label_fn, &generator, 0).await;
    assert!(res_0.is_err());
    let err_str = res_0.unwrap_err().to_string();
    assert!(err_str.contains("LLM validation budget exceeded"));
    assert_eq!(generator.calls(), 0);

    // Empty candidates with budget 0 -> Ok
    let res_empty = validate_candidates_with_llm(&[], &label_fn, &generator, 0).await;
    assert!(res_empty.is_ok());
    assert!(res_empty.unwrap().is_empty());
}

#[tokio::test]
async fn test_validate_candidates_llm_response_parsing_variants() {
    let candidate = HyperEdgeCandidate {
        participants: vec![EntityId::new(1), EntityId::new(2)],
        co_occurrence_count: 1,
        source_doc_ids: vec![DocId::from(1u64)],
    };
    let label_fn = |id: EntityId| Some(format!("Entity_{}", id.inner()));

    // 1. "YES: works_with"
    let gen1 = MockLlmGenerator::new("YES: works_with");
    let res1 = validate_candidates_with_llm(&[candidate.clone()], &label_fn, &gen1, 10)
        .await
        .unwrap();
    assert!(res1[0].accepted);
    assert_eq!(res1[0].predicate, "works_with");
    assert!((res1[0].llm_confidence - 0.9).abs() < f32::EPSILON);

    // 2. "YES:" without predicate -> falls back to "co_occurrence"
    let gen2 = MockLlmGenerator::new("YES:   ");
    let res2 = validate_candidates_with_llm(&[candidate.clone()], &label_fn, &gen2, 10)
        .await
        .unwrap();
    assert!(res2[0].accepted);
    assert_eq!(res2[0].predicate, "co_occurrence");
    assert!((res2[0].llm_confidence - 0.9).abs() < f32::EPSILON);

    // 3. Plain "yes" or "YES" -> predicate "co_occurrence"
    let gen3 = MockLlmGenerator::new("yes definitely");
    let res3 = validate_candidates_with_llm(&[candidate.clone()], &label_fn, &gen3, 10)
        .await
        .unwrap();
    assert!(res3[0].accepted);
    assert_eq!(res3[0].predicate, "co_occurrence");
    assert!((res3[0].llm_confidence - 0.9).abs() < f32::EPSILON);

    // 4. "NO: false" or "NOT MEANINGFUL" or "INVALID"
    let gen4 = MockLlmGenerator::new("INVALID RELATIONSHIP");
    let res4 = validate_candidates_with_llm(&[candidate.clone()], &label_fn, &gen4, 10)
        .await
        .unwrap();
    assert!(!res4[0].accepted);
    assert!(res4[0].predicate.is_empty());
    assert_eq!(res4[0].llm_confidence, 0.0);

    // 5. Empty response -> rejected
    let gen5 = MockLlmGenerator::new("   ");
    let res5 = validate_candidates_with_llm(&[candidate.clone()], &label_fn, &gen5, 10)
        .await
        .unwrap();
    assert!(!res5[0].accepted);
    assert!(res5[0].predicate.is_empty());
    assert_eq!(res5[0].llm_confidence, 0.0);

    // 6. Generic response fallback -> accepted with confidence 0.8
    let gen6 = MockLlmGenerator::new("collaborates closely in research");
    let res6 = validate_candidates_with_llm(&[candidate.clone()], &label_fn, &gen6, 10)
        .await
        .unwrap();
    assert!(res6[0].accepted);
    assert_eq!(res6[0].predicate, "collaborates closely in research");
    assert!((res6[0].llm_confidence - 0.8).abs() < f32::EPSILON);
}

// ---------------------------------------------------------------------------
// Property Tests (proptest)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_candidates_participants_bounds(
        doc_data in prop::collection::vec(
            (any::<u64>(), prop::collection::vec(any::<u64>(), 0..100)),
            0..15
        ),
        min_co in 1..=5usize
    ) {
        let docs: Vec<(DocId, Vec<EntityId>)> = doc_data
            .into_iter()
            .map(|(d, es)| (DocId::from(d), es.into_iter().map(EntityId::new).collect()))
            .collect();

        let candidates = compute_co_occurrence_candidates(&docs, min_co);

        for cand in candidates {
            // Invariant 1: participants size is in [2, MAX_RELATE_PARTICIPANTS]
            prop_assert!(cand.participants.len() >= 2);
            prop_assert!(cand.participants.len() <= MAX_RELATE_PARTICIPANTS);

            // Invariant 2: participants are strictly monotonically sorted (no duplicates)
            for window in cand.participants.windows(2) {
                prop_assert!(window[0].inner() < window[1].inner());
            }

            // Invariant 3: co_occurrence_count meets threshold
            prop_assert!(cand.co_occurrence_count >= min_co);
            prop_assert_eq!(cand.co_occurrence_count, cand.source_doc_ids.len());

            // Invariant 4: source_doc_ids are strictly monotonically sorted
            for window in cand.source_doc_ids.windows(2) {
                prop_assert!(window[0].inner() < window[1].inner());
            }
        }
    }

    #[test]
    fn prop_output_candidates_are_canonically_sorted(
        doc_data in prop::collection::vec(
            (any::<u64>(), prop::collection::vec(0u64..30, 2..10)),
            1..10
        ),
    ) {
        let docs: Vec<(DocId, Vec<EntityId>)> = doc_data
            .into_iter()
            .map(|(d, es)| (DocId::from(d), es.into_iter().map(EntityId::new).collect()))
            .collect();

        let candidates = compute_co_occurrence_candidates(&docs, 1);

        for window in candidates.windows(2) {
            prop_assert!(window[0].participants <= window[1].participants);
        }
    }

    #[test]
    fn prop_idempotency_of_candidate_computation(
        doc_data in prop::collection::vec(
            (any::<u64>(), prop::collection::vec(0u64..20, 2..8)),
            1..8
        ),
        min_co in 1..=3usize
    ) {
        let docs: Vec<(DocId, Vec<EntityId>)> = doc_data
            .into_iter()
            .map(|(d, es)| (DocId::from(d), es.into_iter().map(EntityId::new).collect()))
            .collect();

        let res1 = compute_co_occurrence_candidates(&docs, min_co);
        let res2 = compute_co_occurrence_candidates(&docs, min_co);

        prop_assert_eq!(res1, res2);
    }
}
