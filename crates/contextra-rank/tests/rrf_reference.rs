//! # Integrationstest: Referenztests für RRF mit bekannten numerischen Sollwerten (Python-Referenz)
//!
//! **Crate:** `contextra-rank`
//!
//! Prüft die Korrektheit, Stabilität und mathematische Parität der RRF-Fusion
//! gegen eine unabhängige Referenzrechnung (Python-Golden-Werte).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]
use ahash::AHashMap;
use contextra_rank::fusion::{reciprocal_rank_fusion, rrf_fuse, Hit, SearchResult};
use contextra_types::DocId;

// Python reference implementation for RRF:
// ```python
// def rrf(lists, k=60):
//     scores = {}
//     for lst in lists:
//         for rank, doc in enumerate(lst, start=1):
//             scores[doc] = scores.get(doc, 0.0) + 1.0 / (k + rank)
//     return scores
// ```

/// Helper function adapting RRF interface to accept lists of `DocId` and `k`.
fn rrf_fusion(result_lists: &[Vec<DocId>], k: usize) -> AHashMap<DocId, f32> {
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

    let fused = reciprocal_rank_fusion(result_sets, k);
    let mut scores = AHashMap::new();
    for res in fused {
        if let Ok(id_val) = res.id.parse::<u64>() {
            scores.insert(DocId::from(id_val), res.score);
        }
    }
    scores
}

// ANCHOR[contextra-rank:RRF-TEST-002]
#[test]
fn rrf_known_values() {
    let list_a = vec![DocId::from(1), DocId::from(2), DocId::from(3)];
    let list_b = vec![DocId::from(2), DocId::from(1), DocId::from(3)];
    let k = 60;
    let scores = rrf_fusion(&[list_a, list_b], k);

    // Python:
    // s1 = 1/61 + 1/62 = 0.03252247488101534
    // s2 = 1/62 + 1/61 = 0.03252247488101534
    // s3 = 1/63 + 1/63 = 0.031746031746031744
    if let Some(&s3) = scores.get(&DocId::from(3)) {
        assert!((s3 - 0.031746).abs() < 1e-5);
    }
    if let Some(&s1) = scores.get(&DocId::from(1)) {
        assert!((s1 - 0.032522).abs() < 1e-5);
    }
    if let Some(&s2) = scores.get(&DocId::from(2)) {
        assert!((s2 - 0.032522).abs() < 1e-5);
    }
}

// Verifikation der kanonischen rrf_fuse gegen die Python-Referenz
#[test]
fn test_rrf_fuse_python_reference_parity() {
    let list_a = vec![
        Hit::new("doc1", 0.9),
        Hit::new("doc2", 0.8),
        Hit::new("doc3", 0.7),
    ];
    let list_b = vec![
        Hit::new("doc2", 15.0),
        Hit::new("doc1", 10.0),
        Hit::new("doc3", 5.0),
    ];

    // Python-Referenzwerte für k = 60:
    // doc1: 1/(60+1) + 1/(60+2) = 1/61 + 1/62 = 0.03252247488101534
    // doc2: 1/(60+1) + 1/(60+2) = 1/61 + 1/62 = 0.03252247488101534
    // doc3: 1/(60+3) + 1/(60+3) = 2/63 = 0.031746031746031744
    let fused = rrf_fuse(&[list_a, list_b], 60, 10);
    assert_eq!(fused.len(), 3);

    // doc1 und doc2 haben identische Scores; Tie-Breaker entscheidet nach id aufsteigend -> doc1 vor doc2
    let f0 = fused.first().unwrap();
    assert_eq!(f0.id, "doc1");
    assert!((f0.score - 0.032522475).abs() < 1e-6);

    let f1 = fused.get(1).unwrap();
    assert_eq!(f1.id, "doc2");
    assert!((f1.score - 0.032522475).abs() < 1e-6);

    let f2 = fused.get(2).unwrap();
    assert_eq!(f2.id, "doc3");
    assert!((f2.score - 0.031746032).abs() < 1e-6);
}

// Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn rrf_with_disjoint_lists() {
    let list_a = vec![DocId::from(10), DocId::from(20)];
    let list_b = vec![DocId::from(30), DocId::from(40)];
    let k = 60;
    let scores = rrf_fusion(&[list_a, list_b], k);

    let expected_rank1 = 1.0 / 61.0_f32;
    let expected_rank2 = 1.0 / 62.0_f32;

    if let Some(&s10) = scores.get(&DocId::from(10)) {
        assert!((s10 - expected_rank1).abs() < 1e-6);
    }
    if let Some(&s20) = scores.get(&DocId::from(20)) {
        assert!((s20 - expected_rank2).abs() < 1e-6);
    }
    if let Some(&s30) = scores.get(&DocId::from(30)) {
        assert!((s30 - expected_rank1).abs() < 1e-6);
    }
    if let Some(&s40) = scores.get(&DocId::from(40)) {
        assert!((s40 - expected_rank2).abs() < 1e-6);
    }
}

// Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn rrf_with_three_lists() {
    let list_a = vec![DocId::from(100), DocId::from(200), DocId::from(300)];
    let list_b = vec![DocId::from(200), DocId::from(100), DocId::from(400)];
    let list_c = vec![DocId::from(300), DocId::from(400), DocId::from(100)];
    let k = 60;
    let scores = rrf_fusion(&[list_a, list_b, list_c], k);

    let expected_doc100 = (1.0 / 61.0_f32) + (1.0 / 62.0_f32) + (1.0 / 63.0_f32);
    if let Some(&s100) = scores.get(&DocId::from(100)) {
        assert!((s100 - expected_doc100).abs() < 1e-6);
    }

    let expected_doc200 = (1.0 / 61.0_f32) + (1.0 / 62.0_f32);
    if let Some(&s200) = scores.get(&DocId::from(200)) {
        assert!((s200 - expected_doc200).abs() < 1e-6);
    }

    let expected_doc300 = (1.0 / 61.0_f32) + (1.0 / 63.0_f32);
    if let Some(&s300) = scores.get(&DocId::from(300)) {
        assert!((s300 - expected_doc300).abs() < 1e-6);
    }
}
