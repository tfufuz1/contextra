use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};

#[tokio::test]
async fn test_inv_vec_zeroize_1_slot_nulling_on_deletion() {
    let dimension = 8;
    let config = HnswConfig {
        dimension,
        m: 8,
        ef_construction: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).expect("Valid HNSW config");
    let tx = TxId::new(1);
    let doc_id = DocId::new(42);
    let vector = vec![1.5f32, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5];

    index.insert(tx, doc_id, &vector).await.expect("Insert");
    index.commit(tx).await.expect("Commit");

    // Vor Löschung: inspect_residual bestätigt Vektor vorhanden
    let report_before = index.inspect_residual(doc_id);
    assert!(report_before.in_id_map);
    assert!(report_before.non_zero_vector_bytes);

    // O(1) Tombstone-Löschung gemäß INV-VEC-ZEROIZE-1
    let removed = index.remove(doc_id).expect("remove succeeds");
    assert!(removed, "Document must have been removed");

    // Nach Löschung: ID nicht mehr in id_map, und Vektor-Bytes im Slot sind genullt!
    let report_after = index.inspect_residual(doc_id);
    assert!(
        !report_after.in_id_map,
        "doc_id must be removed from ID map"
    );
    assert!(
        !report_after.non_zero_vector_bytes,
        "INV-VEC-ZEROIZE-1: Vector bytes in memory slot MUST be zeroized on deletion"
    );

    // Suche liefert gelöschtes Dokument nicht mehr
    let results = index.search(&vector, 10).await.expect("Search");
    for hit in results {
        assert_ne!(
            hit.doc_id, doc_id,
            "Deleted doc must never appear in search"
        );
    }
}
