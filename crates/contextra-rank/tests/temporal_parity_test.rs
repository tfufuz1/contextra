//! Paritätstest zwischen `contextra-rank::temporal` und der eingefrorenen Referenz (RK3.3).

#![allow(clippy::indexing_slicing)]

use serde_json::json;

fn make_candidate(
    id: &str,
    score: f32,
    metadata: Option<serde_json::Value>,
) -> contextra_rank::SearchResult {
    contextra_rank::SearchResult {
        id: id.to_string(),
        score,
        metadata,
        matched_signals: vec!["test".to_string()],
        provenance: None,
    }
}

#[test]
fn test_temporal_parity_filter_behavior() {
    let candidate_expired = make_candidate(
        "doc-expired",
        0.8,
        Some(json!({
            "valid_until": 100
        })),
    );
    let candidate_valid = make_candidate(
        "doc-valid",
        0.9,
        Some(json!({
            "valid_until": 200
        })),
    );
    let candidate_no_meta = make_candidate("doc-no-meta", 0.7, None);

    let candidates = vec![candidate_expired, candidate_valid, candidate_no_meta];
    let filtered = contextra_rank::temporal::apply_temporal_validity_filter(
        candidates.clone(),
        contextra_rank::local_types::TxId::new(1),
        Some(150),
    );

    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered.first().map(|r| r.id.as_str()), Some("doc-valid"));
    assert_eq!(filtered.get(1).map(|r| r.id.as_str()), Some("doc-no-meta"));

    let hist = contextra_rank::temporal::apply_temporal_validity_filter_at(candidates, 50);
    assert_eq!(hist.len(), 3, "At t=50 all documents are valid");
}

#[test]
fn test_temporal_parity_deterministic_sort() {
    let mut results = vec![
        make_candidate("doc-b", 0.8, None),
        make_candidate("doc-a", 0.9, None),
        make_candidate("doc-c", 0.8, None),
    ];

    contextra_rank::temporal::sort_candidates_deterministic(&mut results);
    assert_eq!(results.first().map(|r| r.id.as_str()), Some("doc-a"));
    assert_eq!(results.get(1).map(|r| r.id.as_str()), Some("doc-b"));
    assert_eq!(results.get(2).map(|r| r.id.as_str()), Some("doc-c"));
}
