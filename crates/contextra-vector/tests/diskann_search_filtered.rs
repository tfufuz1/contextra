//! Integration tests for DiskAnnIndex::search_filtered via adaptive oversampling.

#![cfg(feature = "experimental-diskann")]

use contextra_core::{ContextraError, DistanceMetric, DocId, VectorIndex};
use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};

async fn setup_test_index(num_vectors: usize) -> (DiskAnnIndex, Vec<Vec<f32>>, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index_path = temp_dir.path().join("filtered_test.idx");

    let config = DiskAnnConfig {
        index_path,
        dimension: 4,
        max_degree: 16,
        beam_width: 32,
        sector_size: 4096,
        distance_metric: DistanceMetric::Euclidean,
        quantize: false,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config).expect("valid DiskAnnConfig");

    let mut vectors = Vec::with_capacity(num_vectors);
    let mut ids = Vec::with_capacity(num_vectors);

    for i in 1..=num_vectors {
        let v = vec![(i as f32) * 0.1, 0.0, 0.0, 0.0];
        vectors.push(v);
        ids.push(DocId::from(i as u64));
    }

    index
        .build(&vectors, &ids)
        .await
        .expect("DiskANN build failed");

    (index, vectors, temp_dir)
}

#[tokio::test]
async fn test_diskann_search_filtered_even_doc_ids() {
    let (index, _, _dir) = setup_test_index(300).await;
    let query = vec![10.0, 0.0, 0.0, 0.0];
    let k = 10;

    let filter = |doc_id: DocId| doc_id.inner() % 2 == 0;
    let results = index
        .search_filtered(&query, k, Some(&filter))
        .await
        .expect("search_filtered failed");

    assert!(!results.is_empty(), "Results should not be empty");
    assert!(results.len() <= k, "Should return at most k results");

    for res in &results {
        assert!(
            res.doc_id.inner() % 2 == 0,
            "DocId {} is not even",
            res.doc_id.inner()
        );
    }

    // Verify descending score order (with ascending doc_id tie-breaker)
    for i in 1..results.len() {
        let prev = &results[i - 1];
        let curr = &results[i];
        assert!(
            prev.score > curr.score
                || ((prev.score - curr.score).abs() < 1e-6
                    && prev.doc_id.inner() <= curr.doc_id.inner()),
            "Results not sorted by score descending / doc_id ascending"
        );
    }
}

#[tokio::test]
async fn test_diskann_search_filtered_reject_all() {
    let (index, _, _dir) = setup_test_index(300).await;
    let query = vec![10.0, 0.0, 0.0, 0.0];
    let k = 10;

    let filter = |_doc_id: DocId| false;
    let results = index
        .search_filtered(&query, k, Some(&filter))
        .await
        .expect("search_filtered failed");

    assert!(
        results.is_empty(),
        "Results should be empty when predicate rejects all docs"
    );
}

#[tokio::test]
async fn test_diskann_search_filtered_none_filter() {
    let (index, _, _dir) = setup_test_index(300).await;
    let query = vec![10.0, 0.0, 0.0, 0.0];
    let k = 10;

    let filtered_res = index
        .search_filtered(&query, k, None)
        .await
        .expect("search_filtered with None failed");

    let direct_res = index
        .search(&query, k)
        .await
        .expect("search direct failed");

    assert_eq!(
        filtered_res.len(),
        direct_res.len(),
        "Result count mismatch between search_filtered(None) and search()"
    );

    for (f_doc, d_doc) in filtered_res.iter().zip(direct_res.iter()) {
        assert_eq!(f_doc.doc_id, d_doc.doc_id);
        assert!((f_doc.score - d_doc.score).abs() < 1e-6);
    }
}

#[tokio::test]
async fn test_diskann_search_filtered_highly_selective() {
    let (index, _, _dir) = setup_test_index(300).await;
    let query = vec![10.0, 0.0, 0.0, 0.0];
    let k = 5;

    // Selective filter: only 1 in 100 docs passes (DocIds 100, 200, 300)
    let filter = |doc_id: DocId| doc_id.inner() % 100 == 0;

    let results = index
        .search_filtered(&query, k, Some(&filter))
        .await
        .expect("search_filtered failed");

    // Compute expected brute-force post-filter over full search
    let raw_all = index
        .search(&query, contextra_core::MAX_SEARCH_K)
        .await
        .expect("search all failed");

    let expected: Vec<_> = raw_all
        .into_iter()
        .filter(|doc| doc.doc_id.inner() % 100 == 0)
        .take(k)
        .collect();

    assert_eq!(
        results.len(),
        expected.len(),
        "Selective filter result count mismatch"
    );

    for (actual, exp) in results.iter().zip(expected.iter()) {
        assert_eq!(actual.doc_id, exp.doc_id);
    }
}

#[tokio::test]
async fn test_diskann_search_filtered_k_zero() {
    let (index, _, _dir) = setup_test_index(300).await;
    let query = vec![10.0, 0.0, 0.0, 0.0];

    let filter = |doc_id: DocId| doc_id.inner() % 2 == 0;
    let res = index.search_filtered(&query, 0, Some(&filter)).await;

    assert!(
        matches!(res, Err(ContextraError::InvalidInput(_))),
        "k = 0 should return ContextraError::InvalidInput"
    );
}

#[tokio::test]
async fn test_diskann_search_at_capability_unsupported() {
    let (index, _, _dir) = setup_test_index(300).await;
    let query = vec![10.0, 0.0, 0.0, 0.0];

    let res = index.search_at(&query, 5, 100).await;

    match res {
        Err(ContextraError::CapabilityUnsupported { capability, .. }) => {
            assert_eq!(capability, "snapshot_read_at");
        }
        other => panic!("Expected CapabilityUnsupported('snapshot_read_at'), got: {other:?}"),
    }
}

#[tokio::test]
async fn test_diskann_search_filtered_deterministic() {
    let (index, _, _dir) = setup_test_index(300).await;
    let query = vec![10.0, 0.0, 0.0, 0.0];
    let k = 10;

    let filter = |doc_id: DocId| doc_id.inner() % 3 == 0;

    let run1 = index
        .search_filtered(&query, k, Some(&filter))
        .await
        .expect("run1 failed");

    let run2 = index
        .search_filtered(&query, k, Some(&filter))
        .await
        .expect("run2 failed");

    assert_eq!(run1.len(), run2.len());

    for (a, b) in run1.iter().zip(run2.iter()) {
        assert_eq!(a.doc_id, b.doc_id);
        assert_eq!(a.score.to_bits(), b.score.to_bits());
    }
}
