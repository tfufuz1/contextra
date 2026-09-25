use contextra_rank::fusion::{reciprocal_rank_fusion, SearchResult};
use contextra_types::DocId;
use std::collections::HashMap;

/// Helper function adapting RRF interface to accept lists of `DocId` and `k`.
fn rrf_fusion(result_lists: &[Vec<DocId>], k: usize) -> HashMap<DocId, f32> {
    let result_sets: Vec<Vec<SearchResult>> = result_lists
        .iter()
        .map(|list| {
            list.iter()
                .map(|doc_id| SearchResult {
                    id: doc_id.0.to_string(),
                    score: 0.0,
                    metadata: None,
                    matched_signals: vec![],
                    provenance: None,
                })
                .collect()
        })
        .collect();

    // In `reciprocal_rank_fusion`, internal RRF smoothing parameter is k=60.
    // `k` passed here controls max_results returned (e.g. max 60 results).
    let fused = reciprocal_rank_fusion(result_sets, k);
    let mut scores = HashMap::new();
    for res in fused {
        if let Ok(id_val) = res.id.parse::<u64>() {
            scores.insert(DocId(id_val), res.score);
        }
    }
    scores
}

#[test]
fn rrf_known_values() {
    let list_a = vec![DocId(1), DocId(2), DocId(3)];
    let list_b = vec![DocId(2), DocId(1), DocId(3)];
    let k = 60;
    let scores = rrf_fusion(&[list_a, list_b], k);

    // Derivation of expected scores with k = 60 (1-based ranking):
    // DocId(1): List A Rank 1 -> 1/(60+1) = 1/61 ≈ 0.01639344; List B Rank 2 -> 1/(60+2) = 1/62 ≈ 0.01612903.
    //           Total = 1/61 + 1/62 ≈ 0.03252247
    // DocId(2): List A Rank 2 -> 1/(60+2) = 1/62 ≈ 0.01612903; List B Rank 1 -> 1/(60+1) = 1/61 ≈ 0.01639344.
    //           Total = 1/62 + 1/61 ≈ 0.03252247 (same as DocId(1))
    // DocId(3): List A Rank 3 -> 1/(60+3) = 1/63 ≈ 0.01587302; List B Rank 3 -> 1/(60+3) = 1/63 ≈ 0.01587302.
    //           Total = 1/63 + 1/63 = 2/63 ≈ 0.03174603
    assert!((scores[&DocId(3)] - 0.03175).abs() < 0.001);
    assert!((scores[&DocId(1)] - 0.03252).abs() < 0.001);
    assert!((scores[&DocId(2)] - 0.03252).abs() < 0.001);
}

#[test]
fn rrf_with_disjoint_lists() {
    let list_a = vec![DocId(10), DocId(20)];
    let list_b = vec![DocId(30), DocId(40)];
    let k = 60;
    let scores = rrf_fusion(&[list_a, list_b], k);

    // Derivation of expected scores for disjoint lists (k = 60, 1-based rank):
    // Since lists are completely disjoint, each document only receives a contribution
    // from the list in which it appears (no cross-contamination from the other list).
    //
    // DocId(10): List A Rank 1 -> 1/(60+1) = 1/61 ≈ 0.016393443
    // DocId(20): List A Rank 2 -> 1/(60+2) = 1/62 ≈ 0.016129032
    // DocId(30): List B Rank 1 -> 1/(60+1) = 1/61 ≈ 0.016393443
    // DocId(40): List B Rank 2 -> 1/(60+2) = 1/62 ≈ 0.016129032
    let expected_rank1 = 1.0 / 61.0_f32; // ~0.016393443
    let expected_rank2 = 1.0 / 62.0_f32; // ~0.016129032

    assert!((scores[&DocId(10)] - expected_rank1).abs() < 1e-6);
    assert!((scores[&DocId(20)] - expected_rank2).abs() < 1e-6);
    assert!((scores[&DocId(30)] - expected_rank1).abs() < 1e-6);
    assert!((scores[&DocId(40)] - expected_rank2).abs() < 1e-6);
}

#[test]
fn rrf_with_three_lists() {
    let list_a = vec![DocId(100), DocId(200), DocId(300)];
    let list_b = vec![DocId(200), DocId(100), DocId(400)];
    let list_c = vec![DocId(300), DocId(400), DocId(100)];
    let k = 60;
    let scores = rrf_fusion(&[list_a, list_b, list_c], k);

    // Derivation of expected scores across 3 lists (k = 60, 1-based rank):
    //
    // DocId(100) appears in all 3 lists at different positions:
    // - List A: Rank 1 -> Term 1 = 1 / (60 + 1) = 1/61 ≈ 0.016393443
    // - List B: Rank 2 -> Term 2 = 1 / (60 + 2) = 1/62 ≈ 0.016129032
    // - List C: Rank 3 -> Term 3 = 1 / (60 + 3) = 1/63 ≈ 0.015873016
    // Total Score DocId(100) = Term 1 + Term 2 + Term 3
    //                       = 1/61 + 1/62 + 1/63 ≈ 0.048395491
    let expected_doc100 = (1.0 / 61.0_f32) + (1.0 / 62.0_f32) + (1.0 / 63.0_f32);
    assert!((scores[&DocId(100)] - expected_doc100).abs() < 1e-6);

    // DocId(200) appears in List A (Rank 2) and List B (Rank 1):
    // - List A: Rank 2 -> 1 / (60 + 2) = 1/62 ≈ 0.016129032
    // - List B: Rank 1 -> 1 / (60 + 1) = 1/61 ≈ 0.016393443
    // Total Score DocId(200) = 1/61 + 1/62 ≈ 0.032522475
    let expected_doc200 = (1.0 / 61.0_f32) + (1.0 / 62.0_f32);
    assert!((scores[&DocId(200)] - expected_doc200).abs() < 1e-6);

    // DocId(300) appears in List A (Rank 3) and List C (Rank 1):
    // - List A: Rank 3 -> 1 / (60 + 3) = 1/63 ≈ 0.015873016
    // - List C: Rank 1 -> 1 / (60 + 1) = 1/61 ≈ 0.016393443
    // Total Score DocId(300) = 1/61 + 1/63 ≈ 0.032266459
    let expected_doc300 = (1.0 / 61.0_f32) + (1.0 / 63.0_f32);
    assert!((scores[&DocId(300)] - expected_doc300).abs() < 1e-6);
}
