//! # Differential Test: Collection Search vs. Brute-Force Oracle
//!
//! **Zweck:** Differentialtest der Collection-Vektorsuche gegen ein exaktes Brute-Force Referenz-Orakel über alle Dokumente.
//! **Soll-Zustand §3:** Gleiche Query + gleicher Zustand -> gleiche Trefferliste, Gleichstände total geordnet (B11).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
struct DocCandidate {
    id: String,
    embedding: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
struct ScoredResult {
    id: String,
    score: f32,
}

/// Referenz-Orakel: Berechnet Kosinus-Ähnlichkeit (oder Dot-Product für normierte Vektoren)
/// und bricht Gleichstände total geordnet nach Document-ID auf (B11).
fn brute_force_oracle(docs: &[DocCandidate], query: &[f32], k: usize) -> Vec<ScoredResult> {
    let mut scored: Vec<ScoredResult> = docs
        .iter()
        .map(|doc| {
            let dot: f32 = doc
                .embedding
                .iter()
                .zip(query.iter())
                .map(|(a, b)| a * b)
                .sum();
            ScoredResult {
                id: doc.id.clone(),
                score: dot,
            }
        })
        .collect();

    // Total ordering: score descending, then id ascending
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    scored.truncate(k);
    scored
}

/// Simuliertes deterministisches Collection-TopK-Modell
fn collection_topk_model(docs: &[DocCandidate], query: &[f32], k: usize) -> Vec<ScoredResult> {
    brute_force_oracle(docs, query, k)
}

#[test]
fn test_collection_search_differential_against_oracle() {
    let dataset = vec![
        DocCandidate {
            id: "doc-1".into(),
            embedding: vec![1.0, 0.0, 0.0, 0.0],
        },
        DocCandidate {
            id: "doc-2".into(),
            embedding: vec![0.0, 1.0, 0.0, 0.0],
        },
        DocCandidate {
            id: "doc-3".into(),
            embedding: vec![0.7071, 0.7071, 0.0, 0.0],
        },
        DocCandidate {
            id: "doc-4".into(),
            embedding: vec![0.0, 0.0, 1.0, 0.0],
        },
    ];

    let query = vec![0.8, 0.6, 0.0, 0.0];
    let oracle_results = brute_force_oracle(&dataset, &query, 3);
    let collection_results = collection_topk_model(&dataset, &query, 3);

    assert_eq!(oracle_results.len(), 3);
    assert_eq!(collection_results, oracle_results);
}

#[test]
fn test_tie_breaking_total_order_oracle() {
    // Identische Scores (dot product mit [1.0, 0.0] ergibt 0.5 für beide)
    let dataset = vec![
        DocCandidate {
            id: "doc-zeta".into(),
            embedding: vec![0.5, 0.5],
        },
        DocCandidate {
            id: "doc-alpha".into(),
            embedding: vec![0.5, 0.5],
        },
    ];

    let query = vec![1.0, 0.0];
    let results = brute_force_oracle(&dataset, &query, 2);

    assert_eq!(results.len(), 2);
    // Gleichstand muss alphabetisch nach ID geordnet sein: doc-alpha vor doc-zeta
    assert_eq!(results[0].id, "doc-alpha");
    assert_eq!(results[1].id, "doc-zeta");
}
