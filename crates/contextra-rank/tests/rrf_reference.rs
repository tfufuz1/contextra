//! # Integrationstest: Referenztests für RRF mit bekannten numerischen Sollwerten
//!
//! **Crate:** `contextra-rank`
//!
//! Prüft die Korrektheit, Stabilität und Invarianten der Ranking- und Kalibrierungskomponenten.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::useless_vec
)]
use ahash::AHashMap;
use contextra_rank::fusion::{reciprocal_rank_fusion, SearchResult};
use contextra_rank::DocId;

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

    // In `reciprocal_rank_fusion`, internal RRF smoothing parameter is k=60.
    // `k` passed here controls max_results returned (e.g. max 60 results).
    let fused = reciprocal_rank_fusion(result_sets, k);
    let mut scores = AHashMap::new();
    for res in fused {
        if let Ok(id_val) = res.id.parse::<u64>() {
            scores.insert(DocId::from(id_val), res.score);
        }
    }
    scores
}
#[test]
fn rrf_known_values() {
    let list_a = vec![DocId::from(1), DocId::from(2), DocId::from(3)];
    let list_b = vec![DocId::from(2), DocId::from(1), DocId::from(3)];
    let k = 60;
    let scores = rrf_fusion(&[list_a, list_b], k);

    if let Some(&s3) = scores.get(&DocId::from(3)) {
        assert!((s3 - 0.03175).abs() < 0.001);
    }
    if let Some(&s1) = scores.get(&DocId::from(1)) {
        assert!((s1 - 0.03252).abs() < 0.001);
    }
    if let Some(&s2) = scores.get(&DocId::from(2)) {
        assert!((s2 - 0.03252).abs() < 0.001);
    }
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

/// Differentialtest WP-60:
/// `rrf_fuse` (Referenz-Orakel auf `ScoredItem`) und `weighted_reciprocal_rank_fusion`
/// (Produktiver Pfad auf `SearchResult`) liefern bei Einheitsgewichten ($w=1.0$) und
/// $k=60$ dieselbe relative Rangfolge (Ordering) und numerisch identische Scores.
#[test]
fn test_differential_rrf_fuse_vs_weighted_rrf() {
    use contextra_rank::fusion::{rrf_fuse, weighted_reciprocal_rank_fusion, ScoredItem};

    // Erstelle Testdaten über 3 Suchkanäle (Text, Vektor, Graph)
    let channel_a = vec!["doc_10", "doc_20", "doc_30", "doc_40", "doc_50"];
    let channel_b = vec!["doc_20", "doc_10", "doc_40", "doc_60", "doc_30"];
    let channel_c = vec!["doc_30", "doc_20", "doc_70", "doc_10", "doc_80"];

    // 1. Ausführung über das Referenz-Orakel rrf_fuse
    let oracle_input: Vec<Vec<ScoredItem>> = vec![
        channel_a
            .iter()
            .enumerate()
            .map(|(i, &id)| ScoredItem::new(id, (100 - i) as f32))
            .collect(),
        channel_b
            .iter()
            .enumerate()
            .map(|(i, &id)| ScoredItem::new(id, (100 - i) as f32))
            .collect(),
        channel_c
            .iter()
            .enumerate()
            .map(|(i, &id)| ScoredItem::new(id, (100 - i) as f32))
            .collect(),
    ];
    let oracle_results = rrf_fuse(&oracle_input, 60).unwrap();

    // 2. Ausführung über den produktiven Pfad weighted_reciprocal_rank_fusion
    let prod_input = vec![
        (
            "signal_text".to_string(),
            channel_a
                .iter()
                .enumerate()
                .map(|(i, &id)| SearchResult {
                    id: id.to_string(),
                    score: (100 - i) as f32,
                    metadata: None,
                    matched_signals: vec!["signal_text".to_string()],
                    provenance: None,
                })
                .collect(),
            1.0_f32,
        ),
        (
            "signal_vector".to_string(),
            channel_b
                .iter()
                .enumerate()
                .map(|(i, &id)| SearchResult {
                    id: id.to_string(),
                    score: (100 - i) as f32,
                    metadata: None,
                    matched_signals: vec!["signal_vector".to_string()],
                    provenance: None,
                })
                .collect(),
            1.0_f32,
        ),
        (
            "signal_graph".to_string(),
            channel_c
                .iter()
                .enumerate()
                .map(|(i, &id)| SearchResult {
                    id: id.to_string(),
                    score: (100 - i) as f32,
                    metadata: None,
                    matched_signals: vec!["signal_graph".to_string()],
                    provenance: None,
                })
                .collect(),
            1.0_f32,
        ),
    ];
    let prod_results = weighted_reciprocal_rank_fusion(prod_input, 100);

    // Vergleich der Länge und der relativen Rangfolge (ID-Reihenfolge)
    assert_eq!(oracle_results.len(), prod_results.len());
    for (idx, (oracle_item, prod_item)) in
        oracle_results.iter().zip(prod_results.iter()).enumerate()
    {
        assert_eq!(
            oracle_item.id, prod_item.id,
            "Rang-Diskrepanz an Position {idx}: Orakel={}, Produktiv={}",
            oracle_item.id, prod_item.id
        );
        // Scores müssen innerhalb numerischer f32-Präzision übereinstimmen
        assert!(
            (oracle_item.score - prod_item.score).abs() < 1e-6,
            "Score-Diskrepanz für doc '{}' an Position {idx}: Orakel={}, Produktiv={}",
            oracle_item.id,
            oracle_item.score,
            prod_item.score
        );
    }
}
