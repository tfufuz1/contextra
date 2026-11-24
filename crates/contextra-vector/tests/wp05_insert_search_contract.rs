// FILE-CONTEXT
// ZWECK: Vertragstests für WP-VEC-05 (Einfüge- & Suchpfad, Tie-Break, K-Bounds, Atomarität).
// INVARIANTEN: Zero-Panic; deterministischer Tie-Break (DocId aufsteigend bei gleichem Score); Batch-Rollback isoliert.
// STAND: 2026-11-12

use contextra_vector::hnsw::HnswIndex;
use contextra_vector::types::{DistanceMetric, DocId, MAX_SEARCH_K};
use contextra_vector::HnswConfig;

fn build_test_index(dim: usize) -> HnswIndex {
    let config = HnswConfig {
        dimension: dim,
        max_elements: 10_000,
        m: 16,
        ef_construction: 64,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    HnswIndex::try_new(config).expect("HnswIndex::try_new must succeed with valid config")
}

#[tokio::test]
async fn test_wp05_tie_break_by_id_ascending() {
    let index = build_test_index(4);
    let tx = 1u64;

    // Drei Dokumente mit IDENTISCHEM Vektor einfügen
    let vec = [1.0f32, 2.0, 3.0, 4.0];
    index.insert(tx, DocId::from(300u64), &vec).await.unwrap();
    index.insert(tx, DocId::from(100u64), &vec).await.unwrap();
    index.insert(tx, DocId::from(200u64), &vec).await.unwrap();
    index.commit(tx).await.unwrap();

    let results = index.search_adaptive(&vec, 10).await.unwrap();
    assert_eq!(results.len(), 3);
    // Bei gleichem Score (Distanz 0.0) MUSS nach DocId aufsteigend sortiert sein (100 < 200 < 300)
    assert_eq!(results[0].doc_id, DocId::from(100u64));
    assert_eq!(results[1].doc_id, DocId::from(200u64));
    assert_eq!(results[2].doc_id, DocId::from(300u64));
}

#[tokio::test]
async fn test_wp05_k_bounds_validation() {
    let index = build_test_index(4);
    let query = [0.0f32; 4];

    // k = 0 muss leeren Vektor liefern
    let empty_results = index.search_adaptive(&query, 0).await.unwrap();
    assert!(empty_results.is_empty());

    // k > MAX_SEARCH_K muss Fehler liefern
    let err_k = index.search_adaptive(&query, MAX_SEARCH_K + 1).await;
    assert!(err_k.is_err(), "k > MAX_SEARCH_K must fail validation");
}

#[tokio::test]
async fn test_wp05_dimension_and_nan_inf_rejection() {
    let index = build_test_index(4);

    // Dimension mismatch
    let err_dim = index.search_adaptive(&[1.0f32, 2.0], 5).await;
    assert!(err_dim.is_err(), "Dimension mismatch must fail validation");

    // NaN in query
    let err_nan = index
        .search_adaptive(&[1.0f32, f32::NAN, 3.0, 4.0], 5)
        .await;
    assert!(err_nan.is_err(), "NaN in query must be rejected");

    // Inf in query
    let err_inf = index
        .search_adaptive(&[1.0f32, f32::INFINITY, 3.0, 4.0], 5)
        .await;
    assert!(err_inf.is_err(), "Inf in query must be rejected");

    // Insert with NaN
    let err_insert_nan = index
        .insert(1, DocId::from(1u64), &[f32::NAN, 0.0, 0.0, 0.0])
        .await;
    assert!(
        err_insert_nan.is_err(),
        "Insert with NaN vector must be rejected"
    );
}

#[tokio::test]
async fn test_wp05_batch_rollback_isolation() {
    let index = build_test_index(4);
    let tx1 = 10u64;
    let tx2 = 20u64;

    // Transaktion 1 committen
    index
        .insert(tx1, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
        .await
        .unwrap();
    index.commit(tx1).await.unwrap();

    // Transaktion 2 einfügen aber rollbacken
    index
        .insert(tx2, DocId::from(2u64), &[0.0, 1.0, 0.0, 0.0])
        .await
        .unwrap();
    index.rollback(tx2).await.unwrap();

    // Suche nach Vektor von Tx2 darf doc 2 NICHT sichtbar machen
    let results = index
        .search_adaptive(&[0.0, 1.0, 0.0, 0.0], 5)
        .await
        .unwrap();
    for res in results {
        assert_ne!(
            res.doc_id,
            DocId::from(2u64),
            "Uncommitted/rolled-back doc must not be visible"
        );
    }
}
