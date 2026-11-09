use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::types::Maintain;

#[tokio::test]
async fn test_inv_vec_connectivity_1_reachable_after_compact() {
    let dimension = 8;
    let config = HnswConfig {
        dimension,
        m: 8,
        ef_construction: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).expect("Valid HNSW config");

    // 1. Füge 30 Vektoren ein
    for i in 1..=30u64 {
        let doc_id = DocId::new(i);
        let vector: Vec<f32> = (0..dimension)
            .map(|d| (i * 10 + d as u64) as f32 / 100.0)
            .collect();
        index
            .insert(TxId::new(i), doc_id, &vector)
            .await
            .expect("Insert");
        index.commit(TxId::new(i)).await.expect("Commit");
    }

    // 2. Lösche 10 Knoten per Tombstone
    for i in 1..=10u64 {
        let doc_id = DocId::new(i);
        index.remove(doc_id).expect("remove succeeds");
    }

    // 3. Führe kontrollierten Rebuild via Maintain::compact aus
    index.compact().expect("compact must succeed");

    // 4. INV-VEC-CONNECTIVITY-1: Nach compact ist jeder lebende Knoten vom Einstiegspunkt erreichbar!
    let connected = index.verify_full_connectivity();
    assert!(
        connected,
        "INV-VEC-CONNECTIVITY-1: Every alive node MUST be reachable from entry point after compact"
    );

    // 5. Verifiziere Geisterzeigerfreiheit: Kein lebender Knoten darf eine Kante zu den gelöschten Knoten haben
    for i in 1..=10u64 {
        let doc_id = DocId::new(i);
        let residual = index.inspect_residual(doc_id);
        assert!(!residual.in_id_map, "Deleted doc_id must not be in id map");
        assert_eq!(
            residual.incoming_edge_count, 0,
            "No incoming ghost edge may reference deleted doc_id {i}"
        );
    }

    // 6. Verifiziere Recall: Alle lebenden Dokumente (11..=30) müssen auffindbar sein
    for i in 11..=30u64 {
        let vector: Vec<f32> = (0..dimension)
            .map(|d| (i * 10 + d as u64) as f32 / 100.0)
            .collect();
        let hits = index.search(&vector, 1).await.expect("Search");
        assert!(
            !hits.is_empty(),
            "Search must return results for alive nodes"
        );
        assert_eq!(
            hits[0].doc_id,
            DocId::new(i),
            "Top hit must be the exact vector"
        );
    }
}
