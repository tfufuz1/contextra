// FILE-CONTEXT
// ZWECK: Schließen von Qualitäts- und Randfall-Lücken in der contextra-vector Test-Suite.
// INVARIANTEN:
// 1. ACORN Suche mit 0% Selektivität (kein Treffer) liefert leere Treffermenge ohne Panic.
// 2. ACORN Suche mit 100% Selektivität liefert identische Ergebnisse wie ungefilterte Suche.
// 3. SQ8 Quantisierung mit konstanten Vektoren (alle Dimensionen identisch) dividiert nicht durch 0.
// 4. Cosine-Distanz mit Null-Vektor liefert definierten Maximalabstand 1.0 (kein NaN).
// 5. kNN-Suche mit k größer als Gesamtkandidatenanzahl liefert alle verfügbaren Kandidaten ohne Index-Out-Of-Bounds.
// HOTSPOTS: 35-150
// STAND: 2026-11-12

// ANCHOR[contextra-vector:W10-QUALITY-001]

use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::distance::ScalarDistance;
use contextra_vector::distance::contract::{DistanceFn, Metric};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::quantize::ScalarQuantizer;

#[test]
fn test_gap_cosine_distance_zero_vector_no_nan() {
    let sd = ScalarDistance::new(Metric::Cosine);
    let zero_vec = vec![0.0f32; 16];
    let normal_vec = vec![1.0f32; 16];

    let dist = sd.distance(&zero_vec, &normal_vec);
    assert!(!dist.is_nan(), "Cosine distance with zero vector must not yield NaN");
    assert_eq!(dist, 1.0, "Cosine distance with zero vector must equal 1.0 fallback");

    let dist_both_zero = sd.distance(&zero_vec, &zero_vec);
    assert!(!dist_both_zero.is_nan(), "Cosine distance between zero vectors must not be NaN");
    assert_eq!(dist_both_zero, 1.0);
}

#[test]
fn test_gap_sq8_quantizer_constant_vector_no_div_zero() {
    let dim = 8;
    let const_vec1 = vec![5.0f32; dim];
    let const_vec2 = vec![5.0f32; dim];

    let quantizer = ScalarQuantizer::try_train(&[&const_vec1, &const_vec2], dim);
    assert!(quantizer.is_ok(), "Training on constant vectors must succeed");

    let q = quantizer.unwrap();
    let code = q.quantize(&const_vec1);
    assert!(code.is_ok(), "Quantizing constant vector must succeed");

    let deq = q.dequantize(&code.unwrap());
    assert!(deq.is_ok(), "Dequantizing must succeed");
    for &val in deq.unwrap().iter() {
        assert!(!val.is_nan(), "Dequantized value must not be NaN");
    }
}

#[tokio::test]
async fn test_gap_search_k_greater_than_total_candidates() {
    let dim = 4;
    let config = HnswConfig {
        dimension: dim,
        max_elements: 100,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).unwrap();
    let tx = TxId::new(1);

    index.insert(tx, DocId::new(1), &[1.0, 0.0, 0.0, 0.0]).await.unwrap();
    index.insert(tx, DocId::new(2), &[0.0, 1.0, 0.0, 0.0]).await.unwrap();
    index.insert(tx, DocId::new(3), &[0.0, 0.0, 1.0, 0.0]).await.unwrap();

    let results = index.search(&[1.0, 0.1, 0.0, 0.0], 100).await.unwrap();
    assert_eq!(results.len(), 3, "Search should return all 3 available vectors without panic");
    assert_eq!(results[0].doc_id, DocId::new(1));
}

#[tokio::test]
async fn test_gap_acorn_selectivity_zero_and_hundred_percent() {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        max_elements: 100,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).unwrap();
    let tx = TxId::new(1);

    for i in 1..=20 {
        let v: Vec<f32> = (0..dim).map(|d| ((i * dim + d) as f32 * 0.1).sin()).collect();
        index.insert(tx, DocId::new(i as u64), &v).await.unwrap();
    }

    let q: Vec<f32> = (0..dim).map(|d| (d as f32 * 0.2).cos()).collect();

    let empty_results = index.search_filtered(&q, 10, |_| false).await.unwrap();
    assert!(empty_results.is_empty(), "0% selectivity must yield empty result list");

    let full_filtered = index.search_filtered(&q, 10, |_| true).await.unwrap();
    let unfiltered = index.search(&q, 10).await.unwrap();

    assert_eq!(full_filtered.len(), unfiltered.len());
    for (f, u) in full_filtered.iter().zip(unfiltered.iter()) {
        assert_eq!(f.doc_id, u.doc_id, "100% filtered search must match unfiltered search");
    }
}
