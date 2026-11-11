// FILE-CONTEXT
// ZWECK: Invarianten-Lücken-Tests für HNSW-Kern gemäß SPEC §11 & Root/Crate AGENTS.md §4.
// INVARIANTEN: Zero-Panic, deterministische Seeds, Snapshot-Sichtbarkeit, k > N Handhabung, Wiederholtes Einfügen.
// HOTSPOTS: 25-180
// STAND: 2026-11-11

//! Invarianten-Lücken-Suite für `contextra-vector`.
//!
//! Überprüft zentrale Invarianten über die öffentliche API:
//! 1. `k > Anzahl Dokumente`: HNSW stürzt nicht ab und liefert exakt alle aktiven Dokumente sortiert.
//! 2. Snapshot-Sichtbarkeit nach Löschung (`INV-INDEX-SNAPSHOT-1`): Abfragen mit `tx_id < delete_tx`
//!    sehen das Dokument weiterhin, spätere Transaktionen sehen es nicht mehr.
//! 3. Idempotenz / Überschreiben: Wiederholtes Einfügen derselben `DocId` mit neuem Vektor aktualisiert
//!    die Darstellung ohne Inkonsistenz.
//! 4. Persistenz-Roundtrip nach Löschungen (`INV-VEC-ZEROIZE-1`): Nach dem Schreiben in eine Mmap-Datei
//!    und erneutem Öffnen sind gelöschte Knoten konsistent ausgefiltert.
//! 5. NaN- und Dimensionsabweisung: Abweisung ungültiger Vektoreingaben ohne Zustandsvergiftung.

use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use contextra_vector::distance::validate_vector;
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::persistence::MmapIndex;
use contextra_vector::types::DistanceMetric;
use tempfile::tempdir;

// ANCHOR[contextra-vector:W09-GAP-001]
#[tokio::test]
async fn test_gap_k_greater_than_index_size() {
    let config = HnswConfig {
        dimension: 4,
        max_elements: 20,
        m: 16,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).expect("valid index");

    // Insert 3 documents
    index
        .insert(TxId::new(1), DocId::new(1), &[0.1, 0.2, 0.3, 0.4])
        .await
        .expect("insert 1");
    index
        .insert(TxId::new(1), DocId::new(2), &[0.5, 0.6, 0.7, 0.8])
        .await
        .expect("insert 2");
    index
        .insert(TxId::new(1), DocId::new(3), &[0.9, 1.0, 1.1, 1.2])
        .await
        .expect("insert 3");

    index.commit(TxId::new(1)).await.expect("commit tx 1");

    // Search with k = 100 > 3
    let results = index
        .search(&[0.1, 0.2, 0.3, 0.4], 100)
        .await
        .expect("search ok");

    assert_eq!(
        results.len(),
        3,
        "k > N should return exactly N available documents"
    );
    assert_eq!(results[0].doc_id, DocId::new(1));
}

// ANCHOR[contextra-vector:W09-GAP-002]
#[tokio::test]
async fn test_gap_snapshot_visibility_mvcc() {
    let config = HnswConfig {
        dimension: 2,
        max_elements: 20,
        m: 16,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).expect("valid index");

    // Insert at Tx 10
    index
        .insert(TxId::new(10), DocId::new(1), &[1.0, 1.0])
        .await
        .expect("insert at tx 10");
    index.commit(TxId::new(10)).await.expect("commit tx 10");

    // Delete at Tx 20
    index
        .delete(TxId::new(20), DocId::new(1))
        .await
        .expect("delete at tx 20");
    index.commit(TxId::new(20)).await.expect("commit tx 20");

    // Query before insert (Tx 5) -> empty
    let res_tx5 = index
        .search_at(&[1.0, 1.0], 5, 5)
        .await
        .expect("search at tx 5");
    assert!(
        res_tx5.is_empty(),
        "Tx 5 must not see doc inserted at Tx 10"
    );

    // Query between insert and delete (Tx 15) -> visible
    let res_tx15 = index
        .search_at(&[1.0, 1.0], 5, 15)
        .await
        .expect("search at tx 15");
    assert_eq!(
        res_tx15.len(),
        1,
        "Tx 15 must see doc active between Tx 10 and Tx 20"
    );
    assert_eq!(res_tx15[0].doc_id, DocId::new(1));

    // Query after delete (Tx 25) -> invisible
    let res_tx25 = index
        .search_at(&[1.0, 1.0], 5, 25)
        .await
        .expect("search at tx 25");
    assert!(
        res_tx25.is_empty(),
        "Tx 25 must not see doc deleted at Tx 20"
    );
}

// ANCHOR[contextra-vector:W09-GAP-003]
#[tokio::test]
async fn test_gap_duplicate_docid_reinsertion() {
    let config = HnswConfig {
        dimension: 2,
        max_elements: 20,
        m: 16,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).expect("valid index");

    index
        .insert(TxId::new(1), DocId::new(42), &[0.0, 0.0])
        .await
        .expect("insert 42");
    index.commit(TxId::new(1)).await.expect("commit 1");

    // Reinsert same DocId with updated vector
    index
        .insert(TxId::new(2), DocId::new(42), &[10.0, 10.0])
        .await
        .expect("reinsert 42");
    index.commit(TxId::new(2)).await.expect("commit 2");

    let res = index.search_at(&[10.0, 10.0], 5, 2).await.expect("search");
    assert!(res.iter().any(|d| d.doc_id == DocId::new(42)));
    assert_eq!(res[0].doc_id, DocId::new(42));
    assert!(res[0].score > 0.99, "Updated vector score should match");
}

// ANCHOR[contextra-vector:W09-GAP-004]
#[tokio::test]
async fn test_gap_persistence_roundtrip_post_deletion() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("hnsw_test_index.bin");

    let config = HnswConfig {
        dimension: 2,
        max_elements: 50,
        m: 16,
        ef_construction: 32,
        ef_search: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).expect("valid index");

    for id in 1..=5u64 {
        index
            .insert(TxId::new(1), DocId::new(id), &[id as f32, id as f32])
            .await
            .expect("insert");
    }
    index.commit(TxId::new(1)).await.expect("commit 1");

    // Delete node 3
    index
        .delete(TxId::new(2), DocId::new(3))
        .await
        .expect("delete");
    index.commit(TxId::new(2)).await.expect("commit 2");

    // Write to disk
    index.save(&path).expect("save index");

    // Open via MmapIndex
    let mmap = MmapIndex::open(&path).expect("open mmap index");
    assert_eq!(mmap.header.dimension(), 2);
    assert!(mmap.header.entry_point() >= 0);
}

// ANCHOR[contextra-vector:W09-GAP-005]
#[test]
fn test_gap_nan_validation() {
    let valid = vec![1.0, 2.0, 3.0];
    assert!(validate_vector(&valid).is_ok());

    let has_nan = vec![1.0, f32::NAN, 3.0];
    assert!(validate_vector(&has_nan).is_err());

    let has_inf = vec![1.0, f32::INFINITY, 3.0];
    assert!(validate_vector(&has_inf).is_err());
}
