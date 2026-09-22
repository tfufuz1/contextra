use memfuse_core::{DistanceMetric, DocId, MemFuseError, TxId, VectorIndex};
use memfuse_vector::distance::{compute_distance_trusted, validate_vector};
use memfuse_vector::hnsw::HnswIndex;
use memfuse_vector::HnswConfig;

// BEWEIST: [Invariante] Vektoren mit NaN, Infinity oder Negative Infinity werden beim Einfügen in den Index konsistent mit Err(InvalidInput) abgelehnt.
#[tokio::test]
async fn proof_nan_rejected_at_insert() {
    let config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).expect("failed to create hnsw index");
    let tx = TxId::new(1);

    let invalid_values = [
        ("f32::NAN", f32::NAN),
        ("f32::INFINITY", f32::INFINITY),
        ("-f32::INFINITY", -f32::INFINITY),
        ("f32::NEG_INFINITY", f32::NEG_INFINITY),
    ];

    for (i, (name, val)) in invalid_values.iter().enumerate() {
        let mut test_vec = vec![1.0, 0.0, 0.0, 0.0];
        test_vec[1] = *val;
        let res = index.insert(tx, DocId::new(i as u64 + 1), &test_vec).await;

        assert!(
            matches!(res, Err(MemFuseError::InvalidInput(_))),
            "Expected InvalidInput for vector insert containing non-finite value {}, got: {:?}",
            name,
            res
        );
    }
}

// BEWEIST: [Invariante] compute_distance_trusted() führt im Hot-Path keine O(D) NaN-Scans durch und vertraut validierten Eingaben.
#[test]
fn proof_compute_distance_trusted_never_scans_nan() {
    let a = vec![1.0, f32::NAN, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0, 0.0];

    // compute_distance_trusted scans no NaN and does not return Err for NaN input
    let res = compute_distance_trusted(&a, &b, DistanceMetric::Cosine);
    assert!(
        res.is_ok(),
        "compute_distance_trusted in hot-path should trust input and not return Err on NaN check, got: {:?}",
        res
    );

    // Structural check on re-exported distance module
    let source = include_str!("../src/distance.rs");
    let trusted_fn_start = source
        .find("compute_distance_trusted")
        .expect("compute_distance_trusted reference found in distance.rs");

    let trusted_fn_body = &source[trusted_fn_start..trusted_fn_start + 600];
    assert!(
        !trusted_fn_body.contains("is_nan()"),
        "compute_distance_trusted must not contain is_nan() checks in hot path"
    );
    assert!(
        !trusted_fn_body.contains("is_finite()"),
        "compute_distance_trusted must not contain is_finite() checks in hot path"
    );
}

// BEWEIST: [Invariante] Nach einer Ablehnung wegen NaN-Werten bleibt der Index-Zustand sauber, sodass nachfolgende valide Inserts derselben DocId erfolgreich indiziert werden.
#[tokio::test]
async fn proof_valid_insert_after_nan_rejection() {
    let config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).expect("failed to create hnsw index");
    let tx = TxId::new(1);
    let target_doc = DocId::new(10);

    // 1. Invalid insert with NaN
    let nan_vec = vec![1.0, f32::NAN, 0.0, 0.0];
    let res_nan = index.insert(tx, target_doc, &nan_vec).await;
    assert!(
        matches!(res_nan, Err(MemFuseError::InvalidInput(_))),
        "Expected InvalidInput for NaN insert, got: {:?}",
        res_nan
    );

    // 2. Valid insert for same DocId
    let valid_vec = vec![1.0, 0.0, 0.0, 0.0];
    let res_valid = index.insert(tx, target_doc, &valid_vec).await;
    assert!(
        res_valid.is_ok(),
        "Valid insert for DocId 10 after rejected NaN insert must succeed, got: {:?}",
        res_valid
    );

    index.commit(tx).await.expect("commit should succeed");

    // 3. Search yields exactly 1 result
    let search_res = index
        .search(&[1.0, 0.0, 0.0, 0.0], 5)
        .await
        .expect("search should succeed");
    assert_eq!(
        search_res.len(),
        1,
        "Expected exactly 1 document in search results, got {}",
        search_res.len()
    );
    assert_eq!(
        search_res[0].doc_id, target_doc,
        "Search result doc_id should match target_doc 10"
    );
}

// BEWEIST: [Invariante] validate_vector() erkennt nicht-finite Werte an jeder Vektor-Position (Anfang, Mitte, Ende) über verschiedene Dimensionen hinweg.
#[test]
fn proof_validate_vector_covers_all_positions() {
    let dimensions = [4, 128, 768];

    for dim in dimensions {
        let valid_vec = vec![0.5f32; dim];
        assert!(
            validate_vector(&valid_vec).is_ok(),
            "Valid vector of dimension {} must pass validation",
            dim
        );

        let positions = [0, dim / 2, dim - 1];
        for pos in positions {
            let mut test_vec = valid_vec.clone();
            test_vec[pos] = f32::NAN;

            let res = validate_vector(&test_vec);
            assert!(
                matches!(res, Err(MemFuseError::InvalidInput(_))),
                "validate_vector must reject NaN at position {} for dimension {}, got: {:?}",
                pos,
                dim,
                res
            );
        }
    }
}

// BEWEIST: [Invariante] NaN in Query-Vektoren wird beim search()-Aufruf direkt abgelehnt, ohne den Index-Zustand oder bestehende Dokumente zu beeinträchtigen.
#[tokio::test]
async fn proof_nan_in_query_rejected_at_search() {
    let config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).expect("failed to create hnsw index");
    let tx = TxId::new(1);

    // Insert 2 valid documents
    let doc1 = DocId::new(101);
    let doc2 = DocId::new(102);
    index
        .insert(tx, doc1, &[1.0, 0.0, 0.0, 0.0])
        .await
        .expect("insert doc1");
    index
        .insert(tx, doc2, &[0.0, 1.0, 0.0, 0.0])
        .await
        .expect("insert doc2");
    index.commit(tx).await.expect("commit");

    // Search query with NaN
    let nan_query = vec![1.0, f32::NAN, 0.0, 0.0];
    let search_nan_res = index.search(&nan_query, 5).await;
    assert!(
        matches!(search_nan_res, Err(MemFuseError::InvalidInput(_))),
        "Expected InvalidInput error when querying with NaN vector, got: {:?}",
        search_nan_res
    );

    // Valid search afterwards returns both documents intact
    let valid_query = vec![1.0, 0.0, 0.0, 0.0];
    let search_valid_res = index
        .search(&valid_query, 5)
        .await
        .expect("valid search after failed NaN search must succeed");

    assert_eq!(
        search_valid_res.len(),
        2,
        "Valid documents must remain searchable after failed NaN query search"
    );
}
