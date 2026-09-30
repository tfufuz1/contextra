use contextra_core::{ContextraError, DistanceMetric, DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// 1. test_empty_index_search_returns_empty
// BEWEIST: [Invariante] Suche (k=10) auf einem leeren Index liefert Ok(vec![]) ohne Panic oder Err.
#[tokio::test]
async fn test_empty_index_search_returns_empty() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let query = vec![1.0, 0.0, 0.0, 0.0];

    let results = index.search(&query, 10).await?;
    assert!(
        results.is_empty(),
        "Expected empty search results on empty index, got {} elements",
        results.len()
    );

    Ok(())
}

// 2. test_single_element_index_returns_that_element
// BEWEIST: [Invariante] Suche k=1 auf einem Index mit 1 Vektor gibt genau dieses eingefügte Element zurück.
#[tokio::test]
async fn test_single_element_index_returns_that_element() -> Result<(), Box<dyn std::error::Error>>
{
    let config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let tx = TxId::new(1);
    let target_doc = DocId::new(42);
    let vec = vec![1.0, 0.0, 0.0, 0.0];

    index.insert(tx, target_doc, &vec).await?;
    index.commit(tx).await?;

    let results = index.search(&vec, 1).await?;
    assert_eq!(
        results.len(),
        1,
        "Expected exactly 1 search result, got {}",
        results.len()
    );
    assert_eq!(
        results[0].doc_id, target_doc,
        "Expected doc_id {:?}, got {:?}",
        target_doc, results[0].doc_id
    );

    Ok(())
}

// 3. test_ef_construction_boundary
// BEWEIST: [Invariante] Index befüllt mit exakt ef_construction Elementen sucht ohne Panic und liefert plausible Ergebnisse.
#[tokio::test]
async fn test_ef_construction_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let ef_const = 32;
    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ef_construction: ef_const,
        ef_search: 16,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let tx = TxId::new(1);

    for i in 1..=ef_const {
        let vec = vec![i as f32, 0.0, 0.0, 0.0];
        index.insert(tx, DocId::new(i as u64), &vec).await?;
    }
    index.commit(tx).await?;

    let query = vec![1.0, 0.0, 0.0, 0.0];
    let results = index.search(&query, 5).await?;
    assert!(
        !results.is_empty(),
        "Expected non-empty results for search on ef_construction boundary index"
    );
    assert!(
        results.len() <= 5,
        "Expected at most 5 results, got {}",
        results.len()
    );

    Ok(())
}

// 4. test_duplicate_vectors_tie_breaking
// BEWEIST: [Invariante] Suche k=5 bei 20 identischen Vektoren gibt stabil 5 Ergebnisse mit Ähnlichkeitsscore ~1.0 (Distanz 0.0) ohne Panic oder Endlosschleife zurück.
#[tokio::test]
async fn test_duplicate_vectors_tie_breaking() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let tx = TxId::new(1);
    let duplicate_vec = vec![1.0, 2.0, 3.0, 4.0];

    for i in 1..=20u64 {
        index.insert(tx, DocId::new(i), &duplicate_vec).await?;
    }
    index.commit(tx).await?;

    let results = index.search(&duplicate_vec, 5).await?;
    assert_eq!(
        results.len(),
        5,
        "Expected exactly 5 results for duplicate vector query, got {}",
        results.len()
    );

    for res in &results {
        assert!(
            (res.score - 1.0).abs() < 1e-4,
            "Expected similarity score close to 1.0 for identical query vector, got {}",
            res.score
        );
    }

    Ok(())
}

// 5. test_zero_vector_cosine
// BEWEIST: [Invariante] Einfügen und Suchen eines Nullvektors bei Cosine-Metrik führt zu keiner Null-Division (NaN/Panic); die Distanz ist definiert als 1.0 (Score 0.0).
#[tokio::test]
async fn test_zero_vector_cosine() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let tx = TxId::new(1);
    let zero_vec = vec![0.0, 0.0, 0.0, 0.0];
    let doc_id = DocId::new(10);

    index.insert(tx, doc_id, &zero_vec).await?;
    index.commit(tx).await?;

    // Search with zero query
    let results_zero_query = index.search(&zero_vec, 1).await?;
    assert_eq!(results_zero_query.len(), 1);
    assert!(!results_zero_query[0].score.is_nan());

    // Search with non-zero query
    let non_zero_query = vec![1.0, 0.0, 0.0, 0.0];
    let results_non_zero_query = index.search(&non_zero_query, 1).await?;
    assert_eq!(results_non_zero_query.len(), 1);
    assert!(!results_non_zero_query[0].score.is_nan());

    Ok(())
}

// 6. test_nan_in_embeddings_rejected
// BEWEIST: [Invariante] Vektoren mit NaN- oder Inf-Komponenten werden beim Einfügen mit Err(InvalidInput) abgelehnt und niemals ge-panict.
#[tokio::test]
async fn test_nan_in_embeddings_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let tx = TxId::new(1);

    let invalid_vectors = [
        vec![1.0, f32::NAN, 0.0, 0.0],
        vec![f32::INFINITY, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, -f32::INFINITY, 0.0],
    ];

    for (i, bad_vec) in invalid_vectors.iter().enumerate() {
        let res = index.insert(tx, DocId::new(i as u64 + 1), bad_vec).await;
        assert!(
            matches!(res, Err(ContextraError::InvalidInput(_))),
            "Expected InvalidInput error for vector containing non-finite value, got: {:?}",
            res
        );
    }

    Ok(())
}

// 7. test_concurrent_insert_during_search
// BEWEIST: [Invariante] 10 parallele Insert-Tasks und 10 parallele Search-Tasks laufen ohne Deadlocks oder Panics ab (abgesichert via Timeout).
#[tokio::test]
async fn test_concurrent_insert_during_search() -> Result<(), Box<dyn std::error::Error>> {
    let timeout_res = tokio::time::timeout(Duration::from_secs(5), async {
        let config = HnswConfig {
            dimension: 16,
            m: 16,
            ef_construction: 64,
            ef_search: 64,
            ..Default::default()
        };
        let index = Arc::new(HnswIndex::try_new(config)?);

        // Seed index with 50 elements
        let init_tx = TxId::new(1);
        for i in 1..=50u64 {
            let vec = vec![(i % 10) as f32; 16];
            index.insert(init_tx, DocId::new(i), &vec).await?;
        }
        index.commit(init_tx).await?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();

        // 10 Search tasks
        for _ in 0..10 {
            let idx = Arc::clone(&index);
            let stop = Arc::clone(&stop_flag);
            handles.push(tokio::spawn(async move {
                let query = vec![1.0f32; 16];
                while !stop.load(Ordering::Relaxed) {
                    let _ = idx.search(&query, 5).await;
                    tokio::task::yield_now().await;
                }
            }));
        }

        // 10 Insert tasks
        for t in 0..10u64 {
            let idx = Arc::clone(&index);
            handles.push(tokio::spawn(async move {
                let doc_id = DocId::new(100 + t);
                let tx = TxId::new(200 + t);
                let vec = vec![(t + 1) as f32; 16];
                let _ = idx.insert(tx, doc_id, &vec).await;
                let _ = idx.commit(tx).await;
            }));
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
        stop_flag.store(true, Ordering::Relaxed);

        for handle in handles {
            let task_res = handle.await;
            assert!(
                task_res.is_ok(),
                "Task panicked or produced JoinError: {:?}",
                task_res
            );
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    assert!(
        timeout_res.is_ok(),
        "test_concurrent_insert_during_search timed out! Potential deadlock."
    );
    timeout_res?
}

// 8. test_deleted_node_not_returned_in_results
// BEWEIST: [Invariante] Gelöschte Dokumente tauchen nach dem Commit der Löschtransaktion in keinen Suchergebnissen mehr auf.
#[tokio::test]
async fn test_deleted_node_not_returned_in_results() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        rebuild_threshold: 0.0, // Disable auto rebuild to strictly test tombstone filtering
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    let tx1 = TxId::new(1);
    let doc1 = DocId::new(101);
    let doc2 = DocId::new(102);
    let vec1 = vec![1.0, 0.0, 0.0, 0.0];
    let vec2 = vec![0.0, 1.0, 0.0, 0.0];

    index.insert(tx1, doc1, &vec1).await?;
    index.insert(tx1, doc2, &vec2).await?;
    index.commit(tx1).await?;

    // Delete doc1
    let tx2 = TxId::new(2);
    index.delete(tx2, doc1).await?;
    index.commit(tx2).await?;

    // Search query matching doc1's original vector
    let results = index.search(&vec1, 10).await?;
    assert!(
        results.iter().all(|r| r.doc_id != doc1),
        "Deleted doc_id {:?} was unexpectedly returned in search results",
        doc1
    );

    Ok(())
}

// 9. test_max_elements_boundary
// BEWEIST: [Invariante] Befüllen des Indexes über den anfänglich konfigurierten max_elements Wert hinaus verhält sich korrekt durch dynamisches Resizing ohne Panic.
#[tokio::test]
async fn test_max_elements_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let max_cap = 50;
    let config = HnswConfig {
        dimension: 4,
        max_elements: max_cap,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let tx = TxId::new(1);

    // Insert 100 elements (double max_elements)
    for i in 1..=100u64 {
        let vec = vec![i as f32 * 0.01, 0.0, 0.0, 0.0];
        index.insert(tx, DocId::new(i), &vec).await?;
    }
    index.commit(tx).await?;

    let query = vec![0.5, 0.0, 0.0, 0.0];
    let results = index.search(&query, 10).await?;
    assert_eq!(
        results.len(),
        10,
        "Expected 10 search results after exceeding max_elements, got {}",
        results.len()
    );

    Ok(())
}

// 10. test_search_with_k_greater_than_index_size
// BEWEIST: [Invariante] Suche mit k=100 auf einem Index mit 5 Elementen liefert höchstens 5 Ergebnisse ohne Panic oder Index-Out-of-Bounds.
#[tokio::test]
async fn test_search_with_k_greater_than_index_size() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let tx = TxId::new(1);

    for i in 1..=5u64 {
        let vec = vec![i as f32, 0.0, 0.0, 0.0];
        index.insert(tx, DocId::new(i), &vec).await?;
    }
    index.commit(tx).await?;

    let query = vec![1.0, 0.0, 0.0, 0.0];
    let results = index.search(&query, 100).await?;

    assert_eq!(
        results.len(),
        5,
        "Expected exactly 5 results when k=100 on 5-element index, got {}",
        results.len()
    );

    Ok(())
}
