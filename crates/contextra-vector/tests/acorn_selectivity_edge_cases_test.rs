use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::acorn::{compute_gamma_edge_budget, FilteredIndex, NaiveReferenceIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};

#[tokio::test]
async fn test_acorn_selectivity_zero_and_one_and_oracle() {
    let dimension = 4;
    let config = HnswConfig {
        dimension,
        m: 8,
        ef_construction: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).expect("Valid HNSW config");
    let mut oracle = NaiveReferenceIndex::new(DistanceMetric::Euclidean);

    // 20 Dokumente einfügen
    let tx = TxId::new(1);
    for i in 1..=20u64 {
        let doc_id = DocId::new(i);
        let val = i as f32;
        let vec = vec![val, val * 0.5, val * 0.25, val * 0.1];
        index.insert(tx, doc_id, &vec).await.expect("Insert");
        oracle.insert(doc_id, vec);
    }
    index.commit(tx).await.expect("Commit");

    let query = vec![5.0, 2.5, 1.25, 0.5];

    // 1. Selektivität 0.0: Prädikat matcht NIEMALS (immer false)
    let pred_zero = |_id: DocId| false;
    let res_zero = index
        .search_knn_acorn(&query, 5, &pred_zero, 2)
        .expect("Search selectivity 0");
    let oracle_zero = oracle
        .search_knn_acorn(&query, 5, &pred_zero, 2)
        .expect("Oracle selectivity 0");
    assert!(res_zero.is_empty(), "Selectivity 0 must return 0 hits");
    assert_eq!(res_zero, oracle_zero, "Must match naive oracle exactly");

    // 2. Selektivität 1.0: Prädikat matcht IMMER (immer true)
    let pred_one = |_id: DocId| true;
    let res_one = index
        .search_knn_acorn(&query, 5, &pred_one, 1)
        .expect("Search selectivity 1");
    let oracle_one = oracle
        .search_knn_acorn(&query, 5, &pred_one, 1)
        .expect("Oracle selectivity 1");
    assert_eq!(res_one.len(), 5, "Selectivity 1 must return k hits");
    assert_eq!(res_one.len(), oracle_one.len());
    for (i, hit) in res_one.iter().enumerate() {
        assert_eq!(hit.0, oracle_one[i].0, "Top hits must match oracle");
    }

    // 3. Mittlere Selektivität: Nur gerade DocIds
    let pred_even = |id: DocId| id.inner() % 2 == 0;
    let gamma = compute_gamma_edge_budget(8, 0.5) as u32;
    let res_even = index
        .search_knn_acorn(&query, 3, &pred_even, gamma.max(1))
        .expect("Search even");
    for (doc, _) in &res_even {
        assert_eq!(doc.inner() % 2, 0, "All returned documents must be even");
    }

    // 4. Gelöschte Knoten dürfen nie im ACORN-Ergebnis auftauchen
    index.remove(DocId::new(4)).expect("Remove doc 4");
    index.remove(DocId::new(6)).expect("Remove doc 6");

    let res_even_after_delete = index
        .search_knn_acorn(&query, 5, &pred_even, gamma.max(1))
        .expect("Search even after delete");
    for (doc, _) in res_even_after_delete {
        assert_ne!(
            doc,
            DocId::new(4),
            "Deleted doc 4 must never appear in ACORN"
        );
        assert_ne!(
            doc,
            DocId::new(6),
            "Deleted doc 6 must never appear in ACORN"
        );
    }
}
